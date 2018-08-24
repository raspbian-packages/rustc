//!# elasticlunr-rs
//!
//! [![Build Status](https://travis-ci.org/mattico/elasticlunr-rs.svg?branch=master)](https://travis-ci.org/mattico/elasticlunr-rs)
//! [![Documentation](https://docs.rs/elasticlunr-rs/badge.svg)](https://docs.rs/elasticlunr-rs)
//! [![Crates.io](https://img.shields.io/crates/v/elasticlunr-rs.svg)](https://crates.io/crates/elasticlunr-rs)
//!
//! A partial port of [elasticlunr](https://github.com/weixsong/elasticlunr.js) to Rust. Intended to
//! be used for generating compatible search indices.
//!
//! Access to all index-generating functionality is provided. Most users will only need to use the
//! [`Index`](struct.Index.html) or [`IndexBuilder`](struct.IndexBuilder.html) types.
//!
//! ## Example
//!
//! ```
//! use std::fs::File;
//! use std::io::Write;
//! use elasticlunr::Index;
//!
//! let mut index = Index::new(&["title", "body"]);
//! index.add_doc("1", &["This is a title", "This is body text!"]);
//! // Add more docs...
//! let mut file = File::create("out.json").unwrap();
//! file.write_all(index.to_json_pretty().as_bytes());
//! ```

#![cfg_attr(all(test, feature = "bench"), feature(test))]

#[macro_use]
extern crate lazy_static;
extern crate regex;
extern crate serde;
#[macro_use]
extern crate serde_derive;
extern crate serde_json;
extern crate strum;
#[macro_use]
extern crate strum_macros;

#[cfg(feature = "rust-stemmers")]
extern crate rust_stemmers;

#[cfg(test)]
#[macro_use]
extern crate maplit;

/// The version of elasticlunr.js this library was designed for.
pub const ELASTICLUNR_VERSION: &str = "0.9.5";

pub mod config;
pub mod document_store;
pub mod inverted_index;
pub mod lang;
pub mod pipeline;

use std::collections::HashMap;

use document_store::DocumentStore;
use inverted_index::InvertedIndex;
pub use lang::Language;
pub use pipeline::Pipeline;

/// A builder for an `Index` with custom parameters.
///
/// # Example
/// ```
/// # use elasticlunr::{Index, IndexBuilder};
/// let mut index = IndexBuilder::new()
///     .save_docs(false)
///     .add_fields(&["title", "subtitle", "body"])
///     .set_ref("doc_id")
///     .build();
/// index.add_doc("doc_a", &["Chapter 1", "Welcome to Copenhagen", "..."]);
/// ```
pub struct IndexBuilder {
    save: bool,
    fields: Vec<String>,
    ref_field: String,
    pipeline: Option<Pipeline>,
}

impl IndexBuilder {
    pub fn new() -> Self {
        IndexBuilder {
            save: true,
            fields: Vec::new(),
            ref_field: "id".into(),
            pipeline: None,
        }
    }

    /// Set whether or not documents should be saved in the `Index`'s document store.
    pub fn save_docs(mut self, save: bool) -> Self {
        self.save = save;
        self
    }

    /// Add a document field to the `Index`.
    pub fn add_field(mut self, field: &str) -> Self {
        self.fields.push(field.into());
        self
    }

    /// Add the document fields to the `Index`.
    pub fn add_fields<I>(mut self, fields: I) -> Self
    where
        I: IntoIterator,
        I::Item: AsRef<str>,
    {
        self.fields
            .extend(fields.into_iter().map(|f| f.as_ref().into()));
        self
    }

    /// Set the key used to store the document reference field.
    pub fn set_ref(mut self, ref_field: &str) -> Self {
        self.ref_field = ref_field.into();
        self
    }

    /// Set the pipeline used by the `Index`.
    pub fn set_pipeline(mut self, pipeline: Pipeline) -> Self {
        self.pipeline = Some(pipeline);
        self
    }

    /// Build an `Index` from this builder.
    pub fn build(self) -> Index {
        let index = self.fields
            .iter()
            .map(|f| (f.clone(), InvertedIndex::new()))
            .collect();

        Index {
            index,
            fields: self.fields,
            ref_field: self.ref_field,
            document_store: DocumentStore::new(self.save),
            pipeline: self.pipeline.unwrap_or_default(),
            version: ::ELASTICLUNR_VERSION,
        }
    }
}

/// An elasticlunr search index.
#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct Index {
    pub fields: Vec<String>,
    pub pipeline: Pipeline,
    #[serde(rename = "ref")]
    pub ref_field: String,
    pub version: &'static str,
    index: HashMap<String, InvertedIndex>,
    pub document_store: DocumentStore,
}

