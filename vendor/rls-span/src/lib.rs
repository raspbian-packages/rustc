// Copyright 2016 The RLS Project Developers.
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

#![cfg_attr(rustbuild, feature(staged_api, rustc_private))]
#![cfg_attr(rustbuild, unstable(feature = "rustc_private", issue = "27812"))]

#![feature(proc_macro)]

#[cfg(feature = "serialize-serde")]
extern crate serde;
#[cfg(feature = "serialize-serde")]
#[macro_use]
extern crate serde_derive;
#[cfg(feature = "serialize-rustc")]
extern crate rustc_serialize;

#[cfg(feature = "serialize-rustc")]
use rustc_serialize::{Encodable, Decodable};
#[cfg(feature = "serialize-serde")]
use serde::{Serialize, Deserialize};

use std::marker::PhantomData;
use std::path::PathBuf;

pub mod compiler;

#[derive(Debug, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub struct Column<I: Indexed>(pub u32, PhantomData<I>);

impl<I: Indexed> Column<I> {
    fn new(c: u32) -> Column<I> {
        Column(c, PhantomData)
    }
}

impl<I: Indexed> Clone for Column<I> {
    fn clone(&self) -> Column<I> {
        *self
    }
}

impl<I: Indexed> Copy for Column<I> {}

#[cfg(feature = "serialize-serde")]
impl<I: Indexed> Serialize for Column<I> {
    fn serialize<S: serde::Serializer>(&self, s: S) -> Result<<S as serde::Serializer>::Ok, <S as serde::Serializer>::Error> {
        s.serialize_u32(self.0)
    }
}

#[cfg(feature = "serialize-serde")]
impl<'dt, I: Indexed> Deserialize<'dt> for Column<I> {
    fn deserialize<D: serde::Deserializer<'dt>>(d: D) -> std::result::Result<Self, <D as serde::Deserializer<'dt>>::Error> {
        <u32 as Deserialize>::deserialize(d).map(|x| Column::new(x))
    }
}

#[cfg(feature = "serialize-rustc")]
impl<I: Indexed> Decodable for Column<I> {
    fn decode<D: rustc_serialize::Decoder>(d: &mut D) -> Result<Column<I>, D::Error> {
        d.read_u32().map(|x| Column::new(x))
    }
}

#[cfg(feature = "serialize-rustc")]
impl<I: Indexed> Encodable for Column<I> {
    fn encode<S: rustc_serialize::Encoder>(&self, s: &mut S) -> Result<(), S::Error> {
        s.emit_u32(self.0)
    }
}

impl Column<OneIndexed> {
    pub fn new_one_indexed(c: u32) -> Column<OneIndexed> {
        Column(c, PhantomData)
    }

    pub fn zero_indexed(self) -> Column<ZeroIndexed> {
        Column(self.0 - 1, PhantomData)
    }
}

impl Column<ZeroIndexed> {
    pub fn new_zero_indexed(c: u32) -> Column<ZeroIndexed> {
        Column(c, PhantomData)
    }

    pub fn one_indexed(self) -> Column<OneIndexed> {
        Column(self.0 + 1, PhantomData)
    }
}

#[derive(Debug, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub struct Row<I: Indexed>(pub u32, PhantomData<I>);

impl<I: Indexed> Row<I> {
    fn new(c: u32) -> Row<I> {
        Row(c, PhantomData)
    }
}

impl<I: Indexed> Clone for Row<I> {
    fn clone(&self) -> Row<I> {
        *self
    }
}

impl<I: Indexed> Copy for Row<I> {}

#[cfg(feature = "serialize-serde")]
impl<I: Indexed> serde::Serialize for Row<I> {
    fn serialize<S: serde::Serializer>(&self, s: S) -> Result<S::Ok, S::Error> {
        s.serialize_u32(self.0)
    }
}

#[cfg(feature = "serialize-serde")]
impl<'dt, I: Indexed> serde::Deserialize<'dt> for Row<I> {
    fn deserialize<D: serde::Deserializer<'dt>>(d: D) -> std::result::Result<Self, D::Error> {
        <u32 as Deserialize>::deserialize(d).map(|x| Row::new(x))
    }
}

