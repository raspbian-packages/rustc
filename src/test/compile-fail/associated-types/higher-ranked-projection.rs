// Copyright 2016 The Rust Project Developers. See the COPYRIGHT
// file at the top-level directory of this distribution and at
// http://rust-lang.org/COPYRIGHT.
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

#![feature(rustc_attrs)]

// revisions: good bad

trait Mirror {
    type Image;
}

impl<T> Mirror for T {
    type Image = T;
}

#[cfg(bad)]
fn foo<U, T>(_t: T)
    where for<'a> &'a T: Mirror<Image=U>
{}

#[cfg(good)]
fn foo<U, T>(_t: T)
    where for<'a> &'a T: Mirror<Image=&'a U>
{}

#[rustc_error]
fn main() { //[good]~ ERROR compilation successful
    foo(());
    //[bad]~^ ERROR type mismatch resolving `for<'a> <&'a _ as Mirror>::Image == _`
    //[bad]~| expected bound lifetime parameter 'a, found concrete lifetime
}
