// Copyright 2017 The Rust Project Developers. See the COPYRIGHT
// file at the top-level directory of this distribution and at
// http://rust-lang.org/COPYRIGHT.
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

// run-pass
// Test that we "reset" the mode as we pass through a `&` pattern.
//
// cc #46688

fn surprise(x: i32) {
    assert_eq!(x, 2);
}

fn main() {
    let x = &(1, &2);
    let (_, &b) = x;
    surprise(b);
}
