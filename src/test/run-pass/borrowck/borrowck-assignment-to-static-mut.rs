// Copyright 2016 The Rust Project Developers. See the COPYRIGHT
// file at the top-level directory of this distribution and at
// http://rust-lang.org/COPYRIGHT.
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

// Test taken from #45641 (https://github.com/rust-lang/rust/issues/45641)

// ignore-tidy-linelength
// revisions: ast mir
//[mir]compile-flags: -Z emit-end-regions -Z borrowck-mir

static mut Y: u32 = 0;

unsafe fn should_ok() {
    Y = 1;
}

fn main() {}