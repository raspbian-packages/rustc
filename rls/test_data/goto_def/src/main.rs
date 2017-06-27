// Copyright 2016 The Rust Project Developers. See the COPYRIGHT
// file at the top-level directory of this distribution and at
// http://rust-lang.org/COPYRIGHT.
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.
struct Bar {
    x: u64,
}

#[test]
pub fn test_fn() {
    let bar = Bar { x: 4 };
    println!("bar: {}", bar.x);
}

pub fn main() {
    let world = "world";
    println!("Hello, {}!", world);

    let bar2 = Bar { x: 5 };
    println!("bar2: {}", bar2.x);
}
