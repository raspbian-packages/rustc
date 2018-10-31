// Copyright 2015 The Rust Project Developers. See the COPYRIGHT
// file at the top-level directory of this distribution and at
// http://rust-lang.org/COPYRIGHT.
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

#![allow(dead_code)]
#![feature(recover)]

use std::panic::UnwindSafe;

fn assert<T: UnwindSafe + ?Sized>() {}

fn main() {
    assert::<&mut i32>();
    //~^ ERROR the type `&mut i32` may not be safely transferred across an unwind boundary
}
