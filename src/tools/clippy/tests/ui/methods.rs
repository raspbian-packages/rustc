
#![feature(const_fn)]

#![warn(clippy, clippy_pedantic, option_unwrap_used)]
#![allow(blacklisted_name, unused, print_stdout, non_ascii_literal, new_without_default,
    new_without_default_derive, missing_docs_in_private_items, needless_pass_by_value)]

use std::collections::BTreeMap;
use std::collections::HashMap;
use std::collections::HashSet;
use std::collections::VecDeque;
use std::ops::Mul;
use std::iter::FromIterator;
use std::rc::{self, Rc};
use std::sync::{self, Arc};

pub struct T;

impl T {
    pub fn add(self, other: T) -> T { self }

    pub(crate) fn drop(&mut self) { } // no error, not public interfact
    fn neg(self) -> Self { self } // no error, private function
    fn eq(&self, other: T) -> bool { true } // no error, private function

    fn sub(&self, other: T) -> &T { self } // no error, self is a ref
    fn div(self) -> T { self } // no error, different #arguments
    fn rem(self, other: T) { } // no error, wrong return type

    fn into_u32(self) -> u32 { 0 } // fine
    fn into_u16(&self) -> u16 { 0 }

    fn to_something(self) -> u32 { 0 }

    fn new(self) {}
}

struct Lt<'a> {
    foo: &'a u32,
}

impl<'a> Lt<'a> {
    // The lifetime is different, but that’s irrelevant, see #734
    #[allow(needless_lifetimes)]
    pub fn new<'b>(s: &'b str) -> Lt<'b> { unimplemented!() }
}

struct Lt2<'a> {
    foo: &'a u32,
}

impl<'a> Lt2<'a> {
    // The lifetime is different, but that’s irrelevant, see #734
    pub fn new(s: &str) -> Lt2 { unimplemented!() }
}

struct Lt3<'a> {
    foo: &'a u32,
}

impl<'a> Lt3<'a> {
    // The lifetime is different, but that’s irrelevant, see #734
    pub fn new() -> Lt3<'static> { unimplemented!() }
}

#[derive(Clone,Copy)]
struct U;

impl U {
    fn new() -> Self { U }
    fn to_something(self) -> u32 { 0 } // ok because U is Copy
}

struct V<T> {
    _dummy: T
}

impl<T> V<T> {
    fn new() -> Option<V<T>> { None }
}

impl Mul<T> for T {
    type Output = T;
    fn mul(self, other: T) -> T { self } // no error, obviously
}

/// Utility macro to test linting behavior in `option_methods()`
/// The lints included in `option_methods()` should not lint if the call to map is partially
/// within a macro
macro_rules! opt_map {
    ($opt:expr, $map:expr) => {($opt).map($map)};
}

/// Checks implementation of the following lints:
/// * `OPTION_MAP_UNWRAP_OR`
/// * `OPTION_MAP_UNWRAP_OR_ELSE`
/// * `OPTION_MAP_OR_NONE`
fn option_methods() {
    let opt = Some(1);

    // Check OPTION_MAP_UNWRAP_OR
    // single line case
    let _ = opt.map(|x| x + 1)

               .unwrap_or(0); // should lint even though this call is on a separate line
    // multi line cases
    let _ = opt.map(|x| {
                        x + 1
                    }
              ).unwrap_or(0);
    let _ = opt.map(|x| x + 1)
               .unwrap_or({
                    0
                });
    // single line `map(f).unwrap_or(None)` case
    let _ = opt.map(|x| Some(x + 1)).unwrap_or(None);
    // multiline `map(f).unwrap_or(None)` cases
    let _ = opt.map(|x| {
        Some(x + 1)
    }
    ).unwrap_or(None);
    let _ = opt
        .map(|x| Some(x + 1))
        .unwrap_or(None);
    // macro case
    let _ = opt_map!(opt, |x| x + 1).unwrap_or(0); // should not lint

    // Check OPTION_MAP_UNWRAP_OR_ELSE
    // single line case
    let _ = opt.map(|x| x + 1)

               .unwrap_or_else(|| 0); // should lint even though this call is on a separate line
    // multi line cases
    let _ = opt.map(|x| {
                        x + 1
                    }
              ).unwrap_or_else(|| 0);
    let _ = opt.map(|x| x + 1)
               .unwrap_or_else(||
                    0
                );
    // macro case
    let _ = opt_map!(opt, |x| x + 1).unwrap_or_else(|| 0); // should not lint

    // Check OPTION_MAP_OR_NONE
    // single line case
    let _ = opt.map_or(None, |x| Some(x + 1));
    // multi line case
    let _ = opt.map_or(None, |x| {
                        Some(x + 1)
                       }
                );
}