#[cfg(feature = "serialize-rustc")]
impl<I: Indexed> Decodable for Row<I> {
    fn decode<D: rustc_serialize::Decoder>(d: &mut D) -> Result<Row<I>, D::Error> {
        d.read_u32().map(|x| Row::new(x))
    }
}

#[cfg(feature = "serialize-rustc")]
impl<I: Indexed> Encodable for Row<I> {
    fn encode<S: rustc_serialize::Encoder>(&self, s: &mut S) -> Result<(), S::Error> {
        s.emit_u32(self.0)
    }
}

impl Row<OneIndexed> {
    pub fn new_one_indexed(c: u32) -> Row<OneIndexed> {
        Row(c, PhantomData)
    }

    pub fn zero_indexed(self) -> Row<ZeroIndexed> {
        Row(self.0 - 1, PhantomData)
    }
}

impl Row<ZeroIndexed> {
    pub fn new_zero_indexed(c: u32) -> Row<ZeroIndexed> {
        Row(c, PhantomData)
    }

    pub fn one_indexed(self) -> Row<OneIndexed> {
        Row(self.0 + 1, PhantomData)
    }
}

#[cfg_attr(feature = "serialize-rustc", derive(RustcDecodable, RustcEncodable))]
#[cfg_attr(feature = "serialize-serde", derive(Serialize, Deserialize))]
#[derive(Debug, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub struct Position<I: Indexed> {
    pub row: Row<I>,
    pub col: Column<I>,
}

impl<I: Indexed> Position<I> {
    pub fn new(row: Row<I>,
               col: Column<I>)
               -> Position<I> {
        Position {
            row: row,
            col: col,
        }
    }
}

impl<I: Indexed> Clone for Position<I> {
    fn clone(&self) -> Position<I> {
        *self
    }
}

impl<I: Indexed> Copy for Position<I> {}

impl Position<OneIndexed> {
    pub fn zero_indexed(self) -> Position<ZeroIndexed> {
        Position {
            row: self.row.zero_indexed(),
            col: self.col.zero_indexed(),
        }
    }
}

impl Position<ZeroIndexed> {
    pub fn one_indexed(self) -> Position<OneIndexed> {
        Position {
            row: self.row.one_indexed(),
            col: self.col.one_indexed(),
        }
    }
}

#[cfg_attr(feature = "serialize-rustc", derive(RustcDecodable, RustcEncodable))]
#[cfg_attr(feature = "serialize-serde", derive(Serialize, Deserialize))]
#[derive(Debug, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub struct Range<I: Indexed> {
    pub row_start: Row<I>,
    pub row_end: Row<I>,
    pub col_start: Column<I>,
    pub col_end: Column<I>,
}

impl<I: Indexed> Range<I> {
    pub fn new(row_start: Row<I>,
               row_end: Row<I>,
               col_start: Column<I>,
               col_end: Column<I>)
               -> Range<I> {
        Range {
            row_start: row_start,
            row_end: row_end,
            col_start: col_start,
            col_end: col_end,
        }
    }

    pub fn from_positions(start: Position<I>,
                          end: Position<I>)
                          -> Range<I> {
        Range {
            row_start: start.row,
            row_end: end.row,
            col_start: start.col,
            col_end: end.col,
        }
    }

    pub fn start(self) -> Position<I> {
        Position {
            row: self.row_start,
            col: self.col_start,
        }
    }

    pub fn end(self) -> Position<I> {
        Position {
            row: self.row_end,
            col: self.col_end,
        }
    }
}

impl<I: Indexed> Clone for Range<I> {
    fn clone(&self) -> Range<I> {
        *self
    }
}

impl<I: Indexed> Copy for Range<I> {}

impl Range<OneIndexed> {
    pub fn zero_indexed(self) -> Range<ZeroIndexed> {
        Range {
            row_start: self.row_start.zero_indexed(),
            row_end: self.row_end.zero_indexed(),
            col_start: self.col_start.zero_indexed(),
            col_end: self.col_end.zero_indexed(),
        }
    }
}

impl Range<ZeroIndexed> {
    pub fn one_indexed(self) -> Range<OneIndexed> {
        Range {
            row_start: self.row_start.one_indexed(),
            row_end: self.row_end.one_indexed(),
            col_start: self.col_start.one_indexed(),
            col_end: self.col_end.one_indexed(),
        }
    }
}

