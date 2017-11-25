// Copyright 2014-2017 The html5ever Project Developers. See the
// COPYRIGHT file at the top-level directory of this distribution.
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

#[macro_use] extern crate html5ever;

use std::io;
use std::default::Default;
use std::collections::HashMap;
use std::borrow::Cow;

use html5ever::{Attribute, QualName, ExpandedName};
use html5ever::parse_document;
use html5ever::tree_builder::{TreeSink, QuirksMode, NodeOrText, ElementFlags};
use html5ever::tendril::*;

struct Sink {
    next_id: usize,
    names: HashMap<usize, QualName>,
}

impl Sink {
    fn get_id(&mut self) -> usize {
        let id = self.next_id;
        self.next_id += 2;
        id
    }
}

impl TreeSink for Sink {
    type Handle = usize;
    type Output = Self;
    fn finish(self) -> Self { self }

    fn get_document(&mut self) -> usize {
        0
    }

    fn get_template_contents(&mut self, target: &usize) -> usize {
        if let Some(expanded_name!(html "template")) = self.names.get(&target).map(|n| n.expanded()) {
            target + 1
        } else {
            panic!("not a template element")
        }
    }

    fn same_node(&self, x: &usize, y: &usize) -> bool {
        x == y
    }

    fn same_tree(&self, _x: &usize, _y: &usize) -> bool {
        true
    }

    fn elem_name(&self, target: &usize) -> ExpandedName {
        self.names.get(target).expect("not an element").expanded()
    }

    fn create_element(&mut self, name: QualName, _: Vec<Attribute>, _: ElementFlags) -> usize {
        let id = self.get_id();
        self.names.insert(id, name);
        id
    }

    fn create_comment(&mut self, _text: StrTendril) -> usize {
        self.get_id()
    }

    #[allow(unused_variables)]
    fn create_pi(&mut self, target: StrTendril, value: StrTendril) -> usize {
        unimplemented!()
    }

    fn has_parent_node(&self, _node: &usize) -> bool {
        // `node` will have a parent unless a script moved it, and we're
        // not running scripts.  Therefore we can aways return true.
        true
    }

    fn append_before_sibling(&mut self,
            _sibling: &usize,
            _new_node: NodeOrText<usize>) { }

    fn parse_error(&mut self, _msg: Cow<'static, str>) { }
    fn set_quirks_mode(&mut self, _mode: QuirksMode) { }
    fn append(&mut self, _parent: &usize, _child: NodeOrText<usize>) { }

    fn append_doctype_to_document(&mut self, _: StrTendril, _: StrTendril, _: StrTendril) { }
    fn add_attrs_if_missing(&mut self, target: &usize, _attrs: Vec<Attribute>) {
        assert!(self.names.contains_key(&target), "not an element");
    }
    fn remove_from_parent(&mut self, _target: &usize) { }
    fn reparent_children(&mut self, _node: &usize, _new_parent: &usize) { }
    fn mark_script_already_started(&mut self, _node: &usize) { }
}

fn main() {
    let sink = Sink {
        next_id: 1,
        names: HashMap::new(),
    };
    let stdin = io::stdin();
    parse_document(sink, Default::default())
        .from_utf8()
        .read_from(&mut stdin.lock())
        .unwrap();
}