/// Checks implementation of the following lints:
/// * `RESULT_MAP_UNWRAP_OR_ELSE`
fn result_methods() {
    let res: Result<i32, ()> = Ok(1);

    // Check RESULT_MAP_UNWRAP_OR_ELSE
    // single line case
    let _ = res.map(|x| x + 1)

               .unwrap_or_else(|e| 0); // should lint even though this call is on a separate line
    // multi line cases
    let _ = res.map(|x| {
                        x + 1
                    }
              ).unwrap_or_else(|e| 0);
    let _ = res.map(|x| x + 1)
               .unwrap_or_else(|e|
                    0
                );
    // macro case
    let _ = opt_map!(res, |x| x + 1).unwrap_or_else(|e| 0); // should not lint
}

/// Struct to generate false positives for things with .iter()
#[derive(Copy, Clone)]
struct HasIter;

impl HasIter {
    fn iter(self) -> IteratorFalsePositives {
        IteratorFalsePositives { foo: 0 }
    }

    fn iter_mut(self) -> IteratorFalsePositives {
        IteratorFalsePositives { foo: 0 }
    }
}

/// Struct to generate false positive for Iterator-based lints
#[derive(Copy, Clone)]
struct IteratorFalsePositives {
    foo: u32,
}

impl IteratorFalsePositives {
    fn filter(self) -> IteratorFalsePositives {
        self
    }

    fn next(self) -> IteratorFalsePositives {
        self
    }

    fn find(self) -> Option<u32> {
        Some(self.foo)
    }

    fn position(self) -> Option<u32> {
        Some(self.foo)
    }

    fn rposition(self) -> Option<u32> {
        Some(self.foo)
    }

    fn nth(self, n: usize) -> Option<u32> {
        Some(self.foo)
    }

    fn skip(self, _: usize) -> IteratorFalsePositives {
        self
    }
}

/// Checks implementation of `FILTER_NEXT` lint
fn filter_next() {
    let v = vec![3, 2, 1, 0, -1, -2, -3];

    // check single-line case
    let _ = v.iter().filter(|&x| *x < 0).next();

    // check multi-line case
    let _ = v.iter().filter(|&x| {
                                *x < 0
                            }
                   ).next();

    // check that we don't lint if the caller is not an Iterator
    let foo = IteratorFalsePositives { foo: 0 };
    let _ = foo.filter().next();
}

/// Checks implementation of `SEARCH_IS_SOME` lint
fn search_is_some() {
    let v = vec![3, 2, 1, 0, -1, -2, -3];

    // check `find().is_some()`, single-line
    let _ = v.iter().find(|&x| *x < 0).is_some();

    // check `find().is_some()`, multi-line
    let _ = v.iter().find(|&x| {
                              *x < 0
                          }
                   ).is_some();

    // check `position().is_some()`, single-line
    let _ = v.iter().position(|&x| x < 0).is_some();

    // check `position().is_some()`, multi-line
    let _ = v.iter().position(|&x| {
                                  x < 0
                              }
                   ).is_some();

    // check `rposition().is_some()`, single-line
    let _ = v.iter().rposition(|&x| x < 0).is_some();

    // check `rposition().is_some()`, multi-line
    let _ = v.iter().rposition(|&x| {
                                   x < 0
                               }
                   ).is_some();

    // check that we don't lint if the caller is not an Iterator
    let foo = IteratorFalsePositives { foo: 0 };
    let _ = foo.find().is_some();
    let _ = foo.position().is_some();
    let _ = foo.rposition().is_some();
}

/// Checks implementation of the `OR_FUN_CALL` lint
fn or_fun_call() {
    struct Foo;

    impl Foo {
        fn new() -> Foo { Foo }
    }

    enum Enum {
        A(i32),
    }

    const fn make_const(i: i32) -> i32 { i }

    fn make<T>() -> T { unimplemented!(); }

    let with_enum = Some(Enum::A(1));
    with_enum.unwrap_or(Enum::A(5));

    let with_const_fn = Some(1);
    with_const_fn.unwrap_or(make_const(5));

    let with_constructor = Some(vec![1]);
    with_constructor.unwrap_or(make());

    let with_new = Some(vec![1]);
    with_new.unwrap_or(Vec::new());

    let with_const_args = Some(vec![1]);
    with_const_args.unwrap_or(Vec::with_capacity(12));

    let with_err : Result<_, ()> = Ok(vec![1]);
    with_err.unwrap_or(make());

    let with_err_args : Result<_, ()> = Ok(vec![1]);
    with_err_args.unwrap_or(Vec::with_capacity(12));

    let with_default_trait = Some(1);
    with_default_trait.unwrap_or(Default::default());

    let with_default_type = Some(1);
    with_default_type.unwrap_or(u64::default());

    let with_vec = Some(vec![1]);
    with_vec.unwrap_or(vec![]);

    // FIXME #944: ~|SUGGESTION with_vec.unwrap_or_else(|| vec![]);

    let without_default = Some(Foo);
    without_default.unwrap_or(Foo::new());

    let mut map = HashMap::<u64, String>::new();
    map.entry(42).or_insert(String::new());

    let mut btree = BTreeMap::<u64, String>::new();
    btree.entry(42).or_insert(String::new());

    let stringy = Some(String::from(""));
    let _ = stringy.unwrap_or("".to_owned());
}

