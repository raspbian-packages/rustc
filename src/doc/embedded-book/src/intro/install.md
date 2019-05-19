# Installing the tools

This page contains OS-agnostic installation instructions for a few of the tools:

### Rust Toolchain

Install rustup by following the instructions at [https://rustup.rs](https://rustup.rs).

**NOTE** Make sure you have a compiler version equal to or newer than `1.31`. `rustc
-V` should return a date newer than the one shown below.

``` console
$ rustc -V
rustc 1.31.1 (b6c32da9b 2018-12-18)
```

For bandwidth and disk usage concerns the default installation only supports
native compilation. To add cross compilation support for the ARM Cortex-M
architecture install the following compilation targets.

``` console
$ rustup target add thumbv6m-none-eabi thumbv7m-none-eabi thumbv7em-none-eabi thumbv7em-none-eabihf
```

### `cargo-binutils`

``` console
$ cargo install cargo-binutils

$ rustup component add llvm-tools-preview
```

### OS-Specific Instructions

Now follow the instructions specific to the OS you are using:

- [Linux](install/linux.md)
- [Windows](install/windows.md)
- [macOS](install/macos.md)
