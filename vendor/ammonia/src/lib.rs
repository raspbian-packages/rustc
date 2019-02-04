// Copyright (C) Michael Howell and others
// this library is released under the same terms as Rust itself.

//! Ammonia is a whitelist-based HTML sanitization library. It is designed to
//! prevent cross-site scripting, layout breaking, and clickjacking caused
//! by untrusted user-provided HTML being mixed into a larger web page.
//!
//! Ammonia uses [html5ever] to parse and serialize document fragments the same way browsers do,
//! so it is extremely resilient to syntactic obfuscation.
//!
//! Ammonia parses its input exactly according to the HTML5 specification;
//! it will not linkify bare URLs, insert line or paragraph breaks, or convert `(C)` into &copy;.
//! If you want that, use a markup processor before running the sanitizer, like [pulldown-cmark].
//!
//! # Examples
//!
//! ```
//! let result = ammonia::clean(
//!     "<b><img src='' onerror='alert(\\'hax\\')'>I'm not trying to XSS you</b>"
//! );
//! assert_eq!(result, "<b><img src=\"\">I'm not trying to XSS you</b>");
//! ```
//!
//! [html5ever]: https://github.com/servo/html5ever "The HTML parser in Servo"
//! [pulldown-cmark]: https://github.com/google/pulldown-cmark "CommonMark parser"

#[macro_use]
extern crate html5ever;
#[macro_use]
extern crate lazy_static;
#[macro_use]
extern crate maplit;
#[macro_use]
extern crate matches;
#[macro_use]
extern crate tendril;

pub extern crate url;

use html5ever::{driver as html, QualName};
use html5ever::rcdom::{Handle, NodeData, RcDom};
use html5ever::serialize::{serialize, SerializeOpts};
use html5ever::tree_builder::{NodeOrText, TreeSink};
use html5ever::interface::Attribute;
use std::borrow::Cow;
use std::collections::{HashMap, HashSet};
use std::fmt;
use std::io;
use std::mem::replace;
use std::rc::Rc;
use std::str::FromStr;
use tendril::stream::TendrilSink;
use tendril::StrTendril;
pub use url::Url;

lazy_static! {
    static ref AMMONIA: Builder<'static> = Builder::default();
}

/// Clean HTML with a conservative set of defaults.
///
/// * [tags](struct.Builder.html#defaults)
/// * [attributes on specific tags](struct.Builder.html#defaults-1)
/// * [attributes on all tags](struct.Builder.html#defaults-2)
/// * [url schemes](struct.Builder.html#defaults-3)
/// * [relative URLs are passed through, unchanged, by default](struct.Builder.html#defaults-4)
/// * [links are marked `noopener noreferrer` by default](struct.Builder.html#defaults-5)
/// * [all `class=""` settings are blocked by default](struct.Builder.html#defaults-6)
/// * [comments are stripped by default](struct.Builder.html#defaults-7)
///
/// [opener]: https://mathiasbynens.github.io/rel-noopener/
/// [referrer]: https://en.wikipedia.org/wiki/HTTP_referer
///
/// # Examples
///
///     assert_eq!(ammonia::clean("<script>XSS</script>"), "XSS")
pub fn clean(src: &str) -> String {
    AMMONIA.clean(src).to_string()
}

/// An HTML sanitizer.
///
/// Given a fragment of HTML, Ammonia will parse it according to the HTML5
/// parsing algorithm and sanitize any disallowed tags or attributes. This
/// algorithm also takes care of things like unclosed and (some) misnested
/// tags.
///
/// # Examples
///
///     use ammonia::{Builder, UrlRelative};
///
///     let a = Builder::default()
///         .link_rel(None)
///         .url_relative(UrlRelative::PassThrough)
///         .clean("<a href=/>test")
///         .to_string();
///     assert_eq!(
///         a,
///         "<a href=\"/\">test</a>");
///
/// # Panics
///
/// Running [`clean`] or [`clean_from_reader`] may cause a panic if the builder is
/// configured with any of these (contradictory) settings:
///
///  * The `rel` attribute is added to [`generic_attributes`] or the
///    [`tag_attributes`] for the `<a>` tag, and [`link_rel`] is not set to `None`.
///
///    For example, this is going to panic, since [`link_rel`] is set  to
///    `Some("noopener noreferrer")` by default,
///    and it makes no sense to simultaneously say that the user is allowed to
///    set their own `rel` attribute while saying that every link shall be set to
///    a particular value:
///
///    ```should_panic
///    #[macro_use]
///    extern crate maplit;
///    # extern crate ammonia;
///
///    use ammonia::Builder;
///
///    # fn main() {
///    Builder::default()
///        .generic_attributes(hashset!["rel"])
///        .clean("");
///    # }
///    ```
///
///    This, however, is perfectly valid:
///
///    ```
///    #[macro_use]
///    extern crate maplit;
///    # extern crate ammonia;
///
///    use ammonia::Builder;
///
///    # fn main() {
///    Builder::default()
///        .generic_attributes(hashset!["rel"])
///        .link_rel(None)
///        .clean("");
///    # }
///    ```
///
///  * The `class` attribute is in [`allowed_classes`] and is in the
///    corresponding [`tag_attributes`] or in [`generic_attributes`].
///
///    This is done both to line up with the treatment of `rel`,
///    and to prevent people from accidentally allowing arbitrary
///    classes on a particular element.
///
///    This will panic:
///
///    ```should_panic
///    #[macro_use]
///    extern crate maplit;
///    # extern crate ammonia;
///
///    use ammonia::Builder;
///
///    # fn main() {
///    Builder::default()
///        .generic_attributes(hashset!["class"])
///        .allowed_classes(hashmap!["span" => hashset!["hidden"]])
///        .clean("");
///    # }
///    ```
///
///    This, however, is perfectly valid:
///
///    ```
///    #[macro_use]
///    extern crate maplit;
///    # extern crate ammonia;
///
///    use ammonia::Builder;
///
///    # fn main() {
///    Builder::default()
///        .allowed_classes(hashmap!["span" => hashset!["hidden"]])
///        .clean("");
///    # }
///    ```
/// 
///  * A tag is in either [`tags`] or [`tag_attributes`] while also
///    being in [`clean_content_tags`].
/// 
///    Both [`tags`] and [`tag_attributes`] are whitelists but
///    [`clean_content_tags`] is a blacklist, so it doesn't make sense
///    to have the same tag in both.
/// 
///    For example, this will panic, since the `aside` tag is in
///    [`tags`] by default:
/// 
///    ```should_panic
///    #[macro_use]
///    extern crate maplit;
///    # extern crate ammonia;
///    
///    use ammonia::Builder;
///    
///    # fn main() {
///    Builder::default()
///        .clean_content_tags(hashset!["aside"])
///        .clean("");
///    # }
///    ```
/// 
///    This, however, is valid:
/// 
///    ```
///    #[macro_use]
///    extern crate maplit;
///    # extern crate ammonia;
///    
///    use ammonia::Builder;
///    
///    # fn main() {
///    Builder::default()
///        .rm_tags(std::iter::once("aside"))
///        .clean_content_tags(hashset!["aside"])
///        .clean("");
///    # }
///    ```
///
/// [`clean`]: #method.clean
/// [`clean_from_reader`]: #method.clean_from_reader
/// [`generic_attributes`]: #method.generic_attributes
/// [`tag_attributes`]: #method.tag_attributes
/// [`generic_attributes`]: #method.generic_attributes
/// [`link_rel`]: #method.link_rel
/// [`allowed_classes`]: #method.allowed_classes
/// [`id_prefix`]: #method.id_prefix
/// [`tags`]: #method.tags
/// [`clean_content_tags`]: #method.clean_content_tags
#[derive(Debug)]
pub struct Builder<'a> {
    tags: HashSet<&'a str>,
    clean_content_tags: HashSet<&'a str>,
    tag_attributes: HashMap<&'a str, HashSet<&'a str>>,
    generic_attributes: HashSet<&'a str>,
    url_schemes: HashSet<&'a str>,
    url_relative: UrlRelative,
    link_rel: Option<&'a str>,
    allowed_classes: HashMap<&'a str, HashSet<&'a str>>,
    strip_comments: bool,
    id_prefix: Option<&'a str>,
}