/// Checks implementation of `ITER_NTH` lint
fn iter_nth() {
    let mut some_vec = vec![0, 1, 2, 3];
    let mut boxed_slice: Box<[u8]> = Box::new([0, 1, 2, 3]);
    let mut some_vec_deque: VecDeque<_> = some_vec.iter().cloned().collect();

    {
        // Make sure we lint `.iter()` for relevant types
        let bad_vec = some_vec.iter().nth(3);
        let bad_slice = &some_vec[..].iter().nth(3);
        let bad_boxed_slice = boxed_slice.iter().nth(3);
        let bad_vec_deque = some_vec_deque.iter().nth(3);
    }

    {
        // Make sure we lint `.iter_mut()` for relevant types
        let bad_vec = some_vec.iter_mut().nth(3);
    }
    {
        let bad_slice = &some_vec[..].iter_mut().nth(3);
    }
    {
        let bad_vec_deque = some_vec_deque.iter_mut().nth(3);
    }

    // Make sure we don't lint for non-relevant types
    let false_positive = HasIter;
    let ok = false_positive.iter().nth(3);
    let ok_mut = false_positive.iter_mut().nth(3);
}

/// Checks implementation of `ITER_SKIP_NEXT` lint
fn iter_skip_next() {
    let mut some_vec = vec![0, 1, 2, 3];
    let _ = some_vec.iter().skip(42).next();
    let _ = some_vec.iter().cycle().skip(42).next();
    let _ = (1..10).skip(10).next();
    let _ = &some_vec[..].iter().skip(3).next();
    let foo = IteratorFalsePositives { foo : 0 };
    let _ = foo.skip(42).next();
    let _ = foo.filter().skip(42).next();
}

/// Calls which should trigger the `UNNECESSARY_FOLD` lint
fn unnecessary_fold() {
    // Can be replaced by .any
    let _ = (0..3).fold(false, |acc, x| acc || x > 2);
    // Can be replaced by .all
    let _ = (0..3).fold(true, |acc, x| acc && x > 2);
    // Can be replaced by .sum
    let _ = (0..3).fold(0, |acc, x| acc + x);
    // Can be replaced by .product
    let _ = (0..3).fold(1, |acc, x| acc * x);
}

/// Should trigger the `UNNECESSARY_FOLD` lint, with an error span including exactly `.fold(...)`
fn unnecessary_fold_span_for_multi_element_chain() {
    let _ = (0..3).map(|x| 2 * x).fold(false, |acc, x| acc || x > 2);
}

/// Calls which should not trigger the `UNNECESSARY_FOLD` lint
fn unnecessary_fold_should_ignore() {
    let _ = (0..3).fold(true, |acc, x| acc || x > 2);
    let _ = (0..3).fold(false, |acc, x| acc && x > 2);
    let _ = (0..3).fold(1, |acc, x| acc + x);
    let _ = (0..3).fold(0, |acc, x| acc * x);
    let _ = (0..3).fold(0, |acc, x| 1 + acc + x);

    // We only match against an accumulator on the left
    // hand side. We could lint for .sum and .product when
    // it's on the right, but don't for now (and this wouldn't
    // be valid if we extended the lint to cover arbitrary numeric
    // types).
    let _ = (0..3).fold(false, |acc, x| x > 2 || acc);
    let _ = (0..3).fold(true, |acc, x| x > 2 && acc);
    let _ = (0..3).fold(0, |acc, x| x + acc);
    let _ = (0..3).fold(1, |acc, x| x * acc);

    let _ = [(0..2), (0..3)].iter().fold(0, |a, b| a + b.len());
    let _ = [(0..2), (0..3)].iter().fold(1, |a, b| a * b.len());
}

#[allow(similar_names)]
fn main() {
    let opt = Some(0);
    let _ = opt.unwrap();
}
