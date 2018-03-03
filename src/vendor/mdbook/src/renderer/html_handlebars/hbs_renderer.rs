use renderer::html_handlebars::helpers;
use preprocess;
use renderer::Renderer;
use book::MDBook;
use book::bookitem::{BookItem, Chapter};
use config::PlaypenConfig;
use {utils, theme};
use theme::{Theme, playpen_editor};
use errors::*;
use regex::{Regex, Captures};

use std::ascii::AsciiExt;
use std::path::{Path, PathBuf};
use std::fs::{self, File};
use std::io::{self, Read};
use std::collections::BTreeMap;
use std::collections::HashMap;

use handlebars::Handlebars;

use serde_json;

#[derive(Default)]
pub struct HtmlHandlebars;

impl HtmlHandlebars {
    pub fn new() -> Self {
        HtmlHandlebars
    }

    fn render_item(&self, item: &BookItem, mut ctx: RenderItemContext, print_content: &mut String)
        -> Result<()> {
        // FIXME: This should be made DRY-er and rely less on mutable state
        match *item {
            BookItem::Chapter(_, ref ch) |
            BookItem::Affix(ref ch) if !ch.path.as_os_str().is_empty() => {

                let path = ctx.book.get_source().join(&ch.path);
                let content = utils::fs::file_to_string(&path)?;
                let base = path.parent().ok_or_else(
                    || String::from("Invalid bookitem path!"),
                )?;

                // Parse and expand links
                let content = preprocess::links::replace_all(&content, base)?;
                let content = utils::render_markdown(&content, ctx.book.get_curly_quotes());
                print_content.push_str(&content);

                // Update the context with data for this file
                let path = ch.path.to_str().ok_or_else(|| {
                    io::Error::new(io::ErrorKind::Other, "Could not convert path to str")
                })?;

                // Non-lexical lifetimes needed :'( 
                let title: String;
                {
                    let book_title = ctx.data.get("book_title").and_then(serde_json::Value::as_str).unwrap_or("");
                    title = ch.name.clone() + " - " + book_title;
                }
                
                ctx.data.insert("path".to_owned(), json!(path));
                ctx.data.insert("content".to_owned(), json!(content));
                ctx.data.insert("chapter_title".to_owned(), json!(ch.name));
                ctx.data.insert("title".to_owned(), json!(title));
                ctx.data.insert(
                    "path_to_root".to_owned(),
                    json!(utils::fs::path_to_root(&ch.path)),
                );

                // Render the handlebars template with the data
                debug!("[*]: Render template");
                let rendered = ctx.handlebars.render("index", &ctx.data)?;

                let filepath = Path::new(&ch.path).with_extension("html");
                let rendered = self.post_process(rendered,
                    &normalize_path(filepath.to_str()
                        .ok_or(Error::from(format!("Bad file name: {}", filepath.display())))?),
                    ctx.book.get_html_config().get_playpen_config()
                );

                // Write to file
                info!("[*] Creating {:?} ✓", filepath.display());
                ctx.book.write_file(filepath, &rendered.into_bytes())?;

                if ctx.is_index {
                    self.render_index(ctx.book, ch, &ctx.destination)?;
                }
            },
            _ => {},
        }

        Ok(())
    }

    /// Create an index.html from the first element in SUMMARY.md
    fn render_index(&self, book: &MDBook, ch: &Chapter, destination: &Path) -> Result<()> {
        debug!("[*]: index.html");

        let mut content = String::new();

        File::open(destination.join(&ch.path.with_extension("html")))?
            .read_to_string(&mut content)?;

        // This could cause a problem when someone displays
        // code containing <base href=...>
        // on the front page, however this case should be very very rare...
        content = content
            .lines()
            .filter(|line| !line.contains("<base href="))
            .collect::<Vec<&str>>()
            .join("\n");

        book.write_file("index.html", content.as_bytes())?;

        info!(
            "[*] Creating index.html from {:?} ✓",
            book.get_destination()
                .join(&ch.path.with_extension("html"))
        );

        Ok(())
    }

    fn post_process(&self, rendered: String, filepath: &str, playpen_config: &PlaypenConfig) -> String {
        let rendered = build_header_links(&rendered, &filepath);
        let rendered = fix_anchor_links(&rendered, &filepath);
        let rendered = fix_code_blocks(&rendered);
        let rendered = add_playpen_pre(&rendered, playpen_config);

        rendered
    }