impl<'a> Default for Builder<'a> {
    fn default() -> Self {
        #[cfg_attr(rustfmt, rustfmt_skip)]
        let tags = hashset![
            "a", "abbr", "acronym", "area", "article", "aside", "b", "bdi",
            "bdo", "blockquote", "br", "caption", "center", "cite", "code",
            "col", "colgroup", "data", "dd", "del", "details", "dfn", "div",
            "dl", "dt", "em", "figcaption", "figure", "footer", "h1", "h2",
            "h3", "h4", "h5", "h6", "header", "hgroup", "hr", "i", "img",
            "ins", "kbd", "kbd", "li", "map", "mark", "nav", "ol", "p", "pre",
            "q", "rp", "rt", "rtc", "ruby", "s", "samp", "small", "span",
            "strike", "strong", "sub", "summary", "sup", "table", "tbody",
            "td", "th", "thead", "time", "tr", "tt", "u", "ul", "var", "wbr"
        ];
        let clean_content_tags = hashset![];
        let generic_attributes = hashset![
            "lang", "title"
        ];
        let tag_attributes = hashmap![
            "a" => hashset![
                "href", "hreflang"
            ],
            "bdo" => hashset![
                "dir"
            ],
            "blockquote" => hashset![
                "cite"
            ],
            "col" => hashset![
                "align", "char", "charoff", "span"
            ],
            "colgroup" => hashset![
                "align", "char", "charoff", "span"
            ],
            "del" => hashset![
                "cite", "datetime"
            ],
            "hr" => hashset![
                "align", "size", "width"
            ],
            "img" => hashset![
                "align", "alt", "height", "src", "width"
            ],
            "ins" => hashset![
                "cite", "datetime"
            ],
            "ol" => hashset![
                "start"
            ],
            "q" => hashset![
                "cite"
            ],
            "table" => hashset![
                "align", "char", "charoff", "summary"
            ],
            "tbody" => hashset![
                "align", "char", "charoff"
            ],
            "td" => hashset![
                "align", "char", "charoff", "colspan", "headers", "rowspan"
            ],
            "tfoot" => hashset![
                "align", "char", "charoff"
            ],
            "th" => hashset![
                "align", "char", "charoff", "colspan", "headers", "rowspan", "scope"
            ],
            "thead" => hashset![
                "align", "char", "charoff"
            ],
            "tr" => hashset![
                "align", "char", "charoff"
            ],
        ];
        let url_schemes = hashset![
            "bitcoin", "ftp", "ftps", "geo", "http", "https", "im", "irc",
            "ircs", "magnet", "mailto", "mms", "mx", "news", "nntp",
            "openpgp4fpr", "sip", "sms", "smsto", "ssh", "tel", "url",
            "webcal", "wtai", "xmpp"
        ];
        let allowed_classes = hashmap![];

        Builder {
            tags: tags,
            clean_content_tags: clean_content_tags,
            tag_attributes: tag_attributes,
            generic_attributes: generic_attributes,
            url_schemes: url_schemes,
            url_relative: UrlRelative::PassThrough,
            link_rel: Some("noopener noreferrer"),
            allowed_classes: allowed_classes,
            strip_comments: true,
            id_prefix: None,
        }
    }
}

impl<'a> Builder<'a> {
    /// Sets the tags that are allowed.
    ///
    /// # Examples
    ///
    ///     #[macro_use]
    ///     extern crate maplit;
    ///     # extern crate ammonia;
    ///
    ///     use ammonia::Builder;
    ///
    ///     # fn main() {
    ///     let tags = hashset!["my-tag"];
    ///     let a = Builder::new()
    ///         .tags(tags)
    ///         .clean("<my-tag>")
    ///         .to_string();
    ///     assert_eq!(a, "<my-tag></my-tag>");
    ///     # }
    ///
    /// # Defaults
    ///
    /// ```notest
    /// a, abbr, acronym, area, article, aside, b, bdi,
    /// bdo, blockquote, br, caption, center, cite, code,
    /// col, colgroup, data, dd, del, details, dfn, div,
    /// dl, dt, em, figcaption, figure, footer, h1, h2,
    /// h3, h4, h5, h6, header, hgroup, hr, i, img,
    /// ins, kbd, kbd, li, map, mark, nav, ol, p, pre,
    /// q, rp, rt, rtc, ruby, s, samp, small, span,
    /// strike, strong, sub, summary, sup, table, tbody,
    /// td, th, thead, time, tr, tt, u, ul, var, wbr
    /// ```
    pub fn tags(&mut self, value: HashSet<&'a str>) -> &mut Self {
        self.tags = value;
        self
    }

