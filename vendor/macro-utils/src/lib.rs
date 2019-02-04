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

//! Some useful and funny macros.
//!
//! To be able to use it, import is as follows:
//!
//! ```
//! #[macro_use]
//! extern crate macro_utils;
//! # fn main() {}
//! ```
//!
//! # Examples
//!
//! ```
//! # #[macro_use] extern crate macro_utils;
//! let s = "bateau";
//!
//! if_match! {
//!     s == "voiture" => println!("It rolls!"),
//!     s == "avion"   => println!("It flies!"),
//!     s == "pieds"   => println!("It walks!"),
//!     s == "fusée"   => println!("It goes through space!"),
//!     s == "bateau"  => println!("It moves on water!"),
//!     else           => println!("I dont't know how it moves...")
//! }
//!
//! let y = 4;
//! let x = tern_c! { (y & 1 == 0) ? { "even" } : { "odd" } };
//! let x = tern_python! { { "it's even" } if (y & 1 == 0) else { "it's odd" } };
//! let x = tern_haskell! { if (y & 1 == 0) then { "it's even" } else { "it's odd" } };
//! ```

#[macro_use]
mod if_match;
#[macro_use]
mod tern_c;
#[macro_use]
mod tern_haskell;
#[macro_use]
mod tern_python;