#[cfg_attr(feature = "serialize-rustc", derive(RustcDecodable, RustcEncodable))]
#[cfg_attr(feature = "serialize-serde", derive(Serialize, Deserialize))]
#[derive(Debug, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub struct Location<I: Indexed> {
    pub file: PathBuf,
    pub position: Position<I>,
}

impl<I: Indexed> Location<I> {
    pub fn new<F: Into<PathBuf>>(row: Row<I>,
                                 col: Column<I>,
                                 file: F)
                                 -> Location<I> {
        Location {
            position: Position {
                row: row,
                col: col,
            },
            file: file.into(),
        }
    }

    pub fn from_position<F: Into<PathBuf>>(position: Position<I>,
                                           file: F)
                                           -> Location<I> {
        Location {
            position: position,
            file: file.into(),
        }
    }
}

impl<I: Indexed> Clone for Location<I> {
    fn clone(&self) -> Location<I> {
        Location {
            position: self.position,
            file: self.file.clone(),
        }
    }
}

impl Location<OneIndexed> {
    pub fn zero_indexed(&self) -> Location<ZeroIndexed> {
        Location {
            position: self.position.zero_indexed(),
            file: self.file.clone(),
        }
    }
}

impl Location<ZeroIndexed> {
    pub fn one_indexed(&self) -> Location<OneIndexed> {
        Location {
            position: self.position.one_indexed(),
            file: self.file.clone(),
        }
    }
}

#[cfg_attr(feature = "serialize-rustc", derive(RustcDecodable, RustcEncodable))]
#[cfg_attr(feature = "serialize-serde", derive(Serialize, Deserialize))]
#[derive(Debug, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub struct Span<I: Indexed> {
    pub file: PathBuf,
    pub range: Range<I>,
}

impl<I: Indexed> Span<I> {
    pub fn new<F: Into<PathBuf>>(row_start: Row<I>,
                                 row_end: Row<I>,
                                 col_start: Column<I>,
                                 col_end: Column<I>,
                                 file: F)
                                 -> Span<I> {
        Span {
            range: Range {
                row_start: row_start,
                row_end: row_end,
                col_start: col_start,
                col_end: col_end,
            },
            file: file.into(),
        }
    }

    pub fn from_range<F: Into<PathBuf>>(range: Range<I>, file: F) -> Span<I> {
        Span {
            range: range,
            file: file.into(),
        }
    }

    pub fn from_positions<F: Into<PathBuf>>(start: Position<I>,
                                            end: Position<I>,
                                            file: F)
                                            -> Span<I> {
        Span {
            range: Range::from_positions(start, end),
            file: file.into(),
        }
    }
}

impl<I: Indexed> Clone for Span<I> {
    fn clone(&self) -> Span<I> {
        Span {
            range: self.range,
            file: self.file.clone(),
        }
    }
}

impl Span<OneIndexed> {
    pub fn zero_indexed(&self) -> Span<ZeroIndexed> {
        Span {
            range: self.range.zero_indexed(),
            file: self.file.clone(),
        }
    }
}

impl Span<ZeroIndexed> {
    pub fn one_indexed(&self) -> Span<OneIndexed> {
        Span {
            range: self.range.one_indexed(),
            file: self.file.clone(),
        }
    }
}

#[cfg(feature = "serialize-serde")]
pub trait Indexed: {}
#[cfg(not(feature = "serialize-serde"))]
pub trait Indexed: {}
#[cfg_attr(feature = "serialize-rustc", derive(RustcDecodable, RustcEncodable))]
#[cfg_attr(feature = "serialize-serde", derive(Serialize, Deserialize))]
#[derive(Hash, PartialEq, Eq, Debug, PartialOrd, Ord)]
pub struct ZeroIndexed;
impl Indexed for ZeroIndexed {}
#[cfg_attr(feature = "serialize-rustc", derive(RustcDecodable, RustcEncodable))]
#[cfg_attr(feature = "serialize-serde", derive(Serialize, Deserialize))]
#[derive(Hash, PartialEq, Eq, Debug, PartialOrd, Ord)]
pub struct OneIndexed;
impl Indexed for OneIndexed {}


#[cfg(test)]
mod test {
}