    /// Add additonal whitelisted tags without overwriting old ones.
    ///
    /// Does nothing if the tag is already there.
    ///
    /// # Examples
    ///
    ///     let a = ammonia::Builder::default()
    ///         .add_tags(std::iter::once("my-tag"))
    ///         .clean("<my-tag>test</my-tag> <span>mess</span>").to_string();
    ///     assert_eq!("<my-tag>test</my-tag> <span>mess</span>", a);
    pub fn add_tags<I: Iterator<Item=&'a str>>(&mut self, it: I) -> &mut Self {
        self.tags.extend(it);
        self
    }

    /// Remove already-whitelisted tags.
    ///
    /// Does nothing if the tags is already gone.
    ///
    /// # Examples
    ///
    ///     let a = ammonia::Builder::default()
    ///         .rm_tags(std::iter::once("span"))
    ///         .clean("<span></span>").to_string();
    ///     assert_eq!("", a);
    pub fn rm_tags<'b, I: Iterator<Item=&'b str>>(&mut self, it: I) -> &mut Self {
        for i in it {
            self.tags.remove(i);
        }
        self
    }

    /// Returns a copy of the set of whitelisted tags.
    ///
    /// # Examples
    ///
    ///     let tags = ["my-tag-1", "my-tag-2"].into_iter().cloned().collect();
    ///     let mut b = ammonia::Builder::default();
    ///     b.tags(Clone::clone(&tags));
    ///     assert_eq!(tags, b.clone_tags());
    pub fn clone_tags(&self) -> HashSet<&'a str> {
        self.tags.clone()
    }

    /// Sets the tags whose contents will be completely removed from the output.
    ///
    /// Adding tags which are whitelisted in `tags` or `tag_attributes` will cause
    /// a panic.
    /// 
    /// # Examples
    ///
    ///     #[macro_use]
    ///     extern crate maplit;
    ///     # extern crate ammonia;
    ///
    ///     use ammonia::Builder;
    ///
    ///     # fn main() {
    ///     let tag_blacklist = hashset!["script", "style"];
    ///     let a = Builder::new()
    ///         .clean_content_tags(tag_blacklist)
    ///         .clean("<script>alert('hello')</script><style>a { background: #fff }</style>")
    ///         .to_string();
    ///     assert_eq!(a, "");
    ///     # }
    ///
    /// # Defaults
    /// 
    /// No tags have content removed by default.
    pub fn clean_content_tags(&mut self, value: HashSet<&'a str>) -> &mut Self {
        self.clean_content_tags = value;
        self
    }

    /// Add additonal blacklisted clean-content tags without overwriting old ones.
    ///
    /// Does nothing if the tag is already there.
    /// 
    /// Adding tags which are whitelisted in `tags` or `tag_attributes` will cause
    /// a panic.
    ///
    /// # Examples
    ///
    ///     let a = ammonia::Builder::default()
    ///         .add_clean_content_tags(std::iter::once("my-tag"))
    ///         .clean("<my-tag>test</my-tag><span>mess</span>").to_string();
    ///     assert_eq!("<span>mess</span>", a);
    pub fn add_clean_content_tags<I: Iterator<Item=&'a str>>(&mut self, it: I) -> &mut Self {
        self.clean_content_tags.extend(it);
        self
    }

    /// Remove already-blacklisted clean-content tags.
    ///
    /// Does nothing if the tags aren't blacklisted.
    ///
    /// # Examples
    ///     #[macro_use]
    ///     extern crate maplit;
    ///     # extern crate ammonia;
    ///
    ///     use ammonia::Builder;
    ///
    ///     # fn main() {
    ///     let tag_blacklist = hashset!["script"];
    ///     let a = ammonia::Builder::default()
    ///         .clean_content_tags(tag_blacklist)
    ///         .rm_clean_content_tags(std::iter::once("script"))
    ///         .clean("<script>XSS</script>").to_string();
    ///     assert_eq!("XSS", a);
    ///     # }
    pub fn rm_clean_content_tags<'b, I: Iterator<Item=&'b str>>(&mut self, it: I) -> &mut Self {
        for i in it {
            self.clean_content_tags.remove(i);
        }
        self
    }

    /// Returns a copy of the set of blacklisted clean-content tags.
    ///
    /// # Examples
    ///
    ///     let tags = ["my-tag-1", "my-tag-2"].into_iter().cloned().collect();
    ///     let mut b = ammonia::Builder::default();
    ///     b.clean_content_tags(Clone::clone(&tags));
    ///     assert_eq!(tags, b.clone_clean_content_tags());
    pub fn clone_clean_content_tags(&self) -> HashSet<&'a str> {
        self.clean_content_tags.clone()
    }

    /// Sets the HTML attributes that are allowed on specific tags.
    ///
    /// The value is structured as a map from tag names to a set of attribute names.
    ///
    /// If a tag is not itself whitelisted, adding entries to this map will do nothing.
    ///
    /// # Examples
    ///
    ///     #[macro_use]
    ///     extern crate maplit;
    ///     # extern crate ammonia;
    ///
    ///     use ammonia::Builder;
    ///
    ///     # fn main() {
    ///     let tags = hashset!["my-tag"];
    ///     let tag_attributes = hashmap![
    ///         "my-tag" => hashset!["val"]
    ///     ];
    ///     let a = Builder::new().tags(tags).tag_attributes(tag_attributes)
    ///         .clean("<my-tag val=1>")
    ///         .to_string();
    ///     assert_eq!(a, "<my-tag val=\"1\"></my-tag>");
    ///     # }
    ///
    /// # Defaults
    ///
    /// ```notest
    /// a =>
    ///     href, hreflang
    /// bdo =>
    ///     dir
    /// blockquote =>
    ///     cite
    /// col =>
    ///     align, char, charoff, span
    /// colgroup =>
    ///     align, char, charoff, span
    /// del =>
    ///     cite, datetime
    /// hr =>
    ///     align, size, width
    /// img =>
    ///     align, alt, height, src, width
    /// ins =>
    ///     cite, datetime
    /// ol =>
    ///     start
    /// q =>
    ///     cite
    /// table =>
    ///     align, char, charoff, summary
    /// tbody =>
    ///     align, char, charoff
    /// td =>
    ///     align, char, charoff, colspan, headers, rowspan
    /// tfoot =>
    ///     align, char, charoff
    /// th =>
    ///     align, char, charoff, colspan, headers, rowspan, scope
    /// thead =>
    ///     align, char, charoff
    /// tr =>
    ///     align, char, charoff
    /// ```
    pub fn tag_attributes(&mut self, value: HashMap<&'a str, HashSet<&'a str>>) -> &mut Self {
        self.tag_attributes = value;
        self
    }

    /// Add additonal whitelisted tag-specific attributes without overwriting old ones.
    ///
    /// # Examples
    ///
    ///     let a = ammonia::Builder::default()
    ///         .add_tags(std::iter::once("my-tag"))
    ///         .add_tag_attributes("my-tag", std::iter::once("my-attr"))
    ///         .clean("<my-tag my-attr>test</my-tag> <span>mess</span>").to_string();
    ///     assert_eq!("<my-tag my-attr=\"\">test</my-tag> <span>mess</span>", a);
    pub fn add_tag_attributes<I: Iterator<Item=&'a str>>(&mut self, tag: &'a str, it: I) -> &mut Self {
        self.tag_attributes.entry(tag).or_insert_with(|| HashSet::new()).extend(it);
        self
    }

    /// Remove already-whitelisted tag-specific attributes.
    ///
    /// Does nothing if the attribute is already gone.
    ///
    /// # Examples
    ///
    ///     let a = ammonia::Builder::default()
    ///         .rm_tag_attributes("a", std::iter::once("href"))
    ///         .clean("<a href=\"/\"></a>").to_string();
    ///     assert_eq!("<a rel=\"noopener noreferrer\"></a>", a);
    pub fn rm_tag_attributes<'b, 'c, I: Iterator<Item=&'b str>>(&mut self, tag: &'c str, it: I) -> &mut Self {
        if let Some(tag) = self.tag_attributes.get_mut(tag) {
            for i in it {
                tag.remove(i);
            }
        }
        self
    }

    /// Returns a copy of the set of whitelisted tag-specific attributes.
    ///
    /// # Examples
    ///
    ///     let tag_attributes = std::iter::once(
    ///         ("my-tag", ["my-attr-1", "my-attr-2"].into_iter().cloned().collect())
    ///     ).collect();
    ///     let mut b = ammonia::Builder::default();
    ///     b.tag_attributes(Clone::clone(&tag_attributes));
    ///     assert_eq!(tag_attributes, b.clone_tag_attributes());
    pub fn clone_tag_attributes(&self) -> HashMap<&'a str, HashSet<&'a str>> {
        self.tag_attributes.clone()
    }

    /// Sets the attributes that are allowed on any tag.
    ///
    /// # Examples
    ///
    ///     #[macro_use]
    ///     extern crate maplit;
    ///     # extern crate ammonia;
    ///
    ///     use ammonia::Builder;
    ///
    ///     # fn main() {
    ///     let attributes = hashset!["data-val"];
    ///     let a = Builder::new()
    ///         .generic_attributes(attributes)
    ///         .clean("<b data-val=1>")
    ///         .to_string();
    ///     assert_eq!(a, "<b data-val=\"1\"></b>");
    ///     # }
    ///
    /// # Defaults
    ///
    /// ```notest
    /// lang, title
    /// ```
    pub fn generic_attributes(&mut self, value: HashSet<&'a str>) -> &mut Self {
        self.generic_attributes = value;
        self
    }

    /// Add additonal whitelisted attributes without overwriting old ones.
    ///
    /// # Examples
    ///
    ///     let a = ammonia::Builder::default()
    ///         .add_generic_attributes(std::iter::once("my-attr"))
    ///         .clean("<span my-attr>mess</span>").to_string();
    ///     assert_eq!("<span my-attr=\"\">mess</span>", a);
    pub fn add_generic_attributes<I: Iterator<Item=&'a str>>(&mut self, it: I) -> &mut Self {
        self.generic_attributes.extend(it);
        self
    }

    /// Remove already-whitelisted attributes.
    ///
    /// Does nothing if the attribute is already gone.
    ///
    /// # Examples
    ///
    ///     let a = ammonia::Builder::default()
    ///         .rm_generic_attributes(std::iter::once("title"))
    ///         .clean("<span title=\"cool\"></span>").to_string();
    ///     assert_eq!("<span></span>", a);
    pub fn rm_generic_attributes<'b, I: Iterator<Item=&'b str>>(&mut self, it: I) -> &mut Self {
        for i in it {
            self.generic_attributes.remove(i);
        }
        self
    }

    /// Returns a copy of the set of whitelisted attributes.
    ///
    /// # Examples
    ///
    ///     let generic_attributes = ["my-attr-1", "my-attr-2"].into_iter().cloned().collect();
    ///     let mut b = ammonia::Builder::default();
    ///     b.generic_attributes(Clone::clone(&generic_attributes));
    ///     assert_eq!(generic_attributes, b.clone_generic_attributes());
    pub fn clone_generic_attributes(&self) -> HashSet<&'a str> {
        self.generic_attributes.clone()
    }

    /// Sets the URL schemes permitted on `href` and `src` attributes.
    ///
    /// # Examples
    ///
    ///     #[macro_use]
    ///     extern crate maplit;
    ///     # extern crate ammonia;
    ///
    ///     use ammonia::Builder;
    ///
    ///     # fn main() {
    ///     let url_schemes = hashset![
    ///         "http", "https", "mailto", "magnet"
    ///     ];
    ///     let a = Builder::new().url_schemes(url_schemes)
    ///         .clean("<a href=\"magnet:?xt=urn:ed2k:31D6CFE0D16AE931B73C59D7E0C089C0&xl=0&dn=zero_len.fil&xt=urn:bitprint:3I42H3S6NNFQ2MSVX7XZKYAYSCX5QBYJ.LWPNACQDBZRYXW3VHJVCJ64QBZNGHOHHHZWCLNQ&xt=urn:md5:D41D8CD98F00B204E9800998ECF8427E\">zero-length file</a>")
    ///         .to_string();
    ///
    ///     // See `link_rel` for information on the rel="noopener noreferrer" attribute
    ///     // in the cleaned HTML.
    ///     assert_eq!(a,
    ///       "<a href=\"magnet:?xt=urn:ed2k:31D6CFE0D16AE931B73C59D7E0C089C0&amp;xl=0&amp;dn=zero_len.fil&amp;xt=urn:bitprint:3I42H3S6NNFQ2MSVX7XZKYAYSCX5QBYJ.LWPNACQDBZRYXW3VHJVCJ64QBZNGHOHHHZWCLNQ&amp;xt=urn:md5:D41D8CD98F00B204E9800998ECF8427E\" rel=\"noopener noreferrer\">zero-length file</a>");
    ///     # }
    ///
    /// # Defaults
    ///
    /// ```notest
    /// bitcoin, ftp, ftps, geo, http, https, im, irc,
    /// ircs, magnet, mailto, mms, mx, news, nntp,
    /// openpgp4fpr, sip, sms, smsto, ssh, tel, url,
    /// webcal, wtai, xmpp
    /// ```
    pub fn url_schemes(&mut self, value: HashSet<&'a str>) -> &mut Self {
        self.url_schemes = value;
        self
    }

    /// Add additonal whitelisted URL schemes without overwriting old ones.
    ///
    /// # Examples
    ///
    ///     let a = ammonia::Builder::default()
    ///         .add_url_schemes(std::iter::once("my-scheme"))
    ///         .clean("<a href=my-scheme:home>mess</span>").to_string();
    ///     assert_eq!("<a href=\"my-scheme:home\" rel=\"noopener noreferrer\">mess</a>", a);
    pub fn add_url_schemes<I: Iterator<Item=&'a str>>(&mut self, it: I) -> &mut Self {
        self.url_schemes.extend(it);
        self
    }

    /// Remove already-whitelisted attributes.
    ///
    /// Does nothing if the attribute is already gone.
    ///
    /// # Examples
    ///
    ///     let a = ammonia::Builder::default()
    ///         .rm_url_schemes(std::iter::once("ftp"))
    ///         .clean("<a href=\"ftp://ftp.mozilla.org/\"></a>").to_string();
    ///     assert_eq!("<a rel=\"noopener noreferrer\"></a>", a);
    pub fn rm_url_schemes<'b, I: Iterator<Item=&'b str>>(&mut self, it: I) -> &mut Self {
        for i in it {
            self.url_schemes.remove(i);
        }
        self
    }

    /// Returns a copy of the set of whitelisted URL schemes.
    ///
    /// # Examples
    ///
    ///     let url_schemes = ["my-scheme-1", "my-scheme-2"].into_iter().cloned().collect();
    ///     let mut b = ammonia::Builder::default();
    ///     b.url_schemes(Clone::clone(&url_schemes));
    ///     assert_eq!(url_schemes, b.clone_url_schemes());
    pub fn clone_url_schemes(&self) -> HashSet<&'a str> {
        self.url_schemes.clone()
    }

    /// Configures the behavior for relative URLs: pass-through, resolve-with-base, or deny.
    ///
    /// # Examples
    ///
    ///     use ammonia::{Builder, UrlRelative};
    ///
    ///     let a = Builder::new().url_relative(UrlRelative::PassThrough)
    ///         .clean("<a href=/>Home</a>")
    ///         .to_string();
    ///
    ///     // See `link_rel` for information on the rel="noopener noreferrer" attribute
    ///     // in the cleaned HTML.
    ///     assert_eq!(
    ///       a,
    ///       "<a href=\"/\" rel=\"noopener noreferrer\">Home</a>");
    ///
    /// # Defaults
    ///
    /// ```notest
    /// UrlRelative::PassThrough
    /// ```
    pub fn url_relative(&mut self, value: UrlRelative) -> &mut Self {
        self.url_relative = value;
        self
    }

    /// Returns `true` if the relative URL resolver is set to `Deny`.
    ///
    /// # Examples
    ///
    ///     use ammonia::{Builder, UrlRelative};
    ///     let mut a = Builder::default();
    ///     a.url_relative(UrlRelative::Deny);
    ///     assert!(a.is_url_relative_deny());
    ///     a.url_relative(UrlRelative::PassThrough);
    ///     assert!(!a.is_url_relative_deny());
    pub fn is_url_relative_deny(&self) -> bool {
        matches!(self.url_relative, UrlRelative::Deny)
    }

    /// Returns `true` if the relative URL resolver is set to `PassThrough`.
    ///
    /// # Examples
    ///
    ///     use ammonia::{Builder, UrlRelative};
    ///     let mut a = Builder::default();
    ///     a.url_relative(UrlRelative::Deny);
    ///     assert!(!a.is_url_relative_pass_through());
    ///     a.url_relative(UrlRelative::PassThrough);
    ///     assert!(a.is_url_relative_pass_through());
    pub fn is_url_relative_pass_through(&self) -> bool {
        matches!(self.url_relative, UrlRelative::PassThrough)
    }

    /// Returns `true` if the relative URL resolver is set to `Custom`.
    ///
    /// # Examples
    ///
    ///     # extern crate ammonia;
    ///     use ammonia::{Builder, UrlRelative};
    ///     use std::borrow::Cow;
    ///     fn test(a: &str) -> Option<Cow<str>> { None }
    ///     # fn main() {
    ///     let mut a = Builder::default();
    ///     a.url_relative(UrlRelative::Custom(Box::new(test)));
    ///     assert!(a.is_url_relative_custom());
    ///     a.url_relative(UrlRelative::PassThrough);
    ///     assert!(!a.is_url_relative_custom());
    ///     a.url_relative(UrlRelative::Deny);
    ///     assert!(!a.is_url_relative_custom());
    ///     # }
    pub fn is_url_relative_custom(&self) -> bool {
        matches!(self.url_relative, UrlRelative::Custom(_))
    }

    /// Configures a `rel` attribute that will be added on links.
    ///
    /// If `rel` is in the generic or tag attributes, this must be set to `None`.
    /// Common `rel` values to include:
    ///
    /// * `noopener`: This prevents [a particular type of XSS attack],
    ///   and should usually be turned on for untrusted HTML.
    /// * `noreferrer`: This prevents the browser from [sending the source URL]
    ///   to the website that is linked to.
    /// * `nofollow`: This prevents search engines from [using this link for
    ///   ranking], which disincentivizes spammers.
    ///
    /// To turn on rel-insertion, call this function with a space-separated list.
    /// Ammonia does not parse rel-attributes;
    /// it just puts the given string into the attribute directly.
    ///
    /// [a particular type of XSS attack]: https://mathiasbynens.github.io/rel-noopener/
    /// [sending the source URL]: https://en.wikipedia.org/wiki/HTTP_referer
    /// [using this link for ranking]: https://en.wikipedia.org/wiki/Nofollow
    ///
    /// # Examples
    ///
    ///     use ammonia::Builder;
    ///
    ///     let a = Builder::new().link_rel(None)
    ///         .clean("<a href=https://rust-lang.org/>Rust</a>")
    ///         .to_string();
    ///     assert_eq!(
    ///       a,
    ///       "<a href=\"https://rust-lang.org/\">Rust</a>");
    ///
    /// # Defaults
    ///
    /// ```notest
    /// Some("noopener noreferrer")
    /// ```
    pub fn link_rel(&mut self, value: Option<&'a str>) -> &mut Self {
        self.link_rel = value;
        self
    }

    /// Returns the settings for links' `rel` attribute, if one is set.
    ///
    /// # Examples
    ///
    ///     use ammonia::{Builder, UrlRelative};
    ///     let mut a = Builder::default();
    ///     a.link_rel(Some("a b"));
    ///     assert_eq!(a.get_link_rel(), Some("a b"));
    pub fn get_link_rel(&self) -> Option<&str> {
        self.link_rel.clone()
    }

    /// Sets the CSS classes that are allowed on specific tags.
    ///
    /// The values is structured as a map from tag names to a set of class names.
    ///
    /// If the `class` attribute is itself whitelisted for a tag, then adding entries to
    /// this map will cause a panic.
    ///
    /// # Examples
    ///
    ///     #[macro_use]
    ///     extern crate maplit;
    ///     # extern crate ammonia;
    ///
    ///     use ammonia::Builder;
    ///
    ///     # fn main() {
    ///     let allowed_classes = hashmap![
    ///         "code" => hashset!["rs", "ex", "c", "cxx", "js"]
    ///     ];
    ///     let a = Builder::new()
    ///         .allowed_classes(allowed_classes)
    ///         .clean("<code class=rs>fn main() {}</code>")
    ///         .to_string();
    ///     assert_eq!(
    ///       a,
    ///       "<code class=\"rs\">fn main() {}</code>");
    ///     # }
    ///
    /// # Defaults
    ///
    /// The set of allowed classes is empty by default.
    pub fn allowed_classes(&mut self, value: HashMap<&'a str, HashSet<&'a str>>) -> &mut Self {
        self.allowed_classes = value;
        self
    }

    /// Add additonal whitelisted classes without overwriting old ones.
    ///
    /// # Examples
    ///
    ///     let a = ammonia::Builder::default()
    ///         .add_allowed_classes("a", std::iter::once("onebox"))
    ///         .clean("<a href=/ class=onebox>mess</span>").to_string();
    ///     assert_eq!("<a href=\"/\" class=\"onebox\" rel=\"noopener noreferrer\">mess</a>", a);
    pub fn add_allowed_classes<I: Iterator<Item=&'a str>>(&mut self, tag: &'a str, it: I) -> &mut Self {
        self.allowed_classes.entry(tag).or_insert_with(|| HashSet::new()).extend(it);
        self
    }

    /// Remove already-whitelisted attributes.
    ///
    /// Does nothing if the attribute is already gone.
    ///
    /// # Examples
    ///
    ///     let a = ammonia::Builder::default()
    ///         .add_allowed_classes("span", std::iter::once("active"))
    ///         .rm_allowed_classes("span", std::iter::once("active"))
    ///         .clean("<span class=active>").to_string();
    ///     assert_eq!("<span class=\"\"></span>", a);
    pub fn rm_allowed_classes<'b, 'c, I: Iterator<Item=&'b str>>(&mut self, tag: &'c str, it: I) -> &mut Self {
        if let Some(tag) = self.allowed_classes.get_mut(tag) {
            for i in it {
                tag.remove(i);
            }
        }
        self
    }

    /// Returns a copy of the set of whitelisted class attributes.
    ///
    /// # Examples
    ///
    ///     let allowed_classes = std::iter::once(
    ///         ("my-tag", ["my-class-1", "my-class-2"].into_iter().cloned().collect())
    ///     ).collect();
    ///     let mut b = ammonia::Builder::default();
    ///     b.allowed_classes(Clone::clone(&allowed_classes));
    ///     assert_eq!(allowed_classes, b.clone_allowed_classes());
    pub fn clone_allowed_classes(&self) -> HashMap<&'a str, HashSet<&'a str>> {
        self.allowed_classes.clone()
    }

    /// Configures the handling of HTML comments.
    ///
    /// If this option is false, comments will be preserved.
    ///
    /// # Examples
    ///
    ///     use ammonia::Builder;
    ///
    ///     let a = Builder::new().strip_comments(false)
    ///         .clean("<!-- yes -->")
    ///         .to_string();
    ///     assert_eq!(
    ///       a,
    ///       "<!-- yes -->");
    ///
    /// # Defaults
    ///
    /// `true`
    pub fn strip_comments(&mut self, value: bool) -> &mut Self {
        self.strip_comments = value;
        self
    }

    /// Returns `true` if comment stripping is turned on.
    ///
    /// # Examples
    ///
    ///     let mut a = ammonia::Builder::new();
    ///     a.strip_comments(true);
    ///     assert!(a.will_strip_comments());
    ///     a.strip_comments(false);
    ///     assert!(!a.will_strip_comments());
    pub fn will_strip_comments(&self) -> bool {
        self.strip_comments
    }

    /// Prefixes all "id" attribute values with a given string.  Note that the tag and
    /// attribute themselves must still be whitelisted.
    ///
    /// # Examples
    ///
    ///     #[macro_use]
    ///     extern crate maplit;
    ///     # extern crate ammonia;
    ///
    ///     use ammonia::Builder;
    ///
    ///     # fn main() {
    ///     let attributes = hashset!["id"];
    ///     let a = Builder::new()
    ///         .generic_attributes(attributes)
    ///         .id_prefix(Some("safe-"))
    ///         .clean("<b id=42>")
    ///         .to_string();
    ///     assert_eq!(a, "<b id=\"safe-42\"></b>");
    ///     # }

    ///
    /// # Defaults
    ///
    /// `None`
    pub fn id_prefix(&mut self, value: Option<&'a str>) -> &mut Self {
        self.id_prefix = value;
        self
    }

    /// Constructs a [`Builder`] instance configured with the [default options].
    ///
    /// # Examples
    ///
    ///     # extern crate ammonia;
    ///     use ammonia::{Builder, Url, UrlRelative};
    ///     # use std::error::Error;
    ///
    ///     # fn do_main() -> Result<(), Box<Error>> {
    ///     let input = "<!-- comments will be stripped -->This is an <a href=.>Ammonia</a> example using <a href=struct.Builder.html#method.new onclick=xss>the <code onmouseover=xss>new()</code> function</a>.";
    ///     let output = "This is an <a href=\"https://docs.rs/ammonia/1.0/ammonia/\" rel=\"noopener noreferrer\">Ammonia</a> example using <a href=\"https://docs.rs/ammonia/1.0/ammonia/struct.Builder.html#method.new\" rel=\"noopener noreferrer\">the <code>new()</code> function</a>.";
    ///
    ///     let result = Builder::new() // <--
    ///         .url_relative(UrlRelative::RewriteWithBase(Url::parse("https://docs.rs/ammonia/1.0/ammonia/")?))
    ///         .clean(input)
    ///         .to_string();
    ///     assert_eq!(result, output);
    ///     # Ok(())
    ///     # }
    ///     # fn main() { do_main().unwrap() }
    ///
    /// [default options]: fn.clean.html
    /// [`Builder`]: struct.Builder.html
    pub fn new() -> Self {
        Self::default()
    }

    /// Sanitizes an HTML fragment in a string according to the configured options.
    ///
    /// # Examples
    ///
    ///     # extern crate ammonia;
    ///     use ammonia::{Builder, Url, UrlRelative};
    ///     # use std::error::Error;
    ///
    ///     # fn do_main() -> Result<(), Box<Error>> {
    ///     let input = "<!-- comments will be stripped -->This is an <a href=.>Ammonia</a> example using <a href=struct.Builder.html#method.new onclick=xss>the <code onmouseover=xss>new()</code> function</a>.";
    ///     let output = "This is an <a href=\"https://docs.rs/ammonia/1.0/ammonia/\" rel=\"noopener noreferrer\">Ammonia</a> example using <a href=\"https://docs.rs/ammonia/1.0/ammonia/struct.Builder.html#method.new\" rel=\"noopener noreferrer\">the <code>new()</code> function</a>.";
    ///
    ///     let result = Builder::new()
    ///         .url_relative(UrlRelative::RewriteWithBase(Url::parse("https://docs.rs/ammonia/1.0/ammonia/")?))
    ///         .clean(input)
    ///         .to_string(); // <--
    ///     assert_eq!(result, output);
    ///     # Ok(())
    ///     # }
    ///     # fn main() { do_main().unwrap() }
    pub fn clean(&self, src: &str) -> Document {
        let parser = Self::make_parser();
        let dom = parser.one(src);
        self.clean_dom(dom)
    }

    /// Sanitizes an HTML fragment from a reader according to the configured options.
    ///
    /// The input should be in UTF-8 encoding, otherwise the decoding is lossy, just
    /// like when using [`String::from_utf8_lossy`].
    ///
    /// To avoid consuming the reader, a mutable reference can be passed to this method.
    ///
    /// # Examples
    ///
    ///     # extern crate ammonia;
    ///     use ammonia::Builder;
    ///     # use std::error::Error;
    ///
    ///     # fn do_main() -> Result<(), Box<Error>> {
    ///     let a = Builder::new()
    ///         .clean_from_reader(&b"<!-- no -->"[..])? // notice the `b`
    ///         .to_string();
    ///     assert_eq!(a, "");
    ///     # Ok(()) }
    ///     # fn main() { do_main().unwrap() }
    ///
    /// [`String::from_utf8_lossy`]: https://doc.rust-lang.org/std/string/struct.String.html#method.from_utf8_lossy
    pub fn clean_from_reader<R>(&self, mut src: R) -> io::Result<Document>
    where
        R: io::Read,
    {
        let parser = Self::make_parser().from_utf8();
        let dom = parser.read_from(&mut src)?;
        Ok(self.clean_dom(dom))
    }

    /// Clean a post-parsing DOM.
    ///
    /// This is not a public API because RcDom isn't really stable.
    /// We want to be able to take breaking changes to html5ever itself
    /// without having to break Ammonia's API.
    fn clean_dom(&self, mut dom: RcDom) -> Document {
        let mut stack = Vec::new();
        let link_rel = self.link_rel
            .map(|link_rel| format_tendril!("{}", link_rel));
        if link_rel.is_some() {
            assert!(self.generic_attributes.get("rel").is_none());
            assert!(
                self.tag_attributes
                    .get("a")
                    .and_then(|a| a.get("rel"))
                    .is_none()
            );
        }
        assert!(self.allowed_classes.is_empty() || !self.generic_attributes.contains("class"));
        for (tag_name, _classes) in &self.allowed_classes {
            assert!(
                self.tag_attributes
                    .get(tag_name)
                    .and_then(|a| a.get("class"))
                    .is_none()
            );
        }
        for tag_name in &self.clean_content_tags {
            assert!(!self.tags.contains(tag_name));
            assert!(!self.tag_attributes.contains_key(tag_name));
        }
        let url_base = if let UrlRelative::RewriteWithBase(ref base) = self.url_relative {
            Some(base)
        } else {
            None
        };
        let body = {
            let children = dom.document.children.borrow();
            children[0].clone()
        };
        stack.extend(
            replace(&mut *body.children.borrow_mut(), Vec::new())
                .into_iter()
                .rev(),
        );
        while let Some(mut node) = stack.pop() {
            let parent = node.parent
                .replace(None).expect("a node in the DOM will have a parent, except the root, which is not processed")
                .upgrade().expect("a node's parent will be pointed to by its parent (or the root pointer), and will not be dropped");
            if self.clean_node_content(&node) {
                continue;
            }
            let pass = self.clean_child(&mut node);
            if pass {
                self.adjust_node_attributes(&mut node, &link_rel, url_base, self.id_prefix);
                dom.append(&parent.clone(), NodeOrText::AppendNode(node.clone()));
            } else {
                for sub in node.children.borrow_mut().iter_mut() {
                    sub.parent.replace(Some(Rc::downgrade(&parent)));
                }
            }
            stack.extend(
                replace(&mut *node.children.borrow_mut(), Vec::new())
                    .into_iter()
                    .rev(),
            );
        }
        Document(body)
    }

    /// Returns `true` if a node and all its content should be removed.
    fn clean_node_content(&self, node: &Handle) -> bool {
        match node.data {
            NodeData::Text { .. } |
            NodeData::Comment { .. } |
            NodeData::Doctype { .. } |
            NodeData::Document |
            NodeData::ProcessingInstruction { .. } => false,
            NodeData::Element { ref name, .. } => self.clean_content_tags.contains(&*name.local)
        }
    }

    /// Remove unwanted attributes, and check if the node should be kept or not.
    ///
    /// The root node doesn't need cleaning because we create the root node ourselves,
    /// and it doesn't get serialized, and ... it just exists to give the parser
    /// a context (in this case, a div-like block context).
    fn clean_child(&self, child: &mut Handle) -> bool {
        match child.data {
            NodeData::Text { .. } => true,
            NodeData::Comment { .. } => !self.strip_comments,
            NodeData::Doctype { .. } |
            NodeData::Document |
            NodeData::ProcessingInstruction { .. } => false,
            NodeData::Element {
                ref name,
                ref attrs,
                ..
            } => if self.tags.contains(&*name.local) {
                let attr_filter = |attr: &html5ever::Attribute| {
                    let whitelisted = self.generic_attributes.contains(&*attr.name.local) ||
                        self.tag_attributes
                            .get(&*name.local)
                            .map(|ta| ta.contains(&*attr.name.local)) ==
                            Some(true);
                    if !whitelisted {
                        // If the class attribute is not whitelisted,
                        // but there is a whitelisted set of allowed_classes,
                        // do not strip out the class attribute.
                        // Banned classes will be filtered later.
                        &*attr.name.local == "class" &&
                          self.allowed_classes.contains_key(&*name.local)
                    } else if is_url_attr(&*name.local, &*attr.name.local) {
                        let url = Url::parse(&*attr.value);
                        if let Ok(url) = url {
                            self.url_schemes.contains(url.scheme())
                        } else if url == Err(url::ParseError::RelativeUrlWithoutBase) {
                            !matches!(self.url_relative, UrlRelative::Deny)
                        } else {
                            false
                        }
                    } else {
                        true
                    }
                };
                attrs.borrow_mut().retain(attr_filter);
                true
            } else {
                false
            },
        }
    }

    /// Add and transform special-cased attributes and elements.
    ///
    /// This function handles:
    ///
    /// * relative URL rewriting
    /// * adding `<a rel>` attributes
    /// * filtering out banned classes
    fn adjust_node_attributes(
        &self,
        child: &mut Handle,
        link_rel: &Option<StrTendril>,
        url_base: Option<&Url>,
        id_prefix: Option<&'a str>,
    ) {
        if let NodeData::Element {
            ref name,
            ref attrs,
            ..
        } = child.data
        {
            if let Some(ref link_rel) = *link_rel {
                if &*name.local == "a" {
                    attrs.borrow_mut().push(Attribute {
                        name: QualName::new(None, ns!(), local_name!("rel")),
                        value: link_rel.clone(),
                    })
                }
            }
            if let Some(ref id_prefix) = id_prefix {
                for attr in &mut *attrs.borrow_mut() {
                    if &attr.name.local == "id" {
                        if !attr.value.starts_with(id_prefix) {
                            attr.value = format_tendril!("{}{}", id_prefix, attr.value);
                        }
                    }
                }
            }
            if let Some(ref base) = url_base {
                for attr in &mut *attrs.borrow_mut() {
                    if is_url_attr(&*name.local, &*attr.name.local) {
                        let url = base.join(&*attr.value)
                            .expect("invalid URLs should be stripped earlier");
                        attr.value = format_tendril!("{}", url);
                    }
                }
            } else if let UrlRelative::Custom(ref evaluate) = self.url_relative {
                let mut drop_attrs = Vec::new();
                let mut attrs = attrs.borrow_mut();
                for (i, attr) in attrs.iter_mut().enumerate() {
                    if is_url_attr(&*name.local, &*attr.name.local) && is_url_relative(&*attr.value) {
                        let new_value = evaluate.evaluate(&*attr.value)
                            .as_ref()
                            .map(Cow::as_ref)
                            .map(StrTendril::from_str)
                            .and_then(Result::ok);
                        if let Some(new_value) = new_value {
                            attr.value = new_value;
                        } else {
                            drop_attrs.push(i);
                        }
                    }
                }
                // Swap remove scrambles the vector after the current point.
                // We will not do anything except with items before the current point.
                // The `rev()` is, as such, necessary for correctness.
                // We could use regular `remove(usize)` and a forward iterator,
                // but that's slower.
                for i in drop_attrs.into_iter().rev() {
                    attrs.swap_remove(i);
                }
            }
            if let Some(allowed_values) = self.allowed_classes.get(&*name.local) {
                for attr in &mut *attrs.borrow_mut() {
                    if &attr.name.local == "class" {
                        let mut classes = vec![];
                        for class in attr.value.split(' ') {
                            if allowed_values.contains(class) {
                                classes.push(class.to_owned());
                            }
                        }
                        attr.value = format_tendril!("{}", classes.join(" "));
                    }
                }
            }
        }
    }

    /// Initializes an HTML fragment parser.
    ///
    /// Ammonia conforms to the HTML5 fragment parsing rules,
    /// by parsing the given fragment as if it were included in a <div> tag.
    fn make_parser() -> html::Parser<RcDom> {
        html::parse_fragment(
            RcDom::default(),
            html::ParseOpts::default(),
            QualName::new(None, ns!(html), local_name!("div")),
            vec![],
        )
    }
}

