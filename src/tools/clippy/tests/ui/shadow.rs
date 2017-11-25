#![feature(plugin)]
#![plugin(clippy)]

#![warn(clippy, clippy_pedantic)]
#![allow(unused_parens, unused_variables, missing_docs_in_private_items)]

fn id<T>(x: T) -> T { x }

fn first(x: (isize, isize)) -> isize { x.0 }

fn main() {
    let mut x = 1;
    let x = &mut x;
    let x = { x };
    let x = (&*x);
    let x = { *x + 1 };
    let x = id(x);
    let x = (1, x);
    let x = first(x);
    let y = 1;
    let x = y;

    let x;
    x = 42;

    let o = Some(1_u8);

    if let Some(p) = o { assert_eq!(1, p); }
    match o {
        Some(p) => p, // no error, because the p above is in its own scope
        None => 0,
    };

    match (x, o) {
        (1, Some(a)) | (a, Some(1)) => (), // no error though `a` appears twice
        _ => (),
    }
}
