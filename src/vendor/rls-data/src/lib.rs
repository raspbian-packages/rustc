// Copyright 2017 The Rust Project Developers. See the COPYRIGHT
// at http://rust-lang.org/COPYRIGHT.
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

#![cfg_attr(rustbuild, feature(staged_api, rustc_private))]
#![cfg_attr(rustbuild, unstable(feature = "rustc_private", issue = "27812"))]

extern crate rustc_serialize;
extern crate rls_span as span;

#[cfg(feature = "serialize-serde")]
extern crate serde;
#[cfg(feature = "serialize-serde")]
#[macro_use]
extern crate serde_derive;

pub mod config;

use std::path::PathBuf;

use config::Config;


#[derive(Debug, Clone, RustcDecodable, RustcEncodable)]
#[repr(C)]
pub struct Analysis {
    /// The Config used to generate this analysis data.
    pub config: Config,
    pub prelude: Option<CratePreludeData>,
    pub imports: Vec<Import>,
    pub defs: Vec<Def>,
    pub impls: Vec<Impl>,
    pub refs: Vec<Ref>,
    pub macro_refs: Vec<MacroRef>,
    pub relations: Vec<Relation>,
    #[cfg(feature = "borrows")]
    pub per_fn_borrows: Vec<BorrowData>,
}

impl Analysis {
    #[cfg(not(feature = "borrows"))]
    pub fn new(config: Config) -> Analysis {
        Analysis {
            config,
            prelude: None,
            imports: vec![],
            defs: vec![],
            impls: vec![],
            refs: vec![],
            macro_refs: vec![],
            relations: vec![],
        }
    }

    #[cfg(feature = "borrows")]
    pub fn new(config: Config) -> Analysis {
        Analysis {
            config,
            prelude: None,
            imports: vec![],
            defs: vec![],
            impls: vec![],
            refs: vec![],
            macro_refs: vec![],
            relations: vec![],
            per_fn_borrows: vec![],
        }
    }
}

// DefId::index is a newtype and so the JSON serialisation is ugly. Therefore
// we use our own Id which is the same, but without the newtype.
#[derive(Clone, Copy, Debug, RustcDecodable, RustcEncodable, PartialEq, Eq, Hash)]
pub struct Id {
    pub krate: u32,
    pub index: u32,
}

#[derive(Debug, Clone, RustcDecodable, RustcEncodable)]
pub struct SpanData {
    pub file_name: PathBuf,
    pub byte_start: u32,
    pub byte_end: u32,
    pub line_start: span::Row<span::OneIndexed>,
    pub line_end: span::Row<span::OneIndexed>,
    // Character offset.
    pub column_start: span::Column<span::OneIndexed>,
    pub column_end: span::Column<span::OneIndexed>,
}

#[derive(Debug, Clone, RustcDecodable, RustcEncodable)]
pub struct CratePreludeData {
    pub crate_name: String,
    pub crate_root: String,
    pub external_crates: Vec<ExternalCrateData>,
    pub span: SpanData,
}

/// Data for external crates in the prelude of a crate.
#[derive(Debug, Clone, RustcDecodable, RustcEncodable)]
pub struct ExternalCrateData {
    pub name: String,
    pub num: u32,
    pub file_name: String,
}

#[derive(Debug, Clone, RustcDecodable, RustcEncodable)]
pub struct Import {
    pub kind: ImportKind,
    pub ref_id: Option<Id>,
    pub span: SpanData,
    pub name: String,
    pub value: String,
}

#[derive(Debug, RustcDecodable, RustcEncodable, Clone, Copy, PartialEq, Eq)]
pub enum ImportKind {
    ExternCrate,
    Use,
    GlobUse,
}

#[derive(Debug, Clone, RustcDecodable, RustcEncodable)]
pub struct Def {
    pub kind: DefKind,
    pub id: Id,
    pub span: SpanData,
    pub name: String,
    pub qualname: String,
    pub value: String,
    pub parent: Option<Id>,
    pub children: Vec<Id>,
    pub decl_id: Option<Id>,
    pub docs: String,
    pub sig: Option<Signature>,
    pub attributes: Vec<Attribute>,
}

#[derive(Debug, RustcDecodable, RustcEncodable, Clone, Copy, PartialEq, Eq)]
pub enum DefKind {
    // value = variant names
    Enum,
    // value = enum name + variant name + types
    Tuple,
    // value = [enum name +] name + fields
    Struct,
    Union,
    // value = signature
    Trait,
    // value = type + generics
    Function,
    // value = type + generics
    Method,
    // No id, no value.
    Macro,
    // value = file_name
    Mod,
    // value = aliased type
    Type,
    // value = type and init expression (for all variable kinds).
    Local,
    Static,
    Const,
    Field,
}

