# Crates and source files

> **<sup>Syntax</sup>**  
> _Crate_ :  
> &nbsp;&nbsp; UTF8BOM<sup>?</sup>  
> &nbsp;&nbsp; SHEBANG<sup>?</sup>  
> &nbsp;&nbsp; [_InnerAttribute_]<sup>\*</sup>  
> &nbsp;&nbsp; [_Item_]<sup>\*</sup>  

> **<sup>Lexer</sup>**  
> UTF8BOM : `\uFEFF`  
> SHEBANG : `#!` ~[`[` `\n`] ~`\n`<sup>*</sup>

Although Rust, like any other language, can be implemented by an interpreter as
well as a compiler, the only existing implementation is a compiler,
and the language has
always been designed to be compiled. For these reasons, this section assumes a
compiler.

Rust's semantics obey a *phase distinction* between compile-time and
run-time.[^phase-distinction] Semantic rules that have a *static
interpretation* govern the success or failure of compilation, while
semantic rules
that have a *dynamic interpretation* govern the behavior of the program at
run-time.

The compilation model centers on artifacts called _crates_. Each compilation
processes a single crate in source form, and if successful, produces a single
crate in binary form: either an executable or some sort of
library.[^cratesourcefile]

A _crate_ is a unit of compilation and linking, as well as versioning,
distribution and runtime loading. A crate contains a _tree_ of nested
[module] scopes. The top level of this tree is a module that is
anonymous (from the point of view of paths within the module) and any item
within a crate has a canonical [module path] denoting its location
within the crate's module tree.

The Rust compiler is always invoked with a single source file as input, and
always produces a single output crate. The processing of that source file may
result in other source files being loaded as modules. Source files have the
extension `.rs`.

A Rust source file describes a module, the name and location of which &mdash;
in the module tree of the current crate &mdash; are defined from outside the
source file: either by an explicit `mod_item` in a referencing source file, or
by the name of the crate itself. Every source file is a module, but not every
module needs its own source file: [module definitions][module] can be nested
within one file.

Each source file contains a sequence of zero or more `item` definitions, and
may optionally begin with any number of [attributes]
that apply to the containing module, most of which influence the behavior of
the compiler. The anonymous crate module can have additional attributes that
apply to the crate as a whole.

```rust,no_run
// Specify the crate name.
#![crate_name = "projx"]

// Specify the type of output artifact.
#![crate_type = "lib"]

// Turn on a warning.
// This can be done in any module, not just the anonymous crate module.
#![warn(non_camel_case_types)]
```

A crate that contains a `main` function can be compiled to an executable. If a
`main` function is present, its return type must be `()`
("[unit]") and it must take no arguments.

The optional [_UTF8 byte order mark_] (UTF8BOM production) indicates that the
file is encoded in UTF8. It can only occur at the beginning of the file and
is ignored by the compiler.

A source file can have a [_shebang_] (SHEBANG production), which indicates
to the operating system what program to use to execute this file. It serves
essentially to treat the source file as an executable script. The shebang
can only occur at the beginning of the file (but after the optional
_UTF8BOM_). It is ignored by the compiler. For example:

```text,ignore
#!/usr/bin/env rustx

fn main() {
    println!("Hello!");
}
```

[^phase-distinction]: This distinction would also exist in an interpreter.
    Static checks like syntactic analysis, type checking, and lints should
    happen before the program is executed regardless of when it is executed.

[^cratesourcefile]: A crate is somewhat analogous to an *assembly* in the
    ECMA-335 CLI model, a *library* in the SML/NJ Compilation Manager, a *unit*
    in the Owens and Flatt module system, or a *configuration* in Mesa.

[module]: items/modules.html
[module path]: paths.html
[attributes]: items-and-attributes.html
[unit]: types.html#tuple-types
[_InnerAttribute_]: attributes.html
[_Item_]: items.html
[_shebang_]: https://en.wikipedia.org/wiki/Shebang_(Unix)
[_utf8 byte order mark_]: https://en.wikipedia.org/wiki/Byte_order_mark#UTF-8
