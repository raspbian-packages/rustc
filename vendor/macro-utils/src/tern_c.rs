// MIT License
//
// Copyright (c) 2018 Guillaume Gomez
//
// Permission is hereby granted, free of charge, to any person obtaining a copy
// of this software and associated documentation files (the "Software"), to deal
// in the Software without restriction, including without limitation the rights
// to use, copy, modify, merge, publish, distribute, sublicense, and/or sell
// copies of the Software, and to permit persons to whom the Software is
// furnished to do so, subject to the following conditions:
//
// The above copyright notice and this permission notice shall be included in all
// copies or substantial portions of the Software.
//
// THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
// IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY,
// FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE
// AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER
// LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM,
// OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE
// SOFTWARE.

/// You miss C-like ternary conditions? Why not having them in Rust then?
///
/// ```
/// # #[macro_use] extern crate macro_utils;
/// let y = 4;
/// let x = tern_c! { (y & 1 == 0) ? { "even" } : { "odd" } };
///
/// println!("{} is {}", y, x);
/// ```
#[macro_export]
macro_rules! tern_c {
    (($cond:expr) ? { $if_expr:expr } : { $else_expr:expr }) => {
        if $cond {
            $if_expr
        } else {
            $else_expr
        }
    };
}

#[test]
fn tern_c() {
    let y = 4;
    let x = tern_c! { (y & 1 == 0) ? { "it's even" } : { "it's odd" } };

    assert_eq!(x, "it's even");
}