#[derive(Debug, Clone, RustcDecodable, RustcEncodable)]
pub struct Impl {
    pub id: u32,
    pub kind: ImplKind,
    pub span: SpanData,
    pub value: String,
    pub parent: Option<Id>,
    pub children: Vec<Id>,
    pub docs: String,
    pub sig: Option<Signature>,
    pub attributes: Vec<Attribute>,
}

#[derive(Debug, RustcDecodable, RustcEncodable, Clone, PartialEq, Eq)]
pub enum ImplKind {
    // impl Foo { ... }
    Inherent,
    // impl Bar for Foo { ... }
    Direct,
    // impl Bar for &Foo { ... }
    Indirect,
    // impl<T: Baz> Bar for T { ... }
    //   where Foo: Baz
    Blanket,
    // impl Bar for Baz { ... } or impl Baz { ... }, etc.
    //   where Foo: Deref<Target = Baz>
    // Args are name and id of Baz
    Deref(String, Id),
}

#[derive(Debug, Clone, RustcDecodable, RustcEncodable)]
pub struct Attribute {
    pub value: String,
    pub span: SpanData,
}

#[derive(Debug, Clone, RustcDecodable, RustcEncodable)]
pub struct Ref {
    pub kind: RefKind,
    pub span: SpanData,
    pub ref_id: Id,
}

#[derive(Debug, RustcDecodable, RustcEncodable, Clone, Copy, PartialEq, Eq)]
pub enum RefKind {
    Function,
    Mod,
    Type,
    Variable,
}

#[derive(Debug, Clone, RustcDecodable, RustcEncodable)]
pub struct MacroRef {
    pub span: SpanData,
    pub qualname: String,
    pub callee_span: SpanData,
}

#[derive(Debug, Clone, RustcDecodable, RustcEncodable)]
pub struct Relation {
    pub span: SpanData,
    pub kind: RelationKind,
    pub from: Id,
    pub to: Id,
}

#[derive(Debug, RustcDecodable, RustcEncodable, Clone, Copy, PartialEq, Eq)]
pub enum RelationKind {
    Impl,
    SuperTrait,
}

#[derive(Debug, Clone, RustcDecodable, RustcEncodable)]
pub struct Signature {
    pub text: String,
    pub defs: Vec<SigElement>,
    pub refs: Vec<SigElement>,
}

#[derive(Debug, Clone, RustcDecodable, RustcEncodable)]
pub struct SigElement {
    pub id: Id,
    pub start: usize,
    pub end: usize,
}

// Each `BorrowData` represents all of the scopes, loans and moves
// within an fn or closure referred to by `ref_id`.
#[cfg(feature = "borrows")]
#[derive(Debug, Clone, RustcDecodable, RustcEncodable)]
pub struct BorrowData {
    pub ref_id: Id,
    pub scopes: Vec<Scope>,
    pub loans: Vec<Loan>,
    pub moves: Vec<Move>,
    pub span: Option<SpanData>,
}

#[cfg(feature = "borrows")]
#[derive(Debug, RustcDecodable, RustcEncodable, Clone, Copy)]
pub enum BorrowKind {
    ImmBorrow,
    MutBorrow,
}

// Each `Loan` is either temporary or assigned to a variable.
// The `ref_id` refers to the value that is being loaned/borrowed.
// Not all loans will be valid. Invalid loans can be used to help explain
// improper usage.
#[cfg(feature = "borrows")]
#[derive(Debug, Clone, RustcDecodable, RustcEncodable)]
pub struct Loan {
    pub ref_id: Id,
    pub kind: BorrowKind,
    pub span: SpanData,
}

// Each `Move` represents an attempt to move the value referred to by `ref_id`.
// Not all `Move`s will be valid but can be used to help explain improper usage.
#[cfg(feature = "borrows")]
#[derive(Debug, Clone, RustcDecodable, RustcEncodable)]
pub struct Move {
    pub ref_id: Id,
    pub span: SpanData,
}

// Each `Scope` refers to "scope" of a variable (we don't track all values here).
// Its ref_id refers to the variable, and the span refers to the scope/region where
// the variable is "live".
#[cfg(feature = "borrows")]
#[derive(Debug, Clone, RustcDecodable, RustcEncodable)]
pub struct Scope {
    pub ref_id: Id,
    pub span: SpanData,
}