/// Given an element name and attribute name, determine if the given attribute contains a URL.
fn is_url_attr(element: &str, attr: &str) -> bool {
    attr == "href" || attr == "src" || (element == "object" && attr == "data")
}

fn is_url_relative(url: &str) -> bool {
    matches!(Url::parse(url), Err(url::ParseError::RelativeUrlWithoutBase))
}

/// Policy for [relative URLs], that is, URLs that do not specify the scheme in full.
///
/// This policy kicks in, if set, for any attribute named `src` or `href`,
/// as well as the `data` attribute of an `object` tag.
///
/// [relative URLs]: struct.Builder.html#method.url_relative
///
/// # Examples
///
/// ## `Deny`
///
/// * `<a href="test">` is a file-relative URL, and will be removed
/// * `<a href="/test">` is a domain-relative URL, and will be removed
/// * `<a href="//example.com/test">` is a scheme-relative URL, and will be removed
/// * `<a href="http://example.com/test">` is an absolute URL, and will be kept
///
/// ## `PassThrough`
///
/// No changes will be made to any URLs, except if a disallowed scheme is used.
///
/// ## `RewriteWithBase`
///
/// If the base is set to `http://notriddle.com/some-directory/some-file`
///
/// * `<a href="test">` will be rewritten to `<a href="http://notriddle.com/some-directory/test">`
/// * `<a href="/test">` will be rewritten to `<a href="http://notriddle.com/test">`
/// * `<a href="//example.com/test">` will be rewritten to `<a href="http://example.com/test">`
/// * `<a href="http://example.com/test">` is an absolute URL, so it will be kept as-is
///
/// ## `Custom`
///
/// Pass the relative URL to a function.
/// If it returns `Some(string)`, then that one gets used.
/// Otherwise, it will remove the attribute (like `Deny` does).
///
///     # extern crate ammonia;
///     use std::borrow::Cow;
///     fn is_absolute_path(url: &str) -> bool {
///         let u = url.as_bytes();
///         // `//a/b/c` is "protocol-relative", meaning "a" is a hostname
///         // `/a/b/c` is an absolute path, and what we want to do stuff to.
///         u.get(0) == Some(&b'/') && u.get(1) != Some(&b'/')
///     }
///     fn evaluate(url: &str) -> Option<Cow<str>> {
///         if is_absolute_path(url) {
///             Some(Cow::Owned(String::from("/root") + url))
///         } else {
///             Some(Cow::Borrowed(url))
///         }
///     }
///     fn main() {
///         let a = ammonia::Builder::new()
///             .url_relative(ammonia::UrlRelative::Custom(Box::new(evaluate)))
///             .clean("<a href=/test/path>fixed</a><a href=path>passed</a><a href=http://google.com/>skipped</a>")
///             .to_string();
///         assert_eq!(a, "<a href=\"/root/test/path\" rel=\"noopener noreferrer\">fixed</a><a href=\"path\" rel=\"noopener noreferrer\">passed</a><a href=\"http://google.com/\" rel=\"noopener noreferrer\">skipped</a>");
///     }
///
/// This function is only applied to relative URLs.
/// To filter all of the URLs,
/// use the not-yet-implemented Content Security Policy.
pub enum UrlRelative {
    /// Relative URLs will be completely stripped from the document.
    Deny,
    /// Relative URLs will be passed through unchanged.
    PassThrough,
    /// Relative URLs will be changed into absolute URLs, based on this base URL.
    RewriteWithBase(Url),
    /// Rewrite URLs with a custom function.
    Custom(Box<UrlRelativeEvaluate>),
    // Do not allow the user to exhaustively match on UrlRelative,
    // because we may add new items to it later.
    #[doc(hidden)]
    __NonExhaustive,
}

