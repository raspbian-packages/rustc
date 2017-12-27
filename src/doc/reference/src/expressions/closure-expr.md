# Closure expressions

A _closure expression_ defines a closure and denotes it as a value, in a single
expression. A closure expression is a pipe-symbol-delimited (`|`) list of
patterns followed by an expression. Type annotations may optionally be added
for the type of the parameters or for the return type. If there is a return
type, the expression used for the body of the closure must be a normal
[block]. A closure expression also may begin with the
`move` keyword before the initial `|`.

A closure expression denotes a function that maps a list of parameters
(`ident_list`) onto the expression that follows the `ident_list`. The patterns
in the `ident_list` are the parameters to the closure. If a parameter's types
is not specified, then the compiler infers it from context. Each closure
expression has a unique anonymous type.

Closure expressions are most useful when passing functions as arguments to other
functions, as an abbreviation for defining and capturing a separate function.

Significantly, closure expressions _capture their environment_, which regular
[function definitions] do not. Without the `move`
keyword, the closure expression infers how it captures each variable from its
environment, preferring to capture by shared reference, effectively borrowing
all outer variables mentioned inside the closure's body. If needed the compiler
will infer that instead mutable references should be taken, or that the values
should be moved or copied (depending on their type) from the environment. A
closure can be forced to capture its environment by copying or moving values by
prefixing it with the `move` keyword. This is often used to ensure that the
closure's type is `'static`.

The compiler will determine which of the [closure
traits](types.html#closure-types) the closure's type will implement by how it
acts on its captured variables. The closure will also implement
[`Send`](the-send-trait.html) and/or [`Sync`](the-sync-trait.html) if all of
its captured types do. These traits allow functions to accept closures using
generics, even though the exact types can't be named.

In this example, we define a function `ten_times` that takes a higher-order
function argument, and we then call it with a closure expression as an argument,
followed by a closure expression that moves values from its environment.

```rust
fn ten_times<F>(f: F) where F: Fn(i32) {
    for index in 0..10 {
        f(index);
    }
}

ten_times(|j| println!("hello, {}", j));
// With type annotations
ten_times(|j: i32| -> () { println!("hello, {}", j) });

let word = "konnichiwa".to_owned();
ten_times(move |j| println!("{}, {}", word, j));
```

[block]: expressions/block-expr.html
[function definitions]: items/functions.html
