# External blocks

> **<sup>Syntax</sup>**\
> _ExternBlock_ :\
> &nbsp;&nbsp; `extern` [_Abi_]<sup>?</sup> `{`\
> &nbsp;&nbsp; &nbsp;&nbsp; [_InnerAttribute_]<sup>\*</sup>\
> &nbsp;&nbsp; &nbsp;&nbsp; _ExternalItem_<sup>\*</sup>\
> &nbsp;&nbsp; `}`
>
> _ExternalItem_ :\
> &nbsp;&nbsp; [_OuterAttribute_]<sup>\*</sup>\
> &nbsp;&nbsp; [_Visibility_]<sup>?</sup>\
> &nbsp;&nbsp; ( _ExternalStaticItem_ | _ExternalFunctionItem_ )
>
> _ExternalStaticItem_ :\
> &nbsp;&nbsp; `static` `mut`<sup>?</sup> [IDENTIFIER] `:` [_Type_] `;`
>
> _ExternalFunctionItem_ :\
> &nbsp;&nbsp; `fn` [IDENTIFIER]&nbsp;[_Generics_]<sup>?</sup>\
> &nbsp;&nbsp; `(` _NamedFunctionParameters_<sup>?</sup> | _NamedFunctionParametersWithVariadics_ ) `)`\
> &nbsp;&nbsp; [_FunctionReturnType_]<sup>?</sup> [_WhereClause_]<sup>?</sup> `;`
>
> _NamedFunctionParameters_ :\
> &nbsp;&nbsp; _NamedFunctionParam_ ( `,` _NamedFunctionParam_ )<sup>\*</sup> `,`<sup>?</sup>
>
> _NamedFunctionParam_ :\
> &nbsp;&nbsp; ( [IDENTIFIER] | `_` ) `:` [_Type_]
>
> _NamedFunctionParametersWithVariadics_ :\
> &nbsp;&nbsp; ( _NamedFunctionParam_ `,` )<sup>\*</sup> _NamedFunctionParam_ `,` `...`

External blocks form the basis for Rust's foreign function interface.
Declarations in an external block describe symbols in external, non-Rust
libraries.

Functions within external blocks are declared in the same way as other Rust
functions, with the exception that they may not have a body and are instead
terminated by a semicolon. Patterns are not allowed in parameters, only
[IDENTIFIER] or `_` may be used.

Functions within external blocks may be called by Rust code, just like
functions defined in Rust. The Rust compiler automatically translates between
the Rust ABI and the foreign ABI.

Functions within external blocks may be variadic by specifying `...` after one
or more named arguments in the argument list:

```rust,ignore
extern {
    fn foo(x: i32, ...);
}
```

A number of [attributes] control the behavior of external blocks.

By default external blocks assume that the library they are calling uses the
standard C ABI on the specific platform. Other ABIs may be specified using an
`abi` string, as shown here:

```rust,ignore
// Interface to the Windows API
extern "stdcall" { }
```

There are three ABI strings which are cross-platform, and which all compilers
are guaranteed to support:

* `extern "Rust"` -- The default ABI when you write a normal `fn foo()` in any
  Rust code.
* `extern "C"` -- This is the same as `extern fn foo()`; whatever the default
  your C compiler supports.
* `extern "system"` -- Usually the same as `extern "C"`, except on Win32, in
  which case it's `"stdcall"`, or what you should use to link to the Windows
  API itself

There are also some platform-specific ABI strings:

* `extern "cdecl"` -- The default for x86\_32 C code.
* `extern "stdcall"` -- The default for the Win32 API on x86\_32.
* `extern "win64"` -- The default for C code on x86\_64 Windows.
* `extern "sysv64"` -- The default for C code on non-Windows x86\_64.
* `extern "aapcs"` -- The default for ARM.
* `extern "fastcall"` -- The `fastcall` ABI -- corresponds to MSVC's
  `__fastcall` and GCC and clang's `__attribute__((fastcall))`
* `extern "vectorcall"` -- The `vectorcall` ABI -- corresponds to MSVC's
  `__vectorcall` and clang's `__attribute__((vectorcall))`

Finally, there are some rustc-specific ABI strings:

* `extern "rust-intrinsic"` -- The ABI of rustc intrinsics.
* `extern "rust-call"` -- The ABI of the Fn::call trait functions.
* `extern "platform-intrinsic"` -- Specific platform intrinsics -- like, for
  example, `sqrt` -- have this ABI. You should never have to deal with it.

The `link` attribute allows the name of the library to be specified. When
specified the compiler will attempt to link against the native library of the
specified name.

```rust,ignore
#[link(name = "crypto")]
extern { }
```

A function declared in an extern block is implicitly `unsafe`. When coerced to
a function pointer, a function declared in an extern block has type `unsafe
extern "abi" for<'l1, ..., 'lm> fn(A1, ..., An) -> R`, where `'l1`, ... `'lm`
are its lifetime parameters, `A1`, ..., `An` are the declared types of its
parameters and `R` is the declared return type.

It is `unsafe` to access a static item declared in an extern block, whether or
not it's mutable.

It is valid to add the `link` attribute on an empty extern block. You can use
this to satisfy the linking requirements of extern blocks elsewhere in your
code (including upstream crates) instead of adding the attribute to each extern
block.

[IDENTIFIER]: identifiers.html
[_Abi_]: items/functions.html
[_FunctionParam_]: items/functions.html
[_FunctionParameters_]: items/functions.html
[_FunctionReturnType_]: items/functions.html
[_Generics_]: items/generics.html
[_InnerAttribute_]: attributes.html
[_OuterAttribute_]: attributes.html
[_Type_]: types.html#type-expressions
[_Visibility_]: visibility-and-privacy.html
[_WhereClause_]: items/generics.html#where-clauses
[attributes]: attributes.html#ffi-attributes
