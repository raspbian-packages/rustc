# num-integer

[![crate](https://img.shields.io/crates/v/num-integer.svg)](https://crates.io/crates/num-integer)
[![documentation](https://docs.rs/num-integer/badge.svg)](https://docs.rs/num-integer)
![minimum rustc 1.8](https://img.shields.io/badge/rustc-1.8+-red.svg)
[![Travis status](https://travis-ci.org/rust-num/num-integer.svg?branch=master)](https://travis-ci.org/rust-num/num-integer)

`Integer` trait and functions for Rust.

## Usage

Add this to your `Cargo.toml`:

```toml
[dependencies]
num-integer = "0.1"
```

and this to your crate root:

```rust
extern crate num_integer;
```

## Features

This crate can be used without the standard library (`#![no_std]`) by disabling
the default `std` feature.  Use this in `Cargo.toml`:

```toml
[dependencies.num-integer]
version = "0.1.36"
default-features = false
```

There is no functional difference with and without `std` at this time, but
there may be in the future.

## Releases

Release notes are available in [RELEASES.md](RELEASES.md).

## Compatibility

The `num-integer` crate is tested for rustc 1.8 and greater.