impl fmt::Debug for UrlRelative {
    fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        match *self {
            UrlRelative::Deny => write!(f, "UrlRelative::Deny"),
            UrlRelative::PassThrough => write!(f, "UrlRelative::PassThrough"),
            UrlRelative::RewriteWithBase(ref base) => write!(f, "UrlRelative::RewriteWithBase({})", base),
            UrlRelative::Custom(_) => write!(f, "UrlRelative::Custom"),
            UrlRelative::__NonExhaustive => unreachable!(),
        }
    }
}

pub trait UrlRelativeEvaluate: Send + Sync {
    fn evaluate<'a>(&self, &'a str) -> Option<Cow<'a, str>>;
}
impl<T> UrlRelativeEvaluate for T where T: Fn(&str) -> Option<Cow<str>> + Send + Sync {
    fn evaluate<'a>(&self, url: &'a str) -> Option<Cow<'a, str>> {
        self(url)
    }
}

/// A sanitized HTML document.
///
/// The `Document` type is an opaque struct representing an HTML fragment that was sanitized by
/// `ammonia`. It can be converted to a [`String`] or written to a [`Write`] instance. This allows
/// users to avoid buffering the serialized representation to a [`String`] when desired.
///
/// This type is opaque to insulate the caller from breaking changes in the `html5ever` interface.
///
/// Note that this type wraps an `html5ever` DOM tree. `ammonia` does not support streaming, so
/// the complete fragment needs to be stored in memory during processing. Currently, `Document`
/// is backed by an [`html5ever::rcdom::Node`] object.
///
/// [`String`]: https://doc.rust-lang.org/nightly/std/string/struct.String.html
/// [`Write`]: https://doc.rust-lang.org/nightly/std/io/trait.Write.html
/// [`html5ever::rcdom::Node`]: ../markup5ever/rcdom/struct.Node.html
///
/// # Examples
///
///     use ammonia::Builder;
///
///     let input = "<!-- comments will be stripped -->This is an Ammonia example.";
///     let output = "This is an Ammonia example.";
///
///     let document = Builder::new()
///         .clean(input);
///     assert_eq!(document.to_string(), output);
#[derive(Clone)]
pub struct Document(Handle);