impl Index {
    /// Create a new index with the provided fields.
    ///
    /// # Example
    /// ```
    /// # use elasticlunr::Index;
    /// let mut index = Index::new(&["title", "body", "breadcrumbs"]);
    /// index.add_doc("1", &["How to Foo", "First, you need to `bar`.", "Chapter 1 > How to Foo"]);
    /// ```
    pub fn new<I>(fields: I) -> Self
    where
        I: IntoIterator,
        I::Item: AsRef<str>,
    {
        let mut indices = HashMap::new();
        let mut field_vec = Vec::new();
        for field in fields {
            let field = field.as_ref().to_string();
            field_vec.push(field.clone());
            indices.insert(field, InvertedIndex::new());
        }

        Index {
            fields: field_vec,
            index: indices,
            pipeline: Pipeline::default(),
            ref_field: "id".into(),
            version: ::ELASTICLUNR_VERSION,
            document_store: DocumentStore::new(true),
        }
    }

    /// Create a new index with the provided fields for the given
    /// [`Language`](lang/enum.Language.html).
    ///
    /// # Example
    /// ```
    /// # use elasticlunr::{Index, Language};
    /// let mut index = Index::with_language(Language::English, &["title", "body"]);
    /// index.add_doc("1", &["this is a title", "this is body text"]);
    /// ```
    pub fn with_language<I>(lang: Language, fields: I) -> Self
    where
        I: IntoIterator,
        I::Item: AsRef<str>,
    {
        let mut indices = HashMap::new();
        let mut field_vec = Vec::new();
        for field in fields {
            let field = field.as_ref().to_string();
            field_vec.push(field.clone());
            indices.insert(field, InvertedIndex::new());
        }

        Index {
            fields: field_vec,
            index: indices,
            pipeline: lang.make_pipeline(),
            ref_field: "id".into(),
            version: ::ELASTICLUNR_VERSION,
            document_store: DocumentStore::new(true),
        }
    }

    /// Add the data from a document to the index.
    ///
    /// *NOTE: The elements of `data` should be provided in the same order as
    /// the fields used to create the index.*
    ///
    /// # Example
    /// ```
    /// # use elasticlunr::Index;
    /// let mut index = Index::new(&["title", "body"]);
    /// index.add_doc("1", &["this is a title", "this is body text"]);
    /// ```
    pub fn add_doc<I>(&mut self, doc_ref: &str, data: I)
    where
        I: IntoIterator,
        I::Item: AsRef<str>,
    {
        let mut doc = HashMap::new();
        doc.insert(self.ref_field.clone(), doc_ref.into());
        let mut token_freq = HashMap::new();

        for (field, value) in self.fields.iter().zip(data) {
            doc.insert(field.clone(), value.as_ref().to_string());

            if field == &self.ref_field {
                continue;
            }

            let tokens = self.pipeline.run(pipeline::tokenize(value.as_ref()));
            self.document_store
                .add_field_length(doc_ref, field, tokens.len());

            for token in tokens {
                *token_freq.entry(token).or_insert(0u64) += 1;
            }

            for (token, count) in &token_freq {
                let freq = (*count as f64).sqrt();
                self.index
                    .get_mut(field)
                    .expect(&format!("InvertedIndex does not exist for field {}", field))
                    .add_token(doc_ref, token, freq);
            }
        }

        self.document_store.add_doc(doc_ref, doc);
    }

    pub fn get_fields(&self) -> &[String] {
        &self.fields
    }

    /// Returns the index, serialized to pretty-printed JSON.
    pub fn to_json_pretty(&self) -> String {
        serde_json::to_string_pretty(&self).unwrap()
    }

    /// Returns the index, serialized to JSON.
    pub fn to_json(&self) -> String {
        serde_json::to_string(&self).unwrap()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn add_field_to_builder() {
        let idx = IndexBuilder::new().add_field("foo").build();

        assert_eq!(idx.get_fields()[0], "foo");
    }

    #[test]
    fn adding_document_to_index() {
        let mut idx = Index::new(&["body"]);
        idx.add_doc("1", &["this is a test"]);

        assert_eq!(idx.document_store.len(), 1);
        assert_eq!(
            idx.document_store.get_doc("1").unwrap(),
            hashmap!{
                "id".into() => "1".into(),
                "body".into() => "this is a test".into(),
            }
        );
    }

    #[test]
    fn adding_document_with_empty_field() {
        let mut idx = Index::new(&["title", "body"]);

        idx.add_doc("1", &["", "test"]);
        assert_eq!(idx.index["body"].get_doc_frequency("test"), 1);
        assert_eq!(idx.index["body"].get_docs("test").unwrap()["1"], 1.);
    }
}