    fn copy_static_files(&self, book: &MDBook, theme: &Theme) -> Result<()> {
        book.write_file("book.js", &theme.js)?;
        book.write_file("book.css", &theme.css)?;
        book.write_file("favicon.png", &theme.favicon)?;
        book.write_file("jquery.js", &theme.jquery)?;
        book.write_file("highlight.css", &theme.highlight_css)?;
        book.write_file(
            "tomorrow-night.css",
            &theme.tomorrow_night_css,
        )?;
        book.write_file(
            "ayu-highlight.css",
            &theme.ayu_highlight_css,
        )?;
        book.write_file("highlight.js", &theme.highlight_js)?;
        book.write_file("clipboard.min.js", &theme.clipboard_js)?;
        book.write_file("store.js", &theme.store_js)?;
        book.write_file(
            "_FontAwesome/css/font-awesome.css",
            theme::FONT_AWESOME,
        )?;
        book.write_file(
            "_FontAwesome/fonts/fontawesome-webfont.eot",
            theme::FONT_AWESOME_EOT,
        )?;
        book.write_file(
            "_FontAwesome/fonts/fontawesome-webfont.svg",
            theme::FONT_AWESOME_SVG,
        )?;
        book.write_file(
            "_FontAwesome/fonts/fontawesome-webfont.ttf",
            theme::FONT_AWESOME_TTF,
        )?;
        book.write_file(
            "_FontAwesome/fonts/fontawesome-webfont.woff",
            theme::FONT_AWESOME_WOFF,
        )?;
        book.write_file(
            "_FontAwesome/fonts/fontawesome-webfont.woff2",
            theme::FONT_AWESOME_WOFF2,
        )?;
        book.write_file(
            "_FontAwesome/fonts/FontAwesome.ttf",
            theme::FONT_AWESOME_TTF,
        )?;

        let playpen_config = book.get_html_config().get_playpen_config();

        // Ace is a very large dependency, so only load it when requested
        if playpen_config.is_editable() {
            // Load the editor
            let editor = playpen_editor::PlaypenEditor::new(playpen_config.get_editor());
            book.write_file("editor.js", &editor.js)?;
            book.write_file("ace.js", &editor.ace_js)?;
            book.write_file("mode-rust.js", &editor.mode_rust_js)?;
            book.write_file("theme-dawn.js", &editor.theme_dawn_js)?;
            book.write_file("theme-tomorrow_night.js", &editor.theme_tomorrow_night_js)?;
        }

        Ok(())
    }

    /// Helper function to write a file to the build directory, normalizing
    /// the path to be relative to the book root.
    fn write_custom_file(&self, custom_file: &Path, book: &MDBook) -> Result<()> {
        let mut data = Vec::new();
        let mut f = File::open(custom_file)?;
        f.read_to_end(&mut data)?;

        let name = match custom_file.strip_prefix(book.get_root()) {
            Ok(p) => p.to_str().expect("Could not convert to str"),
            Err(_) => {
                custom_file
                    .file_name()
                    .expect("File has a file name")
                    .to_str()
                    .expect("Could not convert to str")
            },
        };

        book.write_file(name, &data)?;

        Ok(())
    }

    /// Update the context with data for this file
    fn configure_print_version(&self, data: &mut serde_json::Map<String, serde_json::Value>, print_content: &str) {
        // Make sure that the Print chapter does not display the title from
        // the last rendered chapter by removing it from its context
        data.remove("title");
        data.insert("is_print".to_owned(), json!(true));
        data.insert("path".to_owned(), json!("print.md"));
        data.insert("content".to_owned(), json!(print_content));
        data.insert("path_to_root".to_owned(), json!(utils::fs::path_to_root(Path::new("print.md"))));
    }

    fn register_hbs_helpers(&self, handlebars: &mut Handlebars) {
        handlebars.register_helper("toc", Box::new(helpers::toc::RenderToc));
        handlebars.register_helper("previous", Box::new(helpers::navigation::previous));
        handlebars.register_helper("next", Box::new(helpers::navigation::next));
    }

    /// Copy across any additional CSS and JavaScript files which the book
    /// has been configured to use.
    fn copy_additional_css_and_js(&self, book: &MDBook) -> Result<()> {
        let custom_files = book.get_additional_css().iter().chain(
            book.get_additional_js()
                .iter(),
        );

        for custom_file in custom_files {
            self.write_custom_file(custom_file, book)?;
        }

        Ok(())
    }
}


