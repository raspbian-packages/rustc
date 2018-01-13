// Copyright 2014-2017 The html5ever Project Developers. See the
// COPYRIGHT file at the top-level directory of this distribution.
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

pub use markup5ever::serialize::{Serialize, Serializer, TraversalScope, AttrRef};
use std::io::{self, Write};
use std::default::Default;

use {LocalName, QualName};

pub fn serialize<Wr, T>(writer: Wr, node: &T, opts: SerializeOpts) -> io::Result<()>
where Wr: Write, T: Serialize {
    let mut ser = HtmlSerializer::new(writer, opts);
    node.serialize(&mut ser, opts.traversal_scope)
}

#[derive(Copy, Clone)]
pub struct SerializeOpts {
    /// Is scripting enabled?
    pub scripting_enabled: bool,

    /// Serialize the root node? Default: ChildrenOnly
    pub traversal_scope: TraversalScope,
}

impl Default for SerializeOpts {
    fn default() -> SerializeOpts {
        SerializeOpts {
            scripting_enabled: true,
            traversal_scope: TraversalScope::ChildrenOnly,
        }
    }
}

struct ElemInfo {
    html_name: Option<LocalName>,
    ignore_children: bool,
    processed_first_child: bool,
}

struct HtmlSerializer<Wr: Write> {
    writer: Wr,
    opts: SerializeOpts,
    stack: Vec<ElemInfo>,
}

fn tagname(name: &QualName) -> LocalName {
    match name.ns {
        ns!(html) | ns!(mathml) | ns!(svg) => (),
        ref ns => {
            // FIXME(#122)
            warn!("node with weird namespace {:?}", ns);
        }
    }

    name.local.clone()
}

impl<Wr: Write> HtmlSerializer<Wr> {
    fn new(writer: Wr, opts: SerializeOpts) -> Self {
        HtmlSerializer {
            writer: writer,
            opts: opts,
            stack: vec!(ElemInfo {
                html_name: None,
                ignore_children: false,
                processed_first_child: false,
            }),
        }
    }

    fn parent(&mut self) -> &mut ElemInfo {
        self.stack.last_mut().expect("no parent ElemInfo")
    }

    fn write_escaped(&mut self, text: &str, attr_mode: bool) -> io::Result<()> {
        for c in text.chars() {
            try!(match c {
                '&' => self.writer.write_all(b"&amp;"),
                '\u{00A0}' => self.writer.write_all(b"&nbsp;"),
                '"' if attr_mode => self.writer.write_all(b"&quot;"),
                '<' if !attr_mode => self.writer.write_all(b"&lt;"),
                '>' if !attr_mode => self.writer.write_all(b"&gt;"),
                c => self.writer.write_fmt(format_args!("{}", c)),
            });
        }
        Ok(())
    }
}

impl<Wr: Write> Serializer for HtmlSerializer<Wr> {
    fn start_elem<'a, AttrIter>(&mut self, name: QualName, attrs: AttrIter) -> io::Result<()>
    where AttrIter: Iterator<Item=AttrRef<'a>> {
        let html_name = match name.ns {
            ns!(html) => Some(name.local.clone()),
            _ => None,
        };

        if self.parent().ignore_children {
            self.stack.push(ElemInfo {
                html_name: html_name,
                ignore_children: true,
                processed_first_child: false,
            });
            return Ok(());
        }

        try!(self.writer.write_all(b"<"));
        try!(self.writer.write_all(tagname(&name).as_bytes()));
        for (name, value) in attrs {
            try!(self.writer.write_all(b" "));

            match name.ns {
                ns!() => (),
                ns!(xml) => try!(self.writer.write_all(b"xml:")),
                ns!(xmlns) => {
                    if name.local != local_name!("xmlns") {
                        try!(self.writer.write_all(b"xmlns:"));
                    }
                }
                ns!(xlink) => try!(self.writer.write_all(b"xlink:")),
                ref ns => {
                    // FIXME(#122)
                    warn!("attr with weird namespace {:?}", ns);
                    try!(self.writer.write_all(b"unknown_namespace:"));
                }
            }

            try!(self.writer.write_all(name.local.as_bytes()));
            try!(self.writer.write_all(b"=\""));
            try!(self.write_escaped(value, true));
            try!(self.writer.write_all(b"\""));
        }
        try!(self.writer.write_all(b">"));

        let ignore_children = name.ns == ns!(html) && match name.local {
            local_name!("area") | local_name!("base") | local_name!("basefont") | local_name!("bgsound") | local_name!("br")
            | local_name!("col") | local_name!("embed") | local_name!("frame") | local_name!("hr") | local_name!("img")
            | local_name!("input") | local_name!("keygen") | local_name!("link")
            | local_name!("meta") | local_name!("param") | local_name!("source") | local_name!("track") | local_name!("wbr")
                => true,
            _ => false,
        };

        self.parent().processed_first_child = true;

        self.stack.push(ElemInfo {
            html_name: html_name,
            ignore_children: ignore_children,
            processed_first_child: false,
        });

        Ok(())
    }

    fn end_elem(&mut self, name: QualName) -> io::Result<()> {
        let info = self.stack.pop().expect("no ElemInfo");
        if info.ignore_children {
            return Ok(());
        }

        try!(self.writer.write_all(b"</"));
        try!(self.writer.write_all(tagname(&name).as_bytes()));
        self.writer.write_all(b">")
    }

    fn write_text(&mut self, text: &str) -> io::Result<()> {
        let prepend_lf = text.starts_with("\n") && {
            let parent = self.parent();
            !parent.processed_first_child && match parent.html_name {
                Some(local_name!("pre")) | Some(local_name!("textarea")) | Some(local_name!("listing")) => true,
                _ => false,
            }
        };

        if prepend_lf {
            try!(self.writer.write_all(b"\n"));
        }

        let escape = match self.parent().html_name {
            Some(local_name!("style")) | Some(local_name!("script")) | Some(local_name!("xmp"))
            | Some(local_name!("iframe")) | Some(local_name!("noembed")) | Some(local_name!("noframes"))
            | Some(local_name!("plaintext")) => false,

            Some(local_name!("noscript")) => !self.opts.scripting_enabled,

            _ => true,
        };

        if escape {
            self.write_escaped(text, false)
        } else {
            self.writer.write_all(text.as_bytes())
        }
    }

    fn write_comment(&mut self, text: &str) -> io::Result<()> {
        try!(self.writer.write_all(b"<!--"));
        try!(self.writer.write_all(text.as_bytes()));
        self.writer.write_all(b"-->")
    }

    fn write_doctype(&mut self, name: &str) -> io::Result<()> {
        try!(self.writer.write_all(b"<!DOCTYPE "));
        try!(self.writer.write_all(name.as_bytes()));
        self.writer.write_all(b">")
    }

    fn write_processing_instruction(&mut self, target: &str, data: &str) -> io::Result<()> {
        try!(self.writer.write_all(b"<?"));
        try!(self.writer.write_all(target.as_bytes()));
        try!(self.writer.write_all(b" "));
        try!(self.writer.write_all(data.as_bytes()));
        self.writer.write_all(b">")
    }
}
