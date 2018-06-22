#![warn(redundant_field_names)]
#![allow(unused_variables)]
#![feature(inclusive_range, inclusive_range_fields)]

#[macro_use]
extern crate derive_new;

use std::ops::{Range, RangeFrom, RangeTo, RangeInclusive, RangeToInclusive};

mod foo {
    pub const BAR: u8 = 0;
}

struct Person {
    gender: u8,
    age: u8,
    name: u8,
    buzz: u64,
    foo: u8,
}

#[derive(new)]
pub struct S {
    v: String,
}

fn main() {
    let gender: u8 = 42;
    let age = 0;
    let fizz: u64 = 0;
    let name: u8 = 0;

    let me = Person {
        gender: gender,
        age: age,

        name, //should be ok
        buzz: fizz, //should be ok
        foo: foo::BAR, //should be ok
    };

    // Range expressions
    let (start, end) = (0, 0);

    let _ = start..;
    let _ = ..end;
    let _ = start..end;

    let _ = ..=end;
    let _ = start..=end;

    // hand-written Range family structs are linted
    let _ = RangeFrom { start: start };
    let _ = RangeTo { end: end };
    let _ = Range { start: start, end: end };
    let _ = RangeInclusive { start: start, end: end };
    let _ = RangeToInclusive { end: end };
}