impl Document {
    /// Serializes a `Document` instance to a `String`.
    ///
    /// This method returns a [`String`] with the sanitized HTML. This is the simplest way to use
    /// `ammonia`.
    ///
    /// [`String`]: https://doc.rust-lang.org/nightly/std/string/struct.String.html
    ///
    /// # Examples
    ///
    ///     use ammonia::Builder;
    ///
    ///     let input = "Some <style>HTML here";
    ///     let output = "Some HTML here";
    ///
    ///     let document = Builder::new()
    ///         .clean(input);
    ///     assert_eq!(document.to_string(), output);
    pub fn to_string(&self) -> String {
        let opts = Self::serialize_opts();
        let mut ret_val = Vec::new();
        serialize(&mut ret_val, &self.0, opts)
            .expect("Writing to a string shouldn't fail (expect on OOM)");
        String::from_utf8(ret_val)
            .expect("html5ever only supports UTF8")
    }

    /// Serializes a `Document` instance to a writer.
    ///
    /// This method writes the sanitized HTML to a [`Write`] instance, avoiding a buffering step.
    ///
    /// To avoid consuming the writer, a mutable reference can be passed, like in the example below.
    ///
    /// Note that the in-memory representation of `Document` is larger than the serialized
    /// `String`.
    ///
    /// [`Write`]: https://doc.rust-lang.org/nightly/std/io/trait.Write.html
    ///
    /// # Examples
    ///
    ///     use ammonia::Builder;
    ///
    ///     let input = "Some <style>HTML here";
    ///     let expected = b"Some HTML here";
    ///
    ///     let document = Builder::new()
    ///         .clean(input);
    ///
    ///     let mut sanitized = Vec::new();
    ///     document.write_to(&mut sanitized)
    ///         .expect("Writing to a string should not fail (except on OOM)");
    ///     assert_eq!(sanitized, expected);
    pub fn write_to<W>(&self, writer: W) -> io::Result<()>
    where
        W: io::Write,
    {
        let opts = Self::serialize_opts();
        serialize(writer, &self.0, opts)
    }

