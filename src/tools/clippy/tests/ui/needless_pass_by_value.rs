


#![warn(needless_pass_by_value)]
#![allow(dead_code, single_match, if_let_redundant_pattern_matching, many_single_char_names)]

use std::borrow::Borrow;
use std::convert::AsRef;

// `v` should be warned
// `w`, `x` and `y` are allowed (moved or mutated)
fn foo<T: Default>(v: Vec<T>, w: Vec<T>, mut x: Vec<T>, y: Vec<T>) -> Vec<T> {
    assert_eq!(v.len(), 42);

    consume(w);

    x.push(T::default());

    y
}

fn consume<T>(_: T) {}

struct Wrapper(String);

fn bar(x: String, y: Wrapper) {
    assert_eq!(x.len(), 42);
    assert_eq!(y.0.len(), 42);
}

// V implements `Borrow<V>`, but should be warned correctly
fn test_borrow_trait<T: Borrow<str>, U: AsRef<str>, V>(t: T, u: U, v: V) {
    println!("{}", t.borrow());
    println!("{}", u.as_ref());
    consume(&v);
}

// ok
fn test_fn<F: Fn(i32) -> i32>(f: F) {
    f(1);
}

// x should be warned, but y is ok
fn test_match(x: Option<Option<String>>, y: Option<Option<String>>) {
    match x {
        Some(Some(_)) => 1, // not moved
        _ => 0,
    };

    match y {
        Some(Some(s)) => consume(s), // moved
        _ => (),
    };
}

// x and y should be warned, but z is ok
fn test_destructure(x: Wrapper, y: Wrapper, z: Wrapper) {
    let Wrapper(s) = z; // moved
    let Wrapper(ref t) = y; // not moved
    let Wrapper(_) = y; // still not moved

    assert_eq!(x.0.len(), s.len());
    println!("{}", t);
}

trait Foo {}

// `S: Serialize` is allowed to be passed by value, since a caller can pass `&S` instead
trait Serialize {}
impl<'a, T> Serialize for &'a T where T: Serialize {}
impl Serialize for i32 {}

fn test_blanket_ref<T: Foo, S: Serialize>(_foo: T, _serializable: S) {}

fn issue_2114(s: String, t: String, u: Vec<i32>, v: Vec<i32>) {
    s.capacity();
    let _ = t.clone();
    u.capacity();
    let _ = v.clone();
}

struct S<T, U>(T, U);

impl<T: Serialize, U> S<T, U> {
    fn foo(
        self, // taking `self` by value is always allowed
        s: String,
        t: String,
    ) -> usize {
        s.len() + t.capacity()
    }

    fn bar(
        _t: T, // Ok, since `&T: Serialize` too
    ) {
    }

    fn baz(
        &self,
        _u: U,
        _s: Self,
    ) {
    }
}

trait FalsePositive {
    fn visit_str(s: &str);
    fn visit_string(s: String) {
        Self::visit_str(&s);
    }
}

fn main() {}
