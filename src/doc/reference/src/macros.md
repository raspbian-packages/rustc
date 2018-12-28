# Macros

A number of minor features of Rust are not central enough to have their own
syntax, and yet are not implementable as functions. Instead, they are given
names, and invoked through a consistent syntax: `some_extension!(...)`.

There are two ways to define new macros:

* [Macros by Example] define new syntax in a higher-level, declarative way.
* [Procedural Macros] can be used to implement custom derive.

## Macro Invocation

> **<sup>Syntax</sup>**\
> _MacroInvocation_ :\
> &nbsp;&nbsp; [_SimplePath_] `!` _DelimTokenTree_
>
> _DelimTokenTree_ :\
> &nbsp;&nbsp; &nbsp;&nbsp;  `(` _TokenTree_<sup>\*</sup> `)`\
> &nbsp;&nbsp; | `[` _TokenTree_<sup>\*</sup> `]`\
> &nbsp;&nbsp; | `{` _TokenTree_<sup>\*</sup> `}`
>
> _TokenTree_ :\
> &nbsp;&nbsp; [_Token_]<sub>_except delimiters_</sub> | _DelimTokenTree_
>
> _MacroInvocationSemi_ :\
> &nbsp;&nbsp; &nbsp;&nbsp; [_SimplePath_] `!` `(` _TokenTree_<sup>\*</sup> `)` `;`\
> &nbsp;&nbsp; | [_SimplePath_] `!` `[` _TokenTree_<sup>\*</sup> `]` `;`\
> &nbsp;&nbsp; | [_SimplePath_] `!` `{` _TokenTree_<sup>\*</sup> `}`

A macro invocation executes a macro at compile time and replaces the
invocation with the result of the macro. Macros may be invoked in the
following situations:

* [Expressions] and [statements]
* [Patterns]
* [Types]
* [Items] including [associated items]
* [`macro_rules`] transcribers

When used as an item or a statement, the _MacroInvocationSemi_ form is used
where a semicolon is required at the end when not using curly braces.
[Visibility qualifiers] are never allowed before a macro invocation or
[`macro_rules`] definition.

```rust
// Used as an expression.
let x = vec![1,2,3];

// Used as a statement.
println!("Hello!");

// Used in a pattern.
macro_rules! pat {
    ($i:ident) => (Some($i))
}

if let pat!(x) = Some(1) {
    assert_eq!(x, 1);
}

// Used in a type.
macro_rules! Tuple {
    { $A:ty, $B:ty } => { ($A, $B) };
}

type N2 = Tuple!(i32, i32);

// Used as an item.
# use std::cell::RefCell;
thread_local!(static FOO: RefCell<u32> = RefCell::new(1));

// Used as an associated item.
macro_rules! const_maker {
    ($t:ty, $v:tt) => { const CONST: $t = $v; };
}
trait T {
    const_maker!{i32, 7}
}

// Macro calls within macros.
macro_rules! example {
    () => { println!("Macro call in a macro!") };
}
// Outer macro `example` is expanded, then inner macro `println` is expanded.
example!();
```

[Macros by Example]: macros-by-example.html
[Procedural Macros]: procedural-macros.html
[_SimplePath_]: paths.html#simple-paths
[_Token_]: tokens.html
[associated items]: items/associated-items.html
[compiler plugins]: ../unstable-book/language-features/plugin.html
[delimiters]: tokens.html#delimiters
[expressions]: expressions.html
[items]: items.html
[`macro_rules`]: macros-by-example.html
[patterns]: patterns.html
[statements]: statements.html
[tokens]: tokens.html
[types]: types.html
[visibility qualifiers]: visibility-and-privacy.html
