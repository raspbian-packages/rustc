// Copyright 2017 The Rust Project Developers. See the COPYRIGHT
// file at the top-level directory of this distribution and at
// http://rust-lang.org/COPYRIGHT.
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.
#![deny(single_use_lifetimes)]
struct Foo<'x> { //~ ERROR lifetime name `'x` only used once
    x: &'x u32 // no warning!
}

// Once #44524 is fixed, this should issue a warning.
impl<'y> Foo<'y> { //~ ERROR lifetime name `'y` only used once
    fn method() { }
}

fn main() { }