impl Renderer for HtmlHandlebars {
    fn render(&self, book: &MDBook) -> Result<()> {
        debug!("[fn]: render");
        let mut handlebars = Handlebars::new();

        let theme = theme::Theme::new(book.get_theme_path());

        debug!("[*]: Register handlebars template");
        handlebars.register_template_string(
            "index",
            String::from_utf8(theme.index.clone())?,
        )?;

        debug!("[*]: Register handlebars helpers");
        self.register_hbs_helpers(&mut handlebars);

        let mut data = make_data(book)?;

        // Print version
        let mut print_content = String::new();

        let destination = book.get_destination();

        debug!("[*]: Check if destination directory exists");
        if fs::create_dir_all(&destination).is_err() {
            bail!("Unexpected error when constructing destination path");
        }

        for (i, item) in book.iter().enumerate() {
            let ctx = RenderItemContext {
                book: book,
                handlebars: &handlebars,
                destination: destination.to_path_buf(),
                data: data.clone(),
                is_index: i == 0,
            };
            self.render_item(item, ctx, &mut print_content)?;
        }

        // Print version
        self.configure_print_version(&mut data, &print_content);
        data.insert("title".to_owned(), json!(book.get_title()));

        // Render the handlebars template with the data
        debug!("[*]: Render template");

        let rendered = handlebars.render("index", &data)?;

        let rendered = self.post_process(rendered, "print.html",
            book.get_html_config().get_playpen_config());

        book.write_file(
            Path::new("print").with_extension("html"),
            &rendered.into_bytes(),
        )?;
        info!("[*] Creating print.html ✓");

        // Copy static files (js, css, images, ...)
        debug!("[*] Copy static files");
        self.copy_static_files(book, &theme)?;
        self.copy_additional_css_and_js(book)?;

        // Copy all remaining files
        utils::fs::copy_files_except_ext(book.get_source(), destination, true, &["md"])?;

        Ok(())
    }
}

fn make_data(book: &MDBook) -> Result<serde_json::Map<String, serde_json::Value>> {
    debug!("[fn]: make_data");

    let mut data = serde_json::Map::new();
    data.insert("language".to_owned(), json!("en"));
    data.insert("book_title".to_owned(), json!(book.get_title()));
    data.insert("description".to_owned(), json!(book.get_description()));
    data.insert("favicon".to_owned(), json!("favicon.png"));
    if let Some(livereload) = book.get_livereload() {
        data.insert("livereload".to_owned(), json!(livereload));
    }

    // Add google analytics tag
    if let Some(ref ga) = book.get_google_analytics_id() {
        data.insert("google_analytics".to_owned(), json!(ga));
    }

    if book.get_mathjax_support() {
        data.insert("mathjax_support".to_owned(), json!(true));
    }

    // Add check to see if there is an additional style
    if book.has_additional_css() {
        let mut css = Vec::new();
        for style in book.get_additional_css() {
            match style.strip_prefix(book.get_root()) {
                Ok(p) => css.push(p.to_str().expect("Could not convert to str")),
                Err(_) => {
                    css.push(
                        style
                            .file_name()
                            .expect("File has a file name")
                            .to_str()
                            .expect("Could not convert to str"),
                    )
                },
            }
        }
        data.insert("additional_css".to_owned(), json!(css));
    }

    // Add check to see if there is an additional script
    if book.has_additional_js() {
        let mut js = Vec::new();
        for script in book.get_additional_js() {
            match script.strip_prefix(book.get_root()) {
                Ok(p) => js.push(p.to_str().expect("Could not convert to str")),
                Err(_) => {
                    js.push(
                        script
                            .file_name()
                            .expect("File has a file name")
                            .to_str()
                            .expect("Could not convert to str"),
                    )
                },
            }
        }
        data.insert("additional_js".to_owned(), json!(js));
    }

    if book.get_html_config().get_playpen_config().is_editable() {
        data.insert("playpens_editable".to_owned(), json!(true));
        data.insert("editor_js".to_owned(), json!("editor.js"));
        data.insert("ace_js".to_owned(), json!("ace.js"));
        data.insert("mode_rust_js".to_owned(), json!("mode-rust.js"));
        data.insert("theme_dawn_js".to_owned(), json!("theme-dawn.js"));
        data.insert("theme_tomorrow_night_js".to_owned(), json!("theme-tomorrow_night.js"));
    }

    let mut chapters = vec![];

    for item in book.iter() {
        // Create the data to inject in the template
        let mut chapter = BTreeMap::new();

        match *item {
            BookItem::Affix(ref ch) => {
                chapter.insert("name".to_owned(), json!(ch.name));
                let path = ch.path.to_str().ok_or_else(|| {
                    io::Error::new(io::ErrorKind::Other, "Could not convert path to str")
                })?;
                chapter.insert("path".to_owned(), json!(path));
            },
            BookItem::Chapter(ref s, ref ch) => {
                chapter.insert("section".to_owned(), json!(s));
                chapter.insert("name".to_owned(), json!(ch.name));
                let path = ch.path.to_str().ok_or_else(|| {
                    io::Error::new(io::ErrorKind::Other, "Could not convert path to str")
                })?;
                chapter.insert("path".to_owned(), json!(path));
            },
            BookItem::Spacer => {
                chapter.insert("spacer".to_owned(), json!("_spacer_"));
            },

        }

        chapters.push(chapter);
    }

    data.insert("chapters".to_owned(), json!(chapters));

    debug!("[*]: JSON constructed");
    Ok(data)
}

