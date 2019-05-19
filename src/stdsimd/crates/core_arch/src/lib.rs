#![doc(include = "core_arch_docs.md")]
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
    stmt_expr_attributes,
    core_intrinsics,
    no_core,
    rustc_attrs,
    stdsimd,
    staged_api,
    align_offset,
    maybe_uninit,
    doc_cfg,
    mmx_target_feature,
    tbm_target_feature,
    sse4a_target_feature,
    arm_target_feature,
    aarch64_target_feature,
    cmpxchg16b_target_feature,
    avx512_target_feature,
    mips_target_feature,
    powerpc_target_feature,
    wasm_target_feature,
    abi_unadjusted,
    adx_target_feature,
    external_doc
)]
#![cfg_attr(test, feature(test, abi_vectorcall, untagged_unions))]
#![cfg_attr(
    feature = "cargo-clippy",
    deny(clippy::missing_inline_in_public_items,)
)]
#![cfg_attr(
    feature = "cargo-clippy",
    allow(
        clippy::inline_always,
        clippy::too_many_arguments,
        clippy::cast_sign_loss,
        clippy::cast_lossless,
        clippy::cast_possible_wrap,
        clippy::cast_possible_truncation,
        clippy::cast_precision_loss,
        clippy::shadow_reuse,
        clippy::cyclomatic_complexity,
        clippy::similar_names,
        clippy::many_single_char_names
    )
)]
#![cfg_attr(test, allow(unused_imports))]
#![no_core]
#![unstable(feature = "stdsimd", issue = "27731")]
#![doc(
    test(attr(deny(warnings))),
    test(attr(allow(dead_code, deprecated, unused_variables, unused_mut)))
)]

#[macro_use]
#[allow(unused_imports)]
extern crate core as _core;
#[cfg(test)]
#[macro_use]
extern crate std;
#[cfg(test)]
#[macro_use]
extern crate std_detect;
#[cfg(test)]
extern crate stdsimd_test;
#[cfg(test)]
extern crate test;

#[cfg(all(test, target_arch = "wasm32"))]
extern crate wasm_bindgen_test;

#[path = "mod.rs"]
mod core_arch;

pub use core_arch::arch::*;

#[allow(unused_imports)]
use _core::{
    clone, cmp, convert, default, fmt, hash, intrinsics, iter, marker, mem, num, ops, option,
    prelude, ptr, result, slice, sync, u128, u8,
};
