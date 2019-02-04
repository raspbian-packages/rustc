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

/// A macro to make big else if conditions easier to read:
///
/// ```
/// # #[macro_use] extern crate macro_utils;
/// let s = "bateau";
///
/// if_match! {
///     s == "voiture" => println!("It rolls!"),
///     s == "avion"   => println!("It flies!"),
///     s == "pieds"   => println!("It walks!"),
///     s == "fusée"   => println!("It goes through space!"),
///     s == "bateau"  => println!("It moves on water!"),
///     else           => println!("I dont't know how it moves...")
/// }
/// ```
///
/// You can use it just like you would use conditions:
///
/// ```
/// # #[macro_use] extern crate macro_utils;
/// let x = -1;
///
/// let result = if_match! {
///     x >= 0 => "it's a positive number",
///     else   => "it's a negative number",
/// };
///
/// assert_eq!(result, "it's a negative number");
/// ```
///
/// And of course, the `else` condition is completely optional:
///
/// ```
/// # #[macro_use] extern crate macro_utils;
/// let x = 12;
///
/// if_match! {
///     x & 1 == 0 => println!("it is even"),
///     x & 1 == 1 => println!("it is odd"),
/// }
/// ```
///
/// Want to use `if let` conditions too? Here you go:
///
/// ```
/// # #[macro_use] extern crate macro_utils;
/// let v = 12;
/// let y = if_match! {
///     let 0 = 1 => 0,
///     v < 1 => 1,
///     v > 10 => 10,
///     let 0 = 1 => 0,
///     else => v
/// };
/// assert_eq!(y, 10);
/// ```
#[macro_export]
macro_rules! if_match {
    ($(let $expr:pat =)* $cond:expr => $then:expr $(,)*) => {
        if $(let $expr =)* $cond {
            $then
        }
    };
    ($(let $expr:pat =)* $cond:expr => $then:expr, else => $elsethen:expr $(,)*) => {
        if $(let $expr =)* $cond {
            $then
        } else {
            $elsethen
        }
    };
    ($(let $expr:pat =)* $cond:expr => $then:expr, $($(let $expr2:pat =)* $else_cond:expr => $else_then:expr,)* else => $else_expr:expr $(,)*) => {
        if $(let $expr =)* $cond {
            $then
        } $(else if $(let $expr2 =)* $else_cond {
            $else_then
        })* else {
            $else_expr
        }
    };
    ($(let $expr:pat =)* $cond:expr => $then:expr, $($(let $expr2:pat =)* $more:expr => $more_then:expr $(,)* )*) => {
        if $(let $expr =)* $cond {
            $then
        } $(else if $(let $expr2 =)* $more {
            $more_then
        })*
    };
    () => {};
}

#[test]
fn if_match() {
    fn check_complete(v: i32) -> i32 {
        if_match! {
            v < 1 => 1,
            v > 10 => 10,
            else => v,
        }
    }

    let x = 12;
    let is_even = if_match! {
        x % 2 == 0 => true,
        else => false,
    };
    assert_eq!(is_even, true);

    let mut not_zero = false;
    if_match! {
        x != 0 => {
            not_zero = true;
        },
    }
    assert_eq!(not_zero, true);

    assert_eq!(check_complete(1), 1);
    assert_eq!(check_complete(0), 1);
    assert_eq!(check_complete(12), 10);
}

#[test]
fn let_match() {
    let v = 12;
    let y = if_match! {
        let 0 = 1 => 0,
        v < 1 => 1,
        v > 10 => 10,
        let 0 = 1 => 0,
        else => v
    };
    assert_eq!(y, 10);

    let y = if_match! {
        v < 1 => 1,
        v > 10 => 10,
        let 0 = 1 => 0,
        else => v,
    };
    assert_eq!(y, 10);

    let mut z = 0;
    if_match! {
        v < 1 => z = 1,
        v > 10 => z = 10,
        let 0 = 1 => z = 0,
    }
    assert_eq!(z, 10);

    if_match! {
        v < 1 => z = 10,
        v > 10 => z = 2,
        let 0 = 1 => z = 0
    }
    assert_eq!(z, 2);

    if_match! {
        let 0 = 1 => println!("1"),
    }
}

#[test]
fn if_match_optional_end_comma() {
    let x = 42;

    if_match! {
        x != 0 => println!("ok")
    }

    if_match! {
        x != 0 => println!("ok"),
        else => print!("0")
    }

    if_match! {
        x & 1 == 0 => println!("it is even"),
        x & 1 == 1 => println!("it is odd"),
        else => print!("wtf is that")
    }

    if_match! {
        x & 1 == 0 => println!("it is even"),
        x & 1 == 1 => println!("it is odd")
    }
}
