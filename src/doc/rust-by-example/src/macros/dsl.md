# Domain Specific Languages (DSLs)

A DSL is a mini "language" embedded in a Rust macro. It is completely valid
Rust because the macro system expands into normal Rust constructs, but it looks
like a small language. This allows you to define concise or intuitive syntax for
some special functionality (within bounds).

Suppose that I want to define a little calculator API. I would like to supply
an expression an have the output printed to console.

```rust,editable
macro_rules! calculate {
    (eval $e:expr) => {{
        {
            let val: usize = $e; // Force types to be integers
            println!("{} = {}", stringify!{$e}, val);
        }
    }};
}

fn main() {
    calculate! {
        eval 1 + 2 // hehehe `eval` is _not_ a Rust keyword!
    }

    calculate! {
        eval (1 + 2) * (3 / 4)
    }
}
```

Output:
```
1 + 2 = 3
(1 + 2) * (3 / 4) = 0
```

This was a very simple example, but much more complex interfaces have been
developed, such as [`lazy_static`](https://crates.io/crates/lazy_static) or
[`clap`](https://crates.io/crates/clap).
