# Contributing to rust-clippy

Hello fellow Rustacean! Great to see your interest in compiler internals and lints!

## Getting started

High level approach:

1. Find something to fix/improve
2. Change code (likely some file in `clippy_lints/src/`)
3. Run `cargo test` in the root directory and wiggle code until it passes
4. Open a PR (also can be done between 2. and 3. if you run into problems)

### Finding something to fix/improve

All issues on Clippy are mentored, if you want help with a bug just ask @Manishearth, @llogiq, @mcarton or @oli-obk.

Some issues are easier than others. The [E-easy](https://github.com/rust-lang-nursery/rust-clippy/labels/E-easy)
label can be used to find the easy issues. If you want to work on an issue, please leave a comment
so that we can assign it to you!

Issues marked [T-AST](https://github.com/rust-lang-nursery/rust-clippy/labels/T-AST) involve simple
matching of the syntax tree structure, and are generally easier than
[T-middle](https://github.com/rust-lang-nursery/rust-clippy/labels/T-middle) issues, which involve types
and resolved paths.

Issues marked [E-medium](https://github.com/rust-lang-nursery/rust-clippy/labels/E-medium) are generally
pretty easy too, though it's recommended you work on an E-easy issue first. They are mostly classified
as `E-medium`, since they might be somewhat involved code wise, but not difficult per-se.

[Llogiq's blog post on lints](https://llogiq.github.io/2015/06/04/workflows.html) is a nice primer
to lint-writing, though it does get into advanced stuff. Most lints consist of an implementation of
`LintPass` with one or more of its default methods overridden. See the existing lints for examples
of this.

T-AST issues will generally need you to match against a predefined syntax structure. To figure out
how this syntax structure is encoded in the AST, it is recommended to run `rustc -Z ast-json` on an
example of the structure and compare with the
[nodes in the AST docs](http://manishearth.github.io/rust-internals-docs/syntax/ast/). Usually
the lint will end up to be a nested series of matches and ifs,
[like so](https://github.com/rust-lang-nursery/rust-clippy/blob/de5ccdfab68a5e37689f3c950ed1532ba9d652a0/src/misc.rs#L34).

T-middle issues can be more involved and require verifying types. The
[`ty`](http://manishearth.github.io/rust-internals-docs/rustc/ty) module contains a
lot of methods that are useful, though one of the most useful would be `expr_ty` (gives the type of
an AST expression). `match_def_path()` in Clippy's `utils` module can also be useful.

### Writing code

Compiling clippy can take almost a minute or more depending on your machine.
You can set the environment flag `CARGO_INCREMENTAL=1` to cut down that time to
almost a third on average, depending on the influence your change has.

Please document your lint with a doc comment akin to the following:

```rust
/// **What it does:** Checks for ... (describe what the lint matches).
///
/// **Why is this bad?** Supply the reason for linting the code.
///
/// **Known problems:** None. (Or describe where it could go wrong.)
///
/// **Example:**
/// ```rust
/// Insert a short example if you have one.
/// ```
```

### Running test suite

Clippy uses UI tests. UI tests check that the output of the compiler is exactly as expected.
Of course there's little sense in writing the output yourself or copying it around.
Therefore you can simply run `tests/ui/update-all-references.sh` and check whether
the output looks as you expect with `git diff`. Commit all `*.stderr` files, too.

### Testing manually

Manually testing against an example file is useful if you have added some
`println!`s and test suite output becomes unreadable.  To try clippy with your
local modifications, run `cargo run -- -L ./target/debug input.rs` from the
working copy root. Your test file, here `input.rs`, needs to have clippy
enabled as a plugin:

```rust
#![feature(plugin)]
#![plugin(clippy)]
```

## Contributions

Clippy welcomes contributions from everyone.

Contributions to Clippy should be made in the form of GitHub pull requests. Each pull request will
be reviewed by a core contributor (someone with permission to land patches) and either landed in the
main tree or given feedback for changes that would be required.

All code in this repository is under the [Mozilla Public License, 2.0](https://www.mozilla.org/MPL/2.0/)

## Conduct

We follow the [Rust Code of Conduct](http://www.rust-lang.org/conduct.html).


<!-- adapted from https://github.com/servo/servo/blob/master/CONTRIBUTING.md -->