/// Goes through the rendered HTML, making sure all header tags are wrapped in
/// an anchor so people can link to sections directly.
fn build_header_links(html: &str, filepath: &str) -> String {
    let regex = Regex::new(r"<h(\d)>(.*?)</h\d>").unwrap();
    let mut id_counter = HashMap::new();

    regex
        .replace_all(html, |caps: &Captures| {
            let level = caps[1].parse().expect(
                "Regex should ensure we only ever get numbers here",
            );

            wrap_header_with_link(level, &caps[2], &mut id_counter, filepath)
        })
        .into_owned()
}

/// Wraps a single header tag with a link, making sure each tag gets its own
/// unique ID by appending an auto-incremented number (if necessary).
fn wrap_header_with_link(level: usize, content: &str, id_counter: &mut HashMap<String, usize>, filepath: &str)
    -> String {
    let raw_id = id_from_content(content);

    let id_count = id_counter.entry(raw_id.clone()).or_insert(0);

    let id = match *id_count {
        0 => raw_id,
        other => format!("{}-{}", raw_id, other),
    };

    *id_count += 1;

    format!(
        r##"<a class="header" href="{filepath}#{id}" id="{id}"><h{level}>{text}</h{level}></a>"##,
        level = level,
        id = id,
        text = content,
        filepath = filepath
    )
}

/// Generate an id for use with anchors which is derived from a "normalised" 
/// string.
fn id_from_content(content: &str) -> String {
    let mut content = content.to_string();

    // Skip any tags or html-encoded stuff
    const REPL_SUB: &[&str] = &[
        "<em>",
        "</em>",
        "<code>",
        "</code>",
        "<strong>",
        "</strong>",
        "&lt;",
        "&gt;",
        "&amp;",
        "&#39;",
        "&quot;",
    ];
    for sub in REPL_SUB {
        content = content.replace(sub, "");
    }

    // Remove spaces and hastags indicating a header
    let trimmed = content.trim().trim_left_matches("#").trim();

    normalize_id(trimmed)
}