    /// Exposes the `Document` instance as an [`html5ever::rcdom::Handle`][h].
    ///
    /// This method returns the inner object backing the `Document` instance. This allows
    /// making further changes to the DOM without introducing redundant serialization and
    /// parsing.
    ///
    /// Note that this method should be considered unstable and sits outside of the semver
    /// stability guarantees. It may change, break, or go away at any time, either because
    /// of `html5ever` changes or `ammonia` implementation changes.
    ///
    /// For this method to be accessible, a `cfg` flag is required. The easiest way is to
    /// use the `RUSTFLAGS` environment variable:
    ///
    /// [h]: ../markup5ever/rcdom/type.Handle.html
    ///
    /// ```text
    /// RUSTFLAGS='--cfg ammonia_unstable' cargo build
    /// ```
    ///
    /// on Unix-like platforms, or
    ///
    /// ```text
    /// set RUSTFLAGS=--cfg ammonia_unstable
    /// cargo build
    /// ```
    ///
    /// on Windows.
    ///
    /// This requirement also applies to crates that transitively depend on crates that use
    /// this flag.
    ///
    /// # Examples
    ///
    ///     # extern crate ammonia;
    ///     extern crate html5ever;
    ///
    ///     use ammonia::Builder;
    ///     use html5ever::serialize::{serialize, SerializeOpts};
    ///
    ///     # use std::error::Error;
    ///     # fn do_main() -> Result<(), Box<Error>> {
    ///     let input = "<a>one link</a> and <a>one more</a>";
    ///     let expected = "<a>one more</a> and <a>one link</a>";
    ///
    ///     let document = Builder::new()
    ///         .link_rel(None)
    ///         .clean(input);
    ///
    ///     let mut node = document.to_dom_node();
    ///     node.children.borrow_mut().reverse();
    ///
    ///     let mut buf = Vec::new();
    ///     serialize(&mut buf, &node, SerializeOpts::default())?;
    ///     let output = String::from_utf8(buf)?;
    ///
    ///     assert_eq!(output, expected);
    ///     # Ok(())
    ///     # }
    ///     # fn main() { do_main().unwrap() }
    #[cfg(ammonia_unstable)]
    pub fn to_dom_node(&self) -> Handle {
        self.0.clone()
    }

    fn serialize_opts() -> SerializeOpts {
        SerializeOpts::default()
    }
}

impl fmt::Debug for Document {
    fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        write!(f, "Document({})", self.to_string())
    }
}

impl From<Document> for String {
    fn from(document: Document) -> Self {
        document.to_string()
    }
}

