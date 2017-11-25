#![feature(plugin, box_syntax)]
#![plugin(clippy)]
#![allow(warnings, clippy)]

#![warn(boxed_local)]

#[derive(Clone)]
struct A;

impl A {
    fn foo(&self){}
}

trait Z {
    fn bar(&self);
}

impl Z for A {
    fn bar(&self) {
        //nothing
    }
}

fn main() {
}

fn ok_box_trait(boxed_trait: &Box<Z>) {
    let boxed_local = boxed_trait;
    // done
}

fn warn_call() {
    let x = box A;
    x.foo();
}

fn warn_arg(x: Box<A>) {
    x.foo();
}

fn nowarn_closure_arg() {
    let x = Some(box A);
    x.map_or((), |x| take_ref(&x));
}

fn warn_rename_call() {
    let x = box A;

    let y = x;
    y.foo(); // via autoderef
}

fn warn_notuse() {
    let bz = box A;
}

fn warn_pass() {
    let bz = box A;
    take_ref(&bz); // via deref coercion
}

fn nowarn_return() -> Box<A> {
    let fx = box A;
    fx // moved out, "escapes"
}

fn nowarn_move() {
    let bx = box A;
    drop(bx) // moved in, "escapes"
}
fn nowarn_call() {
    let bx = box A;
    bx.clone(); // method only available to Box, not via autoderef
}

fn nowarn_pass() {
    let bx = box A;
    take_box(&bx); // fn needs &Box
}


fn take_box(x: &Box<A>) {}
fn take_ref(x: &A) {}


fn nowarn_ref_take() {
    // false positive, should actually warn
    let x = box A;
    let y = &x;
    take_box(y);
}

fn nowarn_match() {
    let x = box A; // moved into a match
    match x {
        y => drop(y)
    }
}

fn warn_match() {
    let x = box A;
    match &x { // not moved
        ref y => ()
    }
}

fn nowarn_large_array() {
    // should not warn, is large array
    // and should not be on stack
    let x = box [1; 10000];
    match &x { // not moved
        ref y => ()
    }
}


/// ICE regression test
pub trait Foo {
    type Item;
}

impl<'a> Foo for &'a () {
    type Item = ();
}

pub struct PeekableSeekable<I: Foo> {
    _peeked: I::Item,
}

pub fn new(_needs_name: Box<PeekableSeekable<&()>>) -> () {
}
