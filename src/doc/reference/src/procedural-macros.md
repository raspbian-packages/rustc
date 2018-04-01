## Procedural Macros

*Procedural macros* allow creating syntax extensions as execution of a function.
Procedural macros can be used to implement custom [derive] on your own
types. See [the book][procedural macros] for a tutorial.

Procedural macros involve a few different parts of the language and its
standard libraries. First is the `proc_macro` crate, included with Rust,
that defines an interface for building a procedural macro. The
`#[proc_macro_derive(Foo)]` attribute is used to mark the deriving
function. This function must have the type signature:

```rust,ignore
use proc_macro::TokenStream;

#[proc_macro_derive(Hello)]
pub fn hello_world(input: TokenStream) -> TokenStream
```

Finally, procedural macros must be in their own crate, with the `proc-macro`
crate type.

[derive]: attributes.html#derive
[procedural macros]: ../book/first-edition/procedural-macros.html