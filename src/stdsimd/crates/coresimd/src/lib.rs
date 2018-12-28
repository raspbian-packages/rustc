//! SIMD and vendor intrinsics support library.
//!
//! This documentation is for the `coresimd` crate, but you probably want to
//! use the [`stdsimd` crate][stdsimd] which should have more complete
//! documentation.
//!
//! [stdsimd]: https://rust-lang-nursery.github.io/stdsimd/x86_64/stdsimd/

#![cfg_attr(stdsimd_strict, deny(warnings))]
#![allow(dead_code)]
#![allow(unused_features)]
#![feature(
    const_fn,
    const_fn_union,
    link_llvm_intrinsics,
    platform_intrinsics,
    repr_simd,
    simd_ffi,
    asm,
    proc_macro_hygiene,
    integer_atomics,
    stmt_expr_attributes,
    core_intrinsics,
    no_core,
    rustc_attrs,
    stdsimd,
    staged_api,
    align_offset,
    doc_cfg,
    mmx_target_feature,
    tbm_target_feature,
    sse4a_target_feature,
    arm_target_feature,
    aarch64_target_feature,
    mips_target_feature,
    powerpc_target_feature,
)]
#![cfg_attr(
    test,
    feature(
        test,
        abi_vectorcall,
        untagged_unions
    )
)]
#![cfg_attr(
    feature = "cargo-clippy",
    allow(
        inline_always,
        too_many_arguments,
        cast_sign_loss,
        cast_lossless,
        cast_possible_wrap,
        cast_possible_truncation,
        cast_precision_loss,
        shadow_reuse,
        cyclomatic_complexity,
        similar_names,
        many_single_char_names
    )
)]
#![cfg_attr(test, allow(unused_imports))]
#![no_core]
#![unstable(feature = "stdsimd", issue = "27731")]
#![doc(
    test(attr(deny(warnings))),
    test(attr(allow(dead_code, deprecated, unused_variables, unused_mut)))
)]

extern crate core as _core;
#[cfg(test)]
#[macro_use]
extern crate std;
#[cfg(test)]
#[macro_use]
extern crate stdsimd;
#[cfg(test)]
extern crate stdsimd_test;
#[cfg(test)]
extern crate test;

#[cfg(all(test, target_arch = "wasm32"))]
extern crate wasm_bindgen_test;

#[path = "../../../coresimd/mod.rs"]
mod coresimd;

pub use coresimd::arch;

#[allow(unused_imports)]
use _core::clone;
#[allow(unused_imports)]
use _core::cmp;
#[allow(unused_imports)]
use _core::convert;
#[allow(unused_imports)]
use _core::default;
#[allow(unused_imports)]
use _core::fmt;
#[allow(unused_imports)]
use _core::hash;
#[allow(unused_imports)]
use _core::intrinsics;
#[allow(unused_imports)]
use _core::iter;
#[allow(unused_imports)]
use _core::marker;
#[allow(unused_imports)]
use _core::mem;
#[allow(unused_imports)]
use _core::num;
#[allow(unused_imports)]
use _core::ops;
#[allow(unused_imports)]
use _core::option;
#[allow(unused_imports)]
use _core::prelude;
#[allow(unused_imports)]
use _core::ptr;
#[allow(unused_imports)]
use _core::result;
#[allow(unused_imports)]
use _core::slice;