#[cfg(test)]
mod test {
    use super::*;
    #[test]
    fn included_angles() {
        let fragment = "1 < 2";
        let result = clean(fragment);
        assert_eq!(result, "1 &lt; 2");
    }
    #[test]
    fn remove_script() {
        let fragment = "an <script>evil()</script> example";
        let result = clean(fragment);
        assert_eq!(result, "an evil() example");
    }
    #[test]
    fn ignore_link() {
        let fragment = "a <a href=\"http://www.google.com\">good</a> example";
        let expected = "a <a href=\"http://www.google.com\" rel=\"noopener noreferrer\">\
                        good</a> example";
        let result = clean(fragment);
        assert_eq!(result, expected);
    }
    #[test]
    fn remove_unsafe_link() {
        let fragment = "an <a onclick=\"evil()\" href=\"http://www.google.com\">evil</a> example";
        let result = clean(fragment);
        assert_eq!(
            result,
            "an <a href=\"http://www.google.com\" rel=\"noopener noreferrer\">evil</a> example"
        );
    }
    #[test]
    fn remove_js_link() {
        let fragment = "an <a href=\"javascript:evil()\">evil</a> example";
        let result = clean(fragment);
        assert_eq!(result, "an <a rel=\"noopener noreferrer\">evil</a> example");
    }
    #[test]
    fn tag_rebalance() {
        let fragment = "<b>AWESOME!";
        let result = clean(fragment);
        assert_eq!(result, "<b>AWESOME!</b>");
    }
    #[test]
    fn allow_url_relative() {
        let fragment = "<a href=test>Test</a>";
        let result = Builder::new()
            .url_relative(UrlRelative::PassThrough)
            .clean(fragment)
            .to_string();
        assert_eq!(
            result,
            "<a href=\"test\" rel=\"noopener noreferrer\">Test</a>"
        );
    }
    #[test]
    fn rewrite_url_relative() {
        let fragment = "<a href=test>Test</a>";
        let result = Builder::new()
            .url_relative(UrlRelative::RewriteWithBase(Url::parse("http://example.com/").unwrap()))
            .clean(fragment)
            .to_string();
        assert_eq!(
            result,
            "<a href=\"http://example.com/test\" rel=\"noopener noreferrer\">Test</a>"
        );
    }
    #[test]
    fn rewrite_url_relative_no_rel() {
        let fragment = "<a href=test>Test</a>";
        let result = Builder::new()
            .url_relative(UrlRelative::RewriteWithBase(Url::parse("http://example.com/").unwrap()))
            .link_rel(None)
            .clean(fragment)
            .to_string();
        assert_eq!(result, "<a href=\"http://example.com/test\">Test</a>");
    }
    #[test]
    fn deny_url_relative() {
        let fragment = "<a href=test>Test</a>";
        let result = Builder::new()
            .url_relative(UrlRelative::Deny)
            .clean(fragment)
            .to_string();
        assert_eq!(result, "<a rel=\"noopener noreferrer\">Test</a>");
    }
    #[test]
    fn replace_rel() {
        let fragment = "<a href=test rel=\"garbage\">Test</a>";
        let result = Builder::new()
            .url_relative(UrlRelative::PassThrough)
            .clean(fragment)
            .to_string();
        assert_eq!(
            result,
            "<a href=\"test\" rel=\"noopener noreferrer\">Test</a>"
        );
    }
    #[test]
    fn consider_rel_still_banned() {
        let fragment = "<a href=test rel=\"garbage\">Test</a>";
        let result = Builder::new()
            .url_relative(UrlRelative::PassThrough)
            .link_rel(None)
            .clean(fragment)
            .to_string();
        assert_eq!(result, "<a href=\"test\">Test</a>");
    }
    #[test]
    fn object_data() {
        let fragment = "<span data=\"javascript:evil()\">Test</span>\
                        <object data=\"javascript:evil()\"></object>M";
        let expected = r#"<span data="javascript:evil()">Test</span><object></object>M"#;
        let result = Builder::new()
            .tags(hashset!["span", "object"])
            .generic_attributes(hashset!["data"])
            .clean(fragment)
            .to_string();
        assert_eq!(result, expected);
    }
    #[test]
    fn remove_attributes() {
        let fragment = "<table border=\"1\"><tr></tr></table>";
        let result = Builder::new().clean(fragment);
        assert_eq!(
            result.to_string(),
            "<table><tbody><tr></tr></tbody></table>"
        );
    }
    #[test]
    fn quotes_in_attrs() {
        let fragment = "<b title='\"'>contents</b>";
        let result = clean(fragment);
        assert_eq!(result, "<b title=\"&quot;\">contents</b>");
    }
    #[test]
    #[should_panic]
    fn panic_if_rel_is_allowed_and_replaced_generic() {
        Builder::new()
            .link_rel(Some("noopener noreferrer"))
            .generic_attributes(hashset!["rel"])
            .clean("something");
    }
    #[test]
    #[should_panic]
    fn panic_if_rel_is_allowed_and_replaced_a() {
        Builder::new()
            .link_rel(Some("noopener noreferrer"))
            .tag_attributes(hashmap![
                "a" => hashset!["rel"],
            ])
            .clean("something");
    }
    #[test]
    fn no_panic_if_rel_is_allowed_and_replaced_span() {
        Builder::new()
            .link_rel(Some("noopener noreferrer"))
            .tag_attributes(hashmap![
                "span" => hashset!["rel"],
            ])
            .clean("<span rel=\"what\">s</span>");
    }
    #[test]
    fn no_panic_if_rel_is_allowed_and_not_replaced_generic() {
        Builder::new()
            .link_rel(None)
            .generic_attributes(hashset!["rel"])
            .clean("<a rel=\"what\">s</a>");
    }
    #[test]
    fn no_panic_if_rel_is_allowed_and_not_replaced_a() {
        Builder::new()
            .link_rel(None)
            .tag_attributes(hashmap![
                "a" => hashset!["rel"],
            ])
            .clean("<a rel=\"what\">s</a>");
    }
    #[test]
    fn dont_close_void_elements() {
        let fragment = "<br>";
        let result = clean(fragment);
        assert_eq!(result.to_string(), "<br>");
    }
    #[should_panic]
    #[test]
    fn panic_on_allowed_classes_tag_attributes() {
        let fragment = "<p class=\"foo bar\"><a class=\"baz bleh\">Hey</a></p>";
        Builder::new()
            .link_rel(None)
            .tag_attributes(hashmap![
                "p" => hashset!["class"],
                "a" => hashset!["class"],
            ])
            .allowed_classes(hashmap![
                "p" => hashset!["foo", "bar"],
                "a" => hashset!["baz"],
            ])
            .clean(fragment);
    }
    #[should_panic]
    #[test]
    fn panic_on_allowed_classes_generic_attributes() {
        let fragment = "<p class=\"foo bar\"><a class=\"baz bleh\">Hey</a></p>";
        Builder::new()
            .link_rel(None)
            .generic_attributes(hashset!["class", "href", "some-foo"])
            .allowed_classes(hashmap![
                "p" => hashset!["foo", "bar"],
                "a" => hashset!["baz"],
            ])
            .clean(fragment);
    }
    #[test]
    fn remove_non_allowed_classes() {
        let fragment = "<p class=\"foo bar\"><a class=\"baz bleh\">Hey</a></p>";
        let result = Builder::new()
            .link_rel(None)
            .allowed_classes(hashmap![
                "p" => hashset!["foo", "bar"],
                "a" => hashset!["baz"],
            ])
            .clean(fragment);
        assert_eq!(
            result.to_string(),
            "<p class=\"foo bar\"><a class=\"baz\">Hey</a></p>"
        );
    }
    #[test]
    fn remove_non_allowed_classes_with_tag_class() {
        let fragment = "<p class=\"foo bar\"><a class=\"baz bleh\">Hey</a></p>";
        let result = Builder::new()
            .link_rel(None)
            .tag_attributes(hashmap![
                "div" => hashset!["class"],
            ])
            .allowed_classes(hashmap![
                "p" => hashset!["foo", "bar"],
                "a" => hashset!["baz"],
            ])
            .clean(fragment);
        assert_eq!(
            result.to_string(),
            "<p class=\"foo bar\"><a class=\"baz\">Hey</a></p>"
        );
    }
    #[test]
    fn remove_entity_link() {
        let fragment = "<a href=\"&#x6A&#x61&#x76&#x61&#x73&#x63&#x72&#x69&#x70&#x74&#x3A&#x61\
                        &#x6C&#x65&#x72&#x74&#x28&#x27&#x58&#x53&#x53&#x27&#x29\">Click me!</a>";
        let result = clean(fragment);
        assert_eq!(
            result.to_string(),
            "<a rel=\"noopener noreferrer\">Click me!</a>"
        );
    }
    #[test]
    fn remove_relative_url_evaluate() {
        fn is_absolute_path(url: &str) -> bool {
            let u = url.as_bytes();
            // `//a/b/c` is "protocol-relative", meaning "a" is a hostname
            // `/a/b/c` is an absolute path, and what we want to do stuff to.
            u.get(0) == Some(&b'/') && u.get(1) != Some(&b'/')
        }
        fn is_banned(url: &str) -> bool {
            let u = url.as_bytes();
            u.get(0) == Some(&b'b') && u.get(1) == Some(&b'a')
        }
        fn evaluate(url: &str) -> Option<Cow<str>> {
            if is_absolute_path(url) {
                Some(Cow::Owned(String::from("/root") + url))
            } else if is_banned(url) {
                None
            } else {
                Some(Cow::Borrowed(url))
            }
        }
        let a = Builder::new()
            .url_relative(UrlRelative::Custom(Box::new(evaluate)))
            .clean("<a href=banned>banned</a><a href=/test/path>fixed</a><a href=path>passed</a><a href=http://google.com/>skipped</a>")
            .to_string();
        assert_eq!(a, "<a rel=\"noopener noreferrer\">banned</a><a href=\"/root/test/path\" rel=\"noopener noreferrer\">fixed</a><a href=\"path\" rel=\"noopener noreferrer\">passed</a><a href=\"http://google.com/\" rel=\"noopener noreferrer\">skipped</a>");
    }
    #[test]
    fn remove_relative_url_evaluate_b() {
        fn is_absolute_path(url: &str) -> bool {
            let u = url.as_bytes();
            // `//a/b/c` is "protocol-relative", meaning "a" is a hostname
            // `/a/b/c` is an absolute path, and what we want to do stuff to.
            u.get(0) == Some(&b'/') && u.get(1) != Some(&b'/')
        }
        fn is_banned(url: &str) -> bool {
            let u = url.as_bytes();
            u.get(0) == Some(&b'b') && u.get(1) == Some(&b'a')
        }
        fn evaluate(url: &str) -> Option<Cow<str>> {
            if is_absolute_path(url) {
                Some(Cow::Owned(String::from("/root") + url))
            } else if is_banned(url) {
                None
            } else {
                Some(Cow::Borrowed(url))
            }
        }
        let a = Builder::new()
            .url_relative(UrlRelative::Custom(Box::new(evaluate)))
            .clean("<a href=banned>banned</a><a href=banned title=test>banned</a><a title=test href=banned>banned</a>")
            .to_string();
        assert_eq!(a, "<a rel=\"noopener noreferrer\">banned</a><a rel=\"noopener noreferrer\" title=\"test\">banned</a><a title=\"test\" rel=\"noopener noreferrer\">banned</a>");
    }
    #[test]
    fn remove_relative_url_evaluate_c() {
        // Don't run on absolute URLs.
        fn evaluate(_: &str) -> Option<Cow<str>> {
            return Some(Cow::Owned(String::from("invalid")));
        }
        let a = Builder::new()
            .url_relative(UrlRelative::Custom(Box::new(evaluate)))
            .clean("<a href=\"https://www.google.com/\">google</a>")
            .to_string();
        assert_eq!(a, "<a href=\"https://www.google.com/\" rel=\"noopener noreferrer\">google</a>");
    }
    #[test]
    fn clean_children_of_bad_element() {
        let fragment = "<bad><evil>a</evil>b</bad>";
        let result = Builder::new().clean(fragment);
        assert_eq!(result.to_string(), "ab");
    }
    #[test]
    fn reader_input() {
        let fragment = b"an <script>evil()</script> example";
        let result = Builder::new().clean_from_reader(&fragment[..]);
        assert!(result.is_ok());
        assert_eq!(result.unwrap().to_string(), "an evil() example");
    }
    #[test]
    fn reader_non_utf8() {
        let fragment = b"non-utf8 \xF0\x90\x80string";
        let result = Builder::new().clean_from_reader(&fragment[..]);
        assert!(result.is_ok());
        assert_eq!(result.unwrap().to_string(), "non-utf8 \u{fffd}string");
    }
    #[test]
    fn debug_impl() {
        let fragment = r#"a <a>link</a>"#;
        let result = Builder::new().link_rel(None).clean(fragment);
        assert_eq!(format!("{:?}", result), "Document(a <a>link</a>)");
    }
    #[cfg(ammonia_unstable)]
    #[test]
    fn to_dom_node() {
        let fragment = r#"a <a>link</a>"#;
        let result = Builder::new().link_rel(None).clean(fragment);
        let _node = result.to_dom_node();
    }
    #[test]
    fn string_from_document() {
        let fragment = r#"a <a>link"#;
        let result = String::from(Builder::new().link_rel(None).clean(fragment));
        assert_eq!(format!("{}", result), "a <a>link</a>");
    }
    fn require_sync<T: Sync>(_: T) {}
    fn require_send<T: Send>(_: T) {}
    #[test]
    fn require_sync_and_send() {
        require_sync(Builder::new());
        require_send(Builder::new());
    }
    #[test]
    fn id_prefixed() {
        let fragment = "<a id=\"hello\"></a><b id=\"hello\"></a>";
        let result = String::from(Builder::new().tag_attributes(hashmap![
            "a" => hashset!["id"],
        ]).id_prefix(Some("prefix-")).clean(fragment));
        assert_eq!(result.to_string(), "<a id=\"prefix-hello\" rel=\"noopener noreferrer\"></a><b></b>");
    }
    #[test]
    fn id_already_prefixed() {
        let fragment = "<a id=\"prefix-hello\"></a>";
        let result = String::from(Builder::new().tag_attributes(hashmap![
            "a" => hashset!["id"],
        ]).id_prefix(Some("prefix-")).clean(fragment));
        assert_eq!(result.to_string(), "<a id=\"prefix-hello\" rel=\"noopener noreferrer\"></a>");
    }
    #[test]
    fn clean_content_tags() {
        let fragment = "<script type=\"text/javascript\"><a>Hello!</a></script>";
        let result = String::from(Builder::new()
            .clean_content_tags(hashset!["script"])
            .clean(fragment));
        assert_eq!(result.to_string(), "");
    }
    #[test]
    fn only_clean_content_tags() {
        let fragment = "<em>This is</em><script><a>Hello!</a></script><p>still here!</p>";
        let result = String::from(Builder::new()
            .clean_content_tags(hashset!["script"])
            .clean(fragment));
        assert_eq!(result.to_string(), "<em>This is</em><p>still here!</p>");
    }
    #[test]
    fn clean_removed_default_tag() {
        let fragment = "<em>This is</em><script><a>Hello!</a></script><p>still here!</p>";
        let result = String::from(Builder::new()
            .rm_tags(["a"].into_iter().cloned())
            .rm_tag_attributes("a", ["href", "hreflang"].into_iter().cloned())
            .clean_content_tags(hashset!["script"])
            .clean(fragment));
        assert_eq!(result.to_string(), "<em>This is</em><p>still here!</p>");
    }
    #[test]
    #[should_panic]
    fn panic_on_clean_content_tag_attribute() {
        Builder::new()
            .rm_tags(std::iter::once("a"))
            .clean_content_tags(hashset!["a"])
            .clean("");
    }
    #[test]
    #[should_panic]
    fn panic_on_clean_content_tag() {
        Builder::new()
            .clean_content_tags(hashset!["a"])
            .clean("");
    }
}
