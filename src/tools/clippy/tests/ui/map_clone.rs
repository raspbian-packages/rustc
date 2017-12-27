#![feature(plugin)]

#![plugin(clippy)]
#![warn(map_clone)]

#![allow(clone_on_copy, unused)]

use std::ops::Deref;

fn map_clone_iter() {
    let x = [1,2,3];
    x.iter().map(|y| y.clone());

    x.iter().map(|&y| y);

    x.iter().map(|y| *y);

    x.iter().map(|y| { y.clone() });

    x.iter().map(|&y| { y });

    x.iter().map(|y| { *y });

    x.iter().map(Clone::clone);

}

fn map_clone_option() {
    let x = Some(4);
    x.as_ref().map(|y| y.clone());

    x.as_ref().map(|&y| y);

    x.as_ref().map(|y| *y);

}

fn not_linted_option() {
    let x = Some(5);

    // Not linted: other statements
    x.as_ref().map(|y| {
        println!("y: {}", y);
        y.clone()
    });

    // Not linted: argument bindings
    let x = Some((6, 7));
    x.map(|(y, _)| y.clone());

    // Not linted: cloning something else
    x.map(|y| y.0.clone());

    // Not linted: no dereferences
    x.map(|y| y);

    // Not linted: multiple dereferences
    let _: Option<(i32, i32)> = x.as_ref().as_ref().map(|&&x| x);
}

#[derive(Copy, Clone)]
struct Wrapper<T>(T);
impl<T> Wrapper<T> {
    fn map<U, F: FnOnce(T) -> U>(self, f: F) -> Wrapper<U> {
        Wrapper(f(self.0))
    }
}

fn map_clone_other() {
    let eight = 8;
    let x = Wrapper(&eight);

    // Not linted: not a linted type
    x.map(|y| y.clone());
    x.map(|&y| y);
    x.map(|y| *y);
}

#[derive(Copy, Clone)]
struct UnusualDeref;
static NINE: i32 = 9;

impl Deref for UnusualDeref {
    type Target = i32;
    fn deref(&self) -> &i32 { &NINE }
}

fn map_clone_deref() {
    let x = Some(UnusualDeref);
    let _: Option<UnusualDeref> = x.as_ref().map(|y| *y);


    // Not linted: using deref conversion
    let _: Option<i32> = x.map(|y| *y);

    // Not linted: using regular deref but also deref conversion
    let _: Option<i32> = x.as_ref().map(|y| **y);
}

// stuff that used to be a false positive
fn former_false_positive() {
    vec![1].iter_mut().map(|x| *x); // #443
}

fn main() { }
