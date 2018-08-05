#![allow(blacklisted_name, collapsible_if, cyclomatic_complexity, eq_op, needless_continue,
         needless_return, never_loop, no_effect, zero_divided_by_zero)]

fn bar<T>(_: T) {}
fn foo() -> bool { unimplemented!() }

struct Foo {
    bar: u8,
}

pub enum Abc {
    A,
    B,
    C,
}

#[warn(if_same_then_else)]
#[warn(match_same_arms)]
fn if_same_then_else() -> Result<&'static str, ()> {
    if true {
        Foo { bar: 42 };
        0..10;
        ..;
        0..;
        ..10;
        0..=10;
        foo();
    }
    else { //~ ERROR same body as `if` block
        Foo { bar: 42 };
        0..10;
        ..;
        0..;
        ..10;
        0..=10;
        foo();
    }

    if true {
        Foo { bar: 42 };
    }
    else {
        Foo { bar: 43 };
    }

    if true {
        ();
    }
    else {
        ()
    }

    if true {
        0..10;
    }
    else {
        0..=10;
    }

    if true {
        foo();
        foo();
    }
    else {
        foo();
    }

    let _ = match 42 {
        42 => {
            foo();
            let mut a = 42 + [23].len() as i32;
            if true {
                a += 7;
            }
            a = -31-a;
            a
        }
        _ => { //~ ERROR match arms have same body
            foo();
            let mut a = 42 + [23].len() as i32;
            if true {
                a += 7;
            }
            a = -31-a;
            a
        }
    };

    let _ = match Abc::A {
        Abc::A => 0,
        Abc::B => 1,
        _ => 0, //~ ERROR match arms have same body
    };

    if true {
        foo();
    }

    let _ = if true {
        42
    }
    else { //~ ERROR same body as `if` block
        42
    };

    if true {
        for _ in &[42] {
            let foo: &Option<_> = &Some::<u8>(42);
            if true {
                break;
            } else {
                continue;
            }
        }
    }
    else { //~ ERROR same body as `if` block
        for _ in &[42] {
            let foo: &Option<_> = &Some::<u8>(42);
            if true {
                break;
            } else {
                continue;
            }
        }
    }

    if true {
        let bar = if true {
            42
        }
        else {
            43
        };

        while foo() { break; }
        bar + 1;
    }
    else { //~ ERROR same body as `if` block
        let bar = if true {
            42
        }
        else {
            43
        };

        while foo() { break; }
        bar + 1;
    }

    if true {
        let _ = match 42 {
            42 => 1,
            a if a > 0 => 2,
            10..=15 => 3,
            _ => 4,
        };
    }
    else if false {
        foo();
    }
    else if foo() {
        let _ = match 42 {
            42 => 1,
            a if a > 0 => 2,
            10..=15 => 3,
            _ => 4,
        };
    }

    if true {
        if let Some(a) = Some(42) {}
    }
    else { //~ ERROR same body as `if` block
        if let Some(a) = Some(42) {}
    }

    if true {
        if let (1, .., 3) = (1, 2, 3) {}
    }
    else { //~ ERROR same body as `if` block
        if let (1, .., 3) = (1, 2, 3) {}
    }

    if true {
        if let (1, .., 3) = (1, 2, 3) {}
    }
    else {
        if let (.., 3) = (1, 2, 3) {}
    }

    if true {
        if let (1, .., 3) = (1, 2, 3) {}
    }
    else {
        if let (.., 4) = (1, 2, 3) {}
    }

    if true {
        if let (1, .., 3) = (1, 2, 3) {}
    }
    else {
        if let (.., 1, 3) = (1, 2, 3) {}
    }

    if true {
        if let Some(42) = None {}
    }
    else {
        if let Option::Some(42) = None {}
    }

    if true {
        if let Some(42) = None::<u8> {}
    }
    else {
        if let Some(42) = None {}
    }

    if true {
        if let Some(42) = None::<u8> {}
    }
    else {
        if let Some(42) = None::<u32> {}
    }

    if true {
        if let Some(a) = Some(42) {}
    }
    else {
        if let Some(a) = Some(43) {}
    }

    let _ = match 42 {
        42 => foo(),
        51 => foo(), //~ ERROR match arms have same body
        _ => true,
    };

    let _ = match Some(42) {
        Some(_) => 24,
        None => 24, //~ ERROR match arms have same body
    };

    let _ = match Some(42) {
        Some(foo) => 24,
        None => 24,
    };

    let _ = match Some(42) {
        Some(42) => 24,
        Some(a) => 24, // bindings are different
        None => 0,
    };

    let _ = match Some(42) {
        Some(a) if a > 0 => 24,
        Some(a) => 24, // one arm has a guard
        None => 0,
    };

    match (Some(42), Some(42)) {
        (Some(a), None) => bar(a),
        (None, Some(a)) => bar(a), //~ ERROR match arms have same body
        _ => (),
    }

    match (Some(42), Some(42)) {
        (Some(a), ..) => bar(a),
        (.., Some(a)) => bar(a), //~ ERROR match arms have same body
        _ => (),
    }

    match (1, 2, 3) {
        (1, .., 3) => 42,
        (.., 3) => 42, //~ ERROR match arms have same body
        _ => 0,
    };

    let _ = if true {
        0.0
    } else { //~ ERROR same body as `if` block
        0.0
    };

    let _ = if true {
        -0.0
    } else { //~ ERROR same body as `if` block
        -0.0
    };

    let _ = if true {
        0.0
    } else {
        -0.0
    };

    // Different NaNs
    let _ = if true {
        0.0 / 0.0
    } else {
        std::f32::NAN
    };

    // Same NaNs
    let _ = if true {
        std::f32::NAN
    } else { //~ ERROR same body as `if` block
        std::f32::NAN
    };

    let _ = match Some(()) {
        Some(()) => 0.0,
        None => -0.0
    };

    match (Some(42), Some("")) {
        (Some(a), None) => bar(a),
        (None, Some(a)) => bar(a), // bindings have different types
        _ => (),
    }

    if true {
        try!(Ok("foo"));
    }
    else { //~ ERROR same body as `if` block
        try!(Ok("foo"));
    }

    if true {
        let foo = "";
        return Ok(&foo[0..]);
    }
    else if false {
        let foo = "bar";
        return Ok(&foo[0..]);
    }
    else {
        let foo = "";
        return Ok(&foo[0..]);
    }
}

#[warn(ifs_same_cond)]
#[allow(if_same_then_else)] // all empty blocks
fn ifs_same_cond() {
    let a = 0;
    let b = false;

    if b {
    }
    else if b { //~ ERROR ifs same condition
    }

    if a == 1 {
    }
    else if a == 1 { //~ ERROR ifs same condition
    }

    if 2*a == 1 {
    }
    else if 2*a == 2 {
    }
    else if 2*a == 1 { //~ ERROR ifs same condition
    }
    else if a == 1 {
    }

    // See #659
    if cfg!(feature = "feature1-659") {
        1
    } else if cfg!(feature = "feature2-659") {
        2
    } else {
        3
    };

    let mut v = vec![1];
    if v.pop() == None { // ok, functions
    }
    else if v.pop() == None {
    }

    if v.len() == 42 { // ok, functions
    }
    else if v.len() == 42 {
    }
}

fn main() {}

// Issue #2423. This was causing an ICE
fn func() {
    if true {
        f(&[0; 62]);
        f(&[0; 4]);
        f(&[0; 3]);
    } else {
        f(&[0; 62]);
        f(&[0; 6]);
        f(&[0; 6]);
    }
}

fn f(val: &[u8]) {}