// anchors to the same page (href="#anchor") do not work because of
// <base href="../"> pointing to the root folder. This function *fixes*
// that in a very inelegant way
fn fix_anchor_links(html: &str, filepath: &str) -> String {
    let regex = Regex::new(r##"<a([^>]+)href="#([^"]+)"([^>]*)>"##).unwrap();
    regex
        .replace_all(html, |caps: &Captures| {
            let before = &caps[1];
            let anchor = &caps[2];
            let after = &caps[3];

            format!(
                "<a{before}href=\"{filepath}#{anchor}\"{after}>",
                before = before,
                filepath = filepath,
                anchor = anchor,
                after = after
            )
        })
        .into_owned()
}


// The rust book uses annotations for rustdoc to test code snippets,
// like the following:
// ```rust,should_panic
// fn main() {
//     // Code here
// }
// ```
// This function replaces all commas by spaces in the code block classes
fn fix_code_blocks(html: &str) -> String {
    let regex = Regex::new(r##"<code([^>]+)class="([^"]+)"([^>]*)>"##).unwrap();
    regex
        .replace_all(html, |caps: &Captures| {
            let before = &caps[1];
            let classes = &caps[2].replace(",", " ");
            let after = &caps[3];

            format!(r#"<code{before}class="{classes}"{after}>"#, before = before, classes = classes, after = after)
        })
        .into_owned()
}

fn add_playpen_pre(html: &str, playpen_config: &PlaypenConfig) -> String {
    let regex = Regex::new(r##"((?s)<code[^>]?class="([^"]+)".*?>(.*?)</code>)"##).unwrap();
    regex
        .replace_all(html, |caps: &Captures| {
            let text = &caps[1];
            let classes = &caps[2];
            let code = &caps[3];

            if (classes.contains("language-rust") && !classes.contains("ignore")) || classes.contains("mdbook-runnable") {
                // wrap the contents in an external pre block
                if playpen_config.is_editable() &&
                    classes.contains("editable") || text.contains("fn main") || text.contains("quick_main!") {
                    format!("<pre class=\"playpen\">{}</pre>", text)
                } else {
                    // we need to inject our own main
                    let (attrs, code) = partition_source(code);

                    format!("<pre class=\"playpen\"><code class=\"{}\">\n# #![allow(unused_variables)]\n\
                        {}#fn main() {{\n\
                        {}\
                        #}}</code></pre>",
                        classes, attrs, code)
                }
            } else {
                // not language-rust, so no-op
                text.to_owned()
            }
        })
        .into_owned()
}

fn partition_source(s: &str) -> (String, String) {
    let mut after_header = false;
    let mut before = String::new();
    let mut after = String::new();

    for line in s.lines() {
        let trimline = line.trim();
        let header = trimline.chars().all(|c| c.is_whitespace()) || trimline.starts_with("#![");
        if !header || after_header {
            after_header = true;
            after.push_str(line);
            after.push_str("\n");
        } else {
            before.push_str(line);
            before.push_str("\n");
        }
    }

    (before, after)
}


struct RenderItemContext<'a> {
    handlebars: &'a Handlebars,
    book: &'a MDBook,
    destination: PathBuf,
    data: serde_json::Map<String, serde_json::Value>,
    is_index: bool,
}

pub fn normalize_path(path: &str) -> String {
    use std::path::is_separator;
    path.chars()
        .map(|ch| if is_separator(ch) { '/' } else { ch })
        .collect::<String>()
}

pub fn normalize_id(content: &str) -> String {
    content.chars()
        .filter_map(|ch|
            if ch.is_alphanumeric() || ch == '_' || ch == '-' {
                Some(ch.to_ascii_lowercase())
            } else if ch.is_whitespace() {
                Some('-')
            } else {
                None
            }
        )
        .collect::<String>()
}


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn original_build_header_links() {
        let inputs = vec![
            (
                "blah blah <h1>Foo</h1>",
                r##"blah blah <a class="header" href="./some_chapter/some_section.html#foo" id="foo"><h1>Foo</h1></a>"##,
            ),
            (
                "<h1>Foo</h1>",
                r##"<a class="header" href="./some_chapter/some_section.html#foo" id="foo"><h1>Foo</h1></a>"##,
            ),
            (
                "<h3>Foo^bar</h3>",
                r##"<a class="header" href="./some_chapter/some_section.html#foobar" id="foobar"><h3>Foo^bar</h3></a>"##,
            ),
            (
                "<h4></h4>",
                r##"<a class="header" href="./some_chapter/some_section.html#" id=""><h4></h4></a>"##
            ),
            (
                "<h4><em>Hï</em></h4>",
                r##"<a class="header" href="./some_chapter/some_section.html#hï" id="hï"><h4><em>Hï</em></h4></a>"##
            ),
            (
                "<h1>Foo</h1><h3>Foo</h3>",
                r##"<a class="header" href="./some_chapter/some_section.html#foo" id="foo"><h1>Foo</h1></a><a class="header" href="./some_chapter/some_section.html#foo-1" id="foo-1"><h3>Foo</h3></a>"##
            ),
        ];

        for (src, should_be) in inputs {
            let filepath = "./some_chapter/some_section.html";
            let got = build_header_links(&src, filepath);
            assert_eq!(got, should_be);

            // This is redundant for most cases
            let got = fix_anchor_links(&got, filepath);
            assert_eq!(got, should_be);
        }
    }

    #[test]
    fn anchor_generation() {
        assert_eq!(id_from_content("## `--passes`: add more rustdoc passes"), "--passes-add-more-rustdoc-passes");
        assert_eq!(id_from_content("## Method-call expressions"), "method-call-expressions");
    }
}
