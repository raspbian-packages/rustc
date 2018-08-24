//! ARMv8 ASIMD intrinsics

// FIXME: replace neon with asimd

use coresimd::arm::*;
use coresimd::simd::*;
use coresimd::simd_llvm::simd_add;
#[cfg(test)]
use stdsimd_test::assert_instr;

types! {
    /// ARM-specific 64-bit wide vector of one packed `f64`.
    pub struct float64x1_t(f64); // FIXME: check this!
    /// ARM-specific 128-bit wide vector of two packed `f64`.
    pub struct float64x2_t(f64, f64);
}
impl_from_bits_!(
    float64x1_t: u32x2,
    i32x2,
    f32x2,
    m32x2,
    u16x4,
    i16x4,
    m16x4,
    u8x8,
    i8x8,
    m8x8
);
impl_from_bits_!(
    float64x2_t: u64x2,
    i64x2,
    f64x2,
    m64x2,
    u32x4,
    i32x4,
    f32x4,
    m32x4,
    u16x8,
    i16x8,
    m16x8,
    u8x16,
    i8x16,
    m8x16
);

#[allow(improper_ctypes)]
extern "C" {
    #[link_name = "llvm.aarch64.neon.smaxv.i8.v8i8"]
    fn vmaxv_s8_(a: int8x8_t) -> i8;
    #[link_name = "llvm.aarch64.neon.smaxv.i8.6i8"]
    fn vmaxvq_s8_(a: int8x16_t) -> i8;
    #[link_name = "llvm.aarch64.neon.smaxv.i16.v4i16"]
    fn vmaxv_s16_(a: int16x4_t) -> i16;
    #[link_name = "llvm.aarch64.neon.smaxv.i16.v8i16"]
    fn vmaxvq_s16_(a: int16x8_t) -> i16;
    #[link_name = "llvm.aarch64.neon.smaxv.i32.v2i32"]
    fn vmaxv_s32_(a: int32x2_t) -> i32;
    #[link_name = "llvm.aarch64.neon.smaxv.i32.v4i32"]
    fn vmaxvq_s32_(a: int32x4_t) -> i32;

    #[link_name = "llvm.aarch64.neon.umaxv.i8.v8i8"]
    fn vmaxv_u8_(a: uint8x8_t) -> u8;
    #[link_name = "llvm.aarch64.neon.umaxv.i8.6i8"]
    fn vmaxvq_u8_(a: uint8x16_t) -> u8;
    #[link_name = "llvm.aarch64.neon.umaxv.i16.v4i16"]
    fn vmaxv_u16_(a: uint16x4_t) -> u16;
    #[link_name = "llvm.aarch64.neon.umaxv.i16.v8i16"]
    fn vmaxvq_u16_(a: uint16x8_t) -> u16;
    #[link_name = "llvm.aarch64.neon.umaxv.i32.v2i32"]
    fn vmaxv_u32_(a: uint32x2_t) -> u32;
    #[link_name = "llvm.aarch64.neon.umaxv.i32.v4i32"]
    fn vmaxvq_u32_(a: uint32x4_t) -> u32;

    #[link_name = "llvm.aarch64.neon.fmaxv.f32.v2f32"]
    fn vmaxv_f32_(a: float32x2_t) -> f32;
    #[link_name = "llvm.aarch64.neon.fmaxv.f32.v4f32"]
    fn vmaxvq_f32_(a: float32x4_t) -> f32;
    #[link_name = "llvm.aarch64.neon.fmaxv.f64.v2f64"]
    fn vmaxvq_f64_(a: float64x2_t) -> f64;

    #[link_name = "llvm.aarch64.neon.sminv.i8.v8i8"]
    fn vminv_s8_(a: int8x8_t) -> i8;
    #[link_name = "llvm.aarch64.neon.sminv.i8.6i8"]
    fn vminvq_s8_(a: int8x16_t) -> i8;
    #[link_name = "llvm.aarch64.neon.sminv.i16.v4i16"]
    fn vminv_s16_(a: int16x4_t) -> i16;
    #[link_name = "llvm.aarch64.neon.sminv.i16.v8i16"]
    fn vminvq_s16_(a: int16x8_t) -> i16;
    #[link_name = "llvm.aarch64.neon.sminv.i32.v2i32"]
    fn vminv_s32_(a: int32x2_t) -> i32;
    #[link_name = "llvm.aarch64.neon.sminv.i32.v4i32"]
    fn vminvq_s32_(a: int32x4_t) -> i32;

    #[link_name = "llvm.aarch64.neon.uminv.i8.v8i8"]
    fn vminv_u8_(a: uint8x8_t) -> u8;
    #[link_name = "llvm.aarch64.neon.uminv.i8.6i8"]
    fn vminvq_u8_(a: uint8x16_t) -> u8;
    #[link_name = "llvm.aarch64.neon.uminv.i16.v4i16"]
    fn vminv_u16_(a: uint16x4_t) -> u16;
    #[link_name = "llvm.aarch64.neon.uminv.i16.v8i16"]
    fn vminvq_u16_(a: uint16x8_t) -> u16;
    #[link_name = "llvm.aarch64.neon.uminv.i32.v2i32"]
    fn vminv_u32_(a: uint32x2_t) -> u32;
    #[link_name = "llvm.aarch64.neon.uminv.i32.v4i32"]
    fn vminvq_u32_(a: uint32x4_t) -> u32;

    #[link_name = "llvm.aarch64.neon.fminv.f32.v2f32"]
    fn vminv_f32_(a: float32x2_t) -> f32;
    #[link_name = "llvm.aarch64.neon.fminv.f32.v4f32"]
    fn vminvq_f32_(a: float32x4_t) -> f32;
    #[link_name = "llvm.aarch64.neon.fminv.f64.v2f64"]
    fn vminvq_f64_(a: float64x2_t) -> f64;

    #[link_name = "llvm.aarch64.neon.sminp.v16i8"]
    fn vpminq_s8_(a: int8x16_t, b: int8x16_t) -> int8x16_t;
    #[link_name = "llvm.aarch64.neon.sminp.v8i16"]
    fn vpminq_s16_(a: int16x8_t, b: int16x8_t) -> int16x8_t;
    #[link_name = "llvm.aarch64.neon.sminp.v4i32"]
    fn vpminq_s32_(a: int32x4_t, b: int32x4_t) -> int32x4_t;
    #[link_name = "llvm.aarch64.neon.uminp.v16i8"]
    fn vpminq_u8_(a: uint8x16_t, b: uint8x16_t) -> uint8x16_t;
    #[link_name = "llvm.aarch64.neon.uminp.v8i16"]
    fn vpminq_u16_(a: uint16x8_t, b: uint16x8_t) -> uint16x8_t;
    #[link_name = "llvm.aarch64.neon.uminp.v4i32"]
    fn vpminq_u32_(a: uint32x4_t, b: uint32x4_t) -> uint32x4_t;
    #[link_name = "llvm.aarch64.neon.fminp.4f32"]
    fn vpminq_f32_(a: float32x4_t, b: float32x4_t) -> float32x4_t;
    #[link_name = "llvm.aarch64.neon.fminp.v2f64"]
    fn vpminq_f64_(a: float64x2_t, b: float64x2_t) -> float64x2_t;

    #[link_name = "llvm.aarch64.neon.smaxp.v16i8"]
    fn vpmaxq_s8_(a: int8x16_t, b: int8x16_t) -> int8x16_t;
    #[link_name = "llvm.aarch64.neon.smaxp.v8i16"]
    fn vpmaxq_s16_(a: int16x8_t, b: int16x8_t) -> int16x8_t;
    #[link_name = "llvm.aarch64.neon.smaxp.v4i32"]
    fn vpmaxq_s32_(a: int32x4_t, b: int32x4_t) -> int32x4_t;
    #[link_name = "llvm.aarch64.neon.umaxp.v16i8"]
    fn vpmaxq_u8_(a: uint8x16_t, b: uint8x16_t) -> uint8x16_t;
    #[link_name = "llvm.aarch64.neon.umaxp.v8i16"]
    fn vpmaxq_u16_(a: uint16x8_t, b: uint16x8_t) -> uint16x8_t;
    #[link_name = "llvm.aarch64.neon.umaxp.v4i32"]
    fn vpmaxq_u32_(a: uint32x4_t, b: uint32x4_t) -> uint32x4_t;
    #[link_name = "llvm.aarch64.neon.fmaxp.4f32"]
    fn vpmaxq_f32_(a: float32x4_t, b: float32x4_t) -> float32x4_t;
    #[link_name = "llvm.aarch64.neon.fmaxp.v2f64"]
    fn vpmaxq_f64_(a: float64x2_t, b: float64x2_t) -> float64x2_t;
}

/// Vector add.
#[inline]
#[target_feature(enable = "neon")]
#[cfg_attr(test, assert_instr(fadd))]
pub unsafe fn vadd_f64(a: float64x1_t, b: float64x1_t) -> float64x1_t {
    simd_add(a, b)
}

/// Vector add.
#[inline]
#[target_feature(enable = "neon")]
#[cfg_attr(test, assert_instr(fadd))]
pub unsafe fn vaddq_f64(a: float64x2_t, b: float64x2_t) -> float64x2_t {
    simd_add(a, b)
}

/// Vector add.
#[inline]
#[target_feature(enable = "neon")]
#[cfg_attr(test, assert_instr(add))]
pub unsafe fn vaddd_s64(a: int64x1_t, b: int64x1_t) -> int64x1_t {
    simd_add(a, b)
}

/// Vector add.
#[inline]
#[target_feature(enable = "neon")]
#[cfg_attr(test, assert_instr(add))]
pub unsafe fn vaddd_u64(a: uint64x1_t, b: uint64x1_t) -> uint64x1_t {
    simd_add(a, b)
}

/// Horizontal vector max.
#[inline]
#[target_feature(enable = "neon")]
#[cfg_attr(test, assert_instr(smaxv))]
pub unsafe fn vmaxv_s8(a: int8x8_t) -> i8 {
    vmaxv_s8_(a)
}

/// Horizontal vector max.
#[inline]
#[target_feature(enable = "neon")]
#[cfg_attr(test, assert_instr(smaxv))]
pub unsafe fn vmaxvq_s8(a: int8x16_t) -> i8 {
    vmaxvq_s8_(a)
}

/// Horizontal vector max.
#[inline]
#[target_feature(enable = "neon")]
#[cfg_attr(test, assert_instr(smaxv))]
pub unsafe fn vmaxv_s16(a: int16x4_t) -> i16 {
    vmaxv_s16_(a)
}

/// Horizontal vector max.
#[inline]
#[target_feature(enable = "neon")]
#[cfg_attr(test, assert_instr(smaxv))]
pub unsafe fn vmaxvq_s16(a: int16x8_t) -> i16 {
    vmaxvq_s16_(a)
}

/// Horizontal vector max.
#[inline]
#[target_feature(enable = "neon")]
#[cfg_attr(test, assert_instr(smaxp))]
pub unsafe fn vmaxv_s32(a: int32x2_t) -> i32 {
    vmaxv_s32_(a)
}

/// Horizontal vector max.
#[inline]
#[target_feature(enable = "neon")]
#[cfg_attr(test, assert_instr(smaxv))]
pub unsafe fn vmaxvq_s32(a: int32x4_t) -> i32 {
    vmaxvq_s32_(a)
}

/// Horizontal vector max.
#[inline]
#[target_feature(enable = "neon")]
#[cfg_attr(test, assert_instr(umaxv))]
pub unsafe fn vmaxv_u8(a: uint8x8_t) -> u8 {
    vmaxv_u8_(a)
}

/// Horizontal vector max.
#[inline]
#[target_feature(enable = "neon")]
#[cfg_attr(test, assert_instr(umaxv))]
pub unsafe fn vmaxvq_u8(a: uint8x16_t) -> u8 {
    vmaxvq_u8_(a)
}

/// Horizontal vector max.
#[inline]
#[target_feature(enable = "neon")]
#[cfg_attr(test, assert_instr(umaxv))]
pub unsafe fn vmaxv_u16(a: uint16x4_t) -> u16 {
    vmaxv_u16_(a)
}

/// Horizontal vector max.
#[inline]
#[target_feature(enable = "neon")]
#[cfg_attr(test, assert_instr(umaxv))]
pub unsafe fn vmaxvq_u16(a: uint16x8_t) -> u16 {
    vmaxvq_u16_(a)
}

/// Horizontal vector max.
#[inline]
#[target_feature(enable = "neon")]
#[cfg_attr(test, assert_instr(umaxp))]
pub unsafe fn vmaxv_u32(a: uint32x2_t) -> u32 {
    vmaxv_u32_(a)
}

/// Horizontal vector max.
#[inline]
#[target_feature(enable = "neon")]
#[cfg_attr(test, assert_instr(umaxv))]
pub unsafe fn vmaxvq_u32(a: uint32x4_t) -> u32 {
    vmaxvq_u32_(a)
}

/// Horizontal vector max.
#[inline]
#[target_feature(enable = "neon")]
#[cfg_attr(test, assert_instr(fmaxp))]
pub unsafe fn vmaxv_f32(a: float32x2_t) -> f32 {
    vmaxv_f32_(a)
}

/// Horizontal vector max.
#[inline]
#[target_feature(enable = "neon")]
#[cfg_attr(test, assert_instr(fmaxv))]
pub unsafe fn vmaxvq_f32(a: float32x4_t) -> f32 {
    vmaxvq_f32_(a)
}

/// Horizontal vector max.
#[inline]
#[target_feature(enable = "neon")]
#[cfg_attr(test, assert_instr(fmaxp))]
pub unsafe fn vmaxvq_f64(a: float64x2_t) -> f64 {
    vmaxvq_f64_(a)
}

/// Horizontal vector min.
#[inline]
#[target_feature(enable = "neon")]
#[cfg_attr(test, assert_instr(sminv))]
pub unsafe fn vminv_s8(a: int8x8_t) -> i8 {
    vminv_s8_(a)
}

/// Horizontal vector min.
#[inline]
#[target_feature(enable = "neon")]
#[cfg_attr(test, assert_instr(sminv))]
pub unsafe fn vminvq_s8(a: int8x16_t) -> i8 {
    vminvq_s8_(a)
}

/// Horizontal vector min.
#[inline]
#[target_feature(enable = "neon")]
#[cfg_attr(test, assert_instr(sminv))]
pub unsafe fn vminv_s16(a: int16x4_t) -> i16 {
    vminv_s16_(a)
}

/// Horizontal vector min.
#[inline]
#[target_feature(enable = "neon")]
#[cfg_attr(test, assert_instr(sminv))]
pub unsafe fn vminvq_s16(a: int16x8_t) -> i16 {
    vminvq_s16_(a)
}

/// Horizontal vector min.
#[inline]
#[target_feature(enable = "neon")]
#[cfg_attr(test, assert_instr(sminp))]
pub unsafe fn vminv_s32(a: int32x2_t) -> i32 {
    vminv_s32_(a)
}

/// Horizontal vector min.
#[inline]
#[target_feature(enable = "neon")]
#[cfg_attr(test, assert_instr(sminv))]
pub unsafe fn vminvq_s32(a: int32x4_t) -> i32 {
    vminvq_s32_(a)
}

/// Horizontal vector min.
#[inline]
#[target_feature(enable = "neon")]
#[cfg_attr(test, assert_instr(uminv))]
pub unsafe fn vminv_u8(a: uint8x8_t) -> u8 {
    vminv_u8_(a)
}

/// Horizontal vector min.
#[inline]
#[target_feature(enable = "neon")]
#[cfg_attr(test, assert_instr(uminv))]
pub unsafe fn vminvq_u8(a: uint8x16_t) -> u8 {
    vminvq_u8_(a)
}

/// Horizontal vector min.
#[inline]
#[target_feature(enable = "neon")]
#[cfg_attr(test, assert_instr(uminv))]
pub unsafe fn vminv_u16(a: uint16x4_t) -> u16 {
    vminv_u16_(a)
}

/// Horizontal vector min.
#[inline]
#[target_feature(enable = "neon")]
#[cfg_attr(test, assert_instr(uminv))]
pub unsafe fn vminvq_u16(a: uint16x8_t) -> u16 {
    vminvq_u16_(a)
}

/// Horizontal vector min.
#[inline]
#[target_feature(enable = "neon")]
#[cfg_attr(test, assert_instr(uminp))]
pub unsafe fn vminv_u32(a: uint32x2_t) -> u32 {
    vminv_u32_(a)
}

/// Horizontal vector min.
#[inline]
#[target_feature(enable = "neon")]
#[cfg_attr(test, assert_instr(uminv))]
pub unsafe fn vminvq_u32(a: uint32x4_t) -> u32 {
    vminvq_u32_(a)
}

/// Horizontal vector min.
#[inline]
#[target_feature(enable = "neon")]
#[cfg_attr(test, assert_instr(fminp))]
pub unsafe fn vminv_f32(a: float32x2_t) -> f32 {
    vminv_f32_(a)
}

/// Horizontal vector min.
#[inline]
#[target_feature(enable = "neon")]
#[cfg_attr(test, assert_instr(fminv))]
pub unsafe fn vminvq_f32(a: float32x4_t) -> f32 {
    vminvq_f32_(a)
}

/// Horizontal vector min.
#[inline]
#[target_feature(enable = "neon")]
#[cfg_attr(test, assert_instr(fminp))]
pub unsafe fn vminvq_f64(a: float64x2_t) -> f64 {
    vminvq_f64_(a)
}

/// Folding minimum of adjacent pairs
#[inline]
#[target_feature(enable = "neon")]
#[cfg_attr(test, assert_instr(sminp))]
pub unsafe fn vpminq_s8(a: int8x16_t, b: int8x16_t) -> int8x16_t {
    vpminq_s8_(a, b)
}

/// Folding minimum of adjacent pairs
#[inline]
#[target_feature(enable = "neon")]
#[cfg_attr(test, assert_instr(sminp))]
pub unsafe fn vpminq_s16(a: int16x8_t, b: int16x8_t) -> int16x8_t {
    vpminq_s16_(a, b)
}

/// Folding minimum of adjacent pairs
#[inline]
#[target_feature(enable = "neon")]
#[cfg_attr(test, assert_instr(sminp))]
pub unsafe fn vpminq_s32(a: int32x4_t, b: int32x4_t) -> int32x4_t {
    vpminq_s32_(a, b)
}

/// Folding minimum of adjacent pairs
#[inline]
#[target_feature(enable = "neon")]
#[cfg_attr(test, assert_instr(uminp))]
pub unsafe fn vpminq_u8(a: uint8x16_t, b: uint8x16_t) -> uint8x16_t {
    vpminq_u8_(a, b)
}

/// Folding minimum of adjacent pairs
#[inline]
#[target_feature(enable = "neon")]
#[cfg_attr(test, assert_instr(uminp))]
pub unsafe fn vpminq_u16(a: uint16x8_t, b: uint16x8_t) -> uint16x8_t {
    vpminq_u16_(a, b)
}

/// Folding minimum of adjacent pairs
#[inline]
#[target_feature(enable = "neon")]
#[cfg_attr(test, assert_instr(uminp))]
pub unsafe fn vpminq_u32(a: uint32x4_t, b: uint32x4_t) -> uint32x4_t {
    vpminq_u32_(a, b)
}

/// Folding minimum of adjacent pairs
#[inline]
#[target_feature(enable = "neon")]
#[cfg_attr(test, assert_instr(fminp))]
pub unsafe fn vpminq_f32(a: float32x4_t, b: float32x4_t) -> float32x4_t {
    vpminq_f32_(a, b)
}

/// Folding minimum of adjacent pairs
#[inline]
#[target_feature(enable = "neon")]
#[cfg_attr(test, assert_instr(fminp))]
pub unsafe fn vpminq_f64(a: float64x2_t, b: float64x2_t) -> float64x2_t {
    vpminq_f64_(a, b)
}

/// Folding maximum of adjacent pairs
#[inline]
#[target_feature(enable = "neon")]
#[cfg_attr(test, assert_instr(smaxp))]
pub unsafe fn vpmaxq_s8(a: int8x16_t, b: int8x16_t) -> int8x16_t {
    vpmaxq_s8_(a, b)
}

/// Folding maximum of adjacent pairs
#[inline]
#[target_feature(enable = "neon")]
#[cfg_attr(test, assert_instr(smaxp))]
pub unsafe fn vpmaxq_s16(a: int16x8_t, b: int16x8_t) -> int16x8_t {
    vpmaxq_s16_(a, b)
}

/// Folding maximum of adjacent pairs
#[inline]
#[target_feature(enable = "neon")]
#[cfg_attr(test, assert_instr(smaxp))]
pub unsafe fn vpmaxq_s32(a: int32x4_t, b: int32x4_t) -> int32x4_t {
    vpmaxq_s32_(a, b)
}

/// Folding maximum of adjacent pairs
#[inline]
#[target_feature(enable = "neon")]
#[cfg_attr(test, assert_instr(umaxp))]
pub unsafe fn vpmaxq_u8(a: uint8x16_t, b: uint8x16_t) -> uint8x16_t {
    vpmaxq_u8_(a, b)
}

/// Folding maximum of adjacent pairs
#[inline]
#[target_feature(enable = "neon")]
#[cfg_attr(test, assert_instr(umaxp))]
pub unsafe fn vpmaxq_u16(a: uint16x8_t, b: uint16x8_t) -> uint16x8_t {
    vpmaxq_u16_(a, b)
}

/// Folding maximum of adjacent pairs
#[inline]
#[target_feature(enable = "neon")]
#[cfg_attr(test, assert_instr(umaxp))]
pub unsafe fn vpmaxq_u32(a: uint32x4_t, b: uint32x4_t) -> uint32x4_t {
    vpmaxq_u32_(a, b)
}

/// Folding maximum of adjacent pairs
#[inline]
#[target_feature(enable = "neon")]
#[cfg_attr(test, assert_instr(fmaxp))]
pub unsafe fn vpmaxq_f32(a: float32x4_t, b: float32x4_t) -> float32x4_t {
    vpmaxq_f32_(a, b)
}

/// Folding maximum of adjacent pairs
#[inline]
#[target_feature(enable = "neon")]
#[cfg_attr(test, assert_instr(fmaxp))]
pub unsafe fn vpmaxq_f64(a: float64x2_t, b: float64x2_t) -> float64x2_t {
    vpmaxq_f64_(a, b)
}


#[cfg(test)]
mod tests {
    use coresimd::aarch64::*;
    use simd::*;
    use std::mem;
    use stdsimd_test::simd_test;

    #[simd_test(enable = "neon")]
    unsafe fn test_vadd_f64() {
        let a = 1.;
        let b = 8.;
        let e = 9.;
        let r: f64 =
            mem::transmute(vadd_f64(mem::transmute(a), mem::transmute(b)));
        assert_eq!(r, e);
    }

    #[simd_test(enable = "neon")]
    unsafe fn test_vaddq_f64() {
        let a = f64x2::new(1., 2.);
        let b = f64x2::new(8., 7.);
        let e = f64x2::new(9., 9.);
        let r: f64x2 = vaddq_f64(a.into_bits(), b.into_bits()).into_bits();
        assert_eq!(r, e);
    }

    #[simd_test(enable = "neon")]
    unsafe fn test_vaddd_s64() {
        let a = 1_i64;
        let b = 8_i64;
        let e = 9_i64;
        let r: i64 =
            mem::transmute(vaddd_s64(mem::transmute(a), mem::transmute(b)));
        assert_eq!(r, e);
    }

    #[simd_test(enable = "neon")]
    unsafe fn test_vaddd_u64() {
        let a = 1_u64;
        let b = 8_u64;
        let e = 9_u64;
        let r: u64 =
            mem::transmute(vaddd_u64(mem::transmute(a), mem::transmute(b)));
        assert_eq!(r, e);
    }

    #[simd_test(enable = "neon")]
    unsafe fn test_vmaxv_s8() {
        let r = vmaxv_s8(i8x8::new(1, 2, 3, 4, -8, 6, 7, 5).into_bits());
        assert_eq!(r, 7_i8);
    }

    #[simd_test(enable = "neon")]
    unsafe fn test_vmaxvq_s8() {
        #[cfg_attr(rustfmt, rustfmt_skip)]
        let r = vmaxvq_s8(i8x16::new(
            1, 2, 3, 4,
            -16, 6, 7, 5,
            8, 1, 1, 1,
            1, 1, 1, 1,
        ).into_bits());
        assert_eq!(r, 8_i8);
    }

    #[simd_test(enable = "neon")]
    unsafe fn test_vmaxv_s16() {
        let r = vmaxv_s16(i16x4::new(1, 2, -4, 3).into_bits());
        assert_eq!(r, 3_i16);
    }

    #[simd_test(enable = "neon")]
    unsafe fn test_vmaxvq_s16() {
        let r = vmaxvq_s16(i16x8::new(1, 2, 7, 4, -16, 6, 7, 5).into_bits());
        assert_eq!(r, 7_i16);
    }

    #[simd_test(enable = "neon")]
    unsafe fn test_vmaxv_s32() {
        let r = vmaxv_s32(i32x2::new(1, -4).into_bits());
        assert_eq!(r, 1_i32);
    }

    #[simd_test(enable = "neon")]
    unsafe fn test_vmaxvq_s32() {
        let r = vmaxvq_s32(i32x4::new(1, 2, -32, 4).into_bits());
        assert_eq!(r, 4_i32);
    }

    #[simd_test(enable = "neon")]
    unsafe fn test_vmaxv_u8() {
        let r = vmaxv_u8(u8x8::new(1, 2, 3, 4, 8, 6, 7, 5).into_bits());
        assert_eq!(r, 8_u8);
    }

    #[simd_test(enable = "neon")]
    unsafe fn test_vmaxvq_u8() {
        #[cfg_attr(rustfmt, rustfmt_skip)]
        let r = vmaxvq_u8(u8x16::new(
            1, 2, 3, 4,
            16, 6, 7, 5,
            8, 1, 1, 1,
            1, 1, 1, 1,
        ).into_bits());
        assert_eq!(r, 16_u8);
    }

    #[simd_test(enable = "neon")]
    unsafe fn test_vmaxv_u16() {
        let r = vmaxv_u16(u16x4::new(1, 2, 4, 3).into_bits());
        assert_eq!(r, 4_u16);
    }

    #[simd_test(enable = "neon")]
    unsafe fn test_vmaxvq_u16() {
        let r = vmaxvq_u16(u16x8::new(1, 2, 7, 4, 16, 6, 7, 5).into_bits());
        assert_eq!(r, 16_u16);
    }

    #[simd_test(enable = "neon")]
    unsafe fn test_vmaxv_u32() {
        let r = vmaxv_u32(u32x2::new(1, 4).into_bits());
        assert_eq!(r, 4_u32);
    }

    #[simd_test(enable = "neon")]
    unsafe fn test_vmaxvq_u32() {
        let r = vmaxvq_u32(u32x4::new(1, 2, 32, 4).into_bits());
        assert_eq!(r, 32_u32);
    }

    #[simd_test(enable = "neon")]
    unsafe fn test_vmaxv_f32() {
        let r = vmaxv_f32(f32x2::new(1., 4.).into_bits());
        assert_eq!(r, 4_f32);
    }

    #[simd_test(enable = "neon")]
    unsafe fn test_vmaxvq_f32() {
        let r = vmaxvq_f32(f32x4::new(1., 2., 32., 4.).into_bits());
        assert_eq!(r, 32_f32);
    }

    #[simd_test(enable = "neon")]
    unsafe fn test_vmaxvq_f64() {
        let r = vmaxvq_f64(f64x2::new(1., 4.).into_bits());
        assert_eq!(r, 4_f64);
    }

    #[simd_test(enable = "neon")]
    unsafe fn test_vminv_s8() {
        let r = vminv_s8(i8x8::new(1, 2, 3, 4, -8, 6, 7, 5).into_bits());
        assert_eq!(r, -8_i8);
    }

    #[simd_test(enable = "neon")]
    unsafe fn test_vminvq_s8() {
        #[cfg_attr(rustfmt, rustfmt_skip)]
        let r = vminvq_s8(i8x16::new(
            1, 2, 3, 4,
            -16, 6, 7, 5,
            8, 1, 1, 1,
            1, 1, 1, 1,
        ).into_bits());
        assert_eq!(r, -16_i8);
    }

    #[simd_test(enable = "neon")]
    unsafe fn test_vminv_s16() {
        let r = vminv_s16(i16x4::new(1, 2, -4, 3).into_bits());
        assert_eq!(r, -4_i16);
    }

    #[simd_test(enable = "neon")]
    unsafe fn test_vminvq_s16() {
        let r = vminvq_s16(i16x8::new(1, 2, 7, 4, -16, 6, 7, 5).into_bits());
        assert_eq!(r, -16_i16);
    }

    #[simd_test(enable = "neon")]
    unsafe fn test_vminv_s32() {
        let r = vminv_s32(i32x2::new(1, -4).into_bits());
        assert_eq!(r, -4_i32);
    }

    #[simd_test(enable = "neon")]
    unsafe fn test_vminvq_s32() {
        let r = vminvq_s32(i32x4::new(1, 2, -32, 4).into_bits());
        assert_eq!(r, -32_i32);
    }

    #[simd_test(enable = "neon")]
    unsafe fn test_vminv_u8() {
        let r = vminv_u8(u8x8::new(1, 2, 3, 4, 8, 6, 7, 5).into_bits());
        assert_eq!(r, 1_u8);
    }

    #[simd_test(enable = "neon")]
    unsafe fn test_vminvq_u8() {
        #[cfg_attr(rustfmt, rustfmt_skip)]
        let r = vminvq_u8(u8x16::new(
            1, 2, 3, 4,
            16, 6, 7, 5,
            8, 1, 1, 1,
            1, 1, 1, 1,
        ).into_bits());
        assert_eq!(r, 1_u8);
    }

    #[simd_test(enable = "neon")]
    unsafe fn test_vminv_u16() {
        let r = vminv_u16(u16x4::new(1, 2, 4, 3).into_bits());
        assert_eq!(r, 1_u16);
    }

    #[simd_test(enable = "neon")]
    unsafe fn test_vminvq_u16() {
        let r = vminvq_u16(u16x8::new(1, 2, 7, 4, 16, 6, 7, 5).into_bits());
        assert_eq!(r, 1_u16);
    }

    #[simd_test(enable = "neon")]
    unsafe fn test_vminv_u32() {
        let r = vminv_u32(u32x2::new(1, 4).into_bits());
        assert_eq!(r, 1_u32);
    }

    #[simd_test(enable = "neon")]
    unsafe fn test_vminvq_u32() {
        let r = vminvq_u32(u32x4::new(1, 2, 32, 4).into_bits());
        assert_eq!(r, 1_u32);
    }

    #[simd_test(enable = "neon")]
    unsafe fn test_vminv_f32() {
        let r = vminv_f32(f32x2::new(1., 4.).into_bits());
        assert_eq!(r, 1_f32);
    }

    #[simd_test(enable = "neon")]
    unsafe fn test_vminvq_f32() {
        let r = vminvq_f32(f32x4::new(1., 2., 32., 4.).into_bits());
        assert_eq!(r, 1_f32);
    }

    #[simd_test(enable = "neon")]
    unsafe fn test_vminvq_f64() {
        let r = vminvq_f64(f64x2::new(1., 4.).into_bits());
        assert_eq!(r, 1_f64);
    }

    #[simd_test(enable = "neon")]
    unsafe fn test_vpminq_s8() {
        #[cfg_attr(rustfmt, skip)]
        let a = i8x16::new(
            1, -2, 3, -4, 5, 6, 7, 8,
            1, 2, 3, 4, 5, 6, 7, 8
        );
        #[cfg_attr(rustfmt, skip)]
        let b = i8x16::new(
            0, 3, 2, 5, 4, 7, 6, 9,
            0, 3, 2, 5, 4, 7, 6, 9
        );
        #[cfg_attr(rustfmt, skip)]
        let e = i8x16::new(
            -2, -4, 5, 7, 1, 3, 5, 7,
            0, 2, 4, 6, 0, 2, 4, 6,
        );
        let r: i8x16 = vpminq_s8(a.into_bits(), b.into_bits()).into_bits();
        assert_eq!(r, e);
    }

    #[simd_test(enable = "neon")]
    unsafe fn test_vpminq_s16() {
        let a = i16x8::new(1, -2, 3, 4, 5, 6, 7, 8);
        let b = i16x8::new(0, 3, 2, 5, 4, 7, 6, 9);
        let e = i16x8::new(-2, 3, 5, 7, 0, 2, 4, 6);
        let r: i16x8 = vpminq_s16(a.into_bits(), b.into_bits()).into_bits();
        assert_eq!(r, e);
    }

    #[simd_test(enable = "neon")]
    unsafe fn test_vpminq_s32() {
        let a = i32x4::new(1, -2, 3, 4);
        let b = i32x4::new(0, 3, 2, 5);
        let e = i32x4::new(-2, 3, 0, 2);
        let r: i32x4 = vpminq_s32(a.into_bits(), b.into_bits()).into_bits();
        assert_eq!(r, e);
    }

    #[simd_test(enable = "neon")]
    unsafe fn test_vpminq_u8() {
        #[cfg_attr(rustfmt, skip)]
        let a = u8x16::new(
            1, 2, 3, 4, 5, 6, 7, 8,
            1, 2, 3, 4, 5, 6, 7, 8
        );
        #[cfg_attr(rustfmt, skip)]
        let b = u8x16::new(
            0, 3, 2, 5, 4, 7, 6, 9,
            0, 3, 2, 5, 4, 7, 6, 9
        );
        #[cfg_attr(rustfmt, skip)]
        let e = u8x16::new(
            1, 3, 5, 7, 1, 3, 5, 7,
            0, 2, 4, 6, 0, 2, 4, 6,
        );
        let r: u8x16 = vpminq_u8(a.into_bits(), b.into_bits()).into_bits();
        assert_eq!(r, e);
    }

    #[simd_test(enable = "neon")]
    unsafe fn test_vpminq_u16() {
        let a = u16x8::new(1, 2, 3, 4, 5, 6, 7, 8);
        let b = u16x8::new(0, 3, 2, 5, 4, 7, 6, 9);
        let e = u16x8::new(1, 3, 5, 7, 0, 2, 4, 6);
        let r: u16x8 = vpminq_u16(a.into_bits(), b.into_bits()).into_bits();
        assert_eq!(r, e);
    }

    #[simd_test(enable = "neon")]
    unsafe fn test_vpminq_u32() {
        let a = u32x4::new(1, 2, 3, 4);
        let b = u32x4::new(0, 3, 2, 5);
        let e = u32x4::new(1, 3, 0, 2);
        let r: u32x4 = vpminq_u32(a.into_bits(), b.into_bits()).into_bits();
        assert_eq!(r, e);
    }

    #[simd_test(enable = "neon")]
    unsafe fn test_vpmin_f32() {
        let a = f32x4::new(1., -2., 3., 4.);
        let b = f32x4::new(0., 3., 2., 5.);
        let e = f32x4::new(-2., 3., 0., 2.);
        let r: f32x4 = vpminq_f32(a.into_bits(), b.into_bits()).into_bits();
        assert_eq!(r, e);
    }

    #[simd_test(enable = "neon")]
    unsafe fn test_vpmin_f64() {
        let a = f64x2::new(1., -2.);
        let b = f64x2::new(0., 3.);
        let e = f64x2::new(-2., 0.);
        let r: f64x2 = vpminq_f64(a.into_bits(), b.into_bits()).into_bits();
        assert_eq!(r, e);
    }

    #[simd_test(enable = "neon")]
    unsafe fn test_vpmaxq_s8() {
        #[cfg_attr(rustfmt, skip)]
        let a = i8x16::new(
            1, -2, 3, -4, 5, 6, 7, 8,
            1, 2, 3, 4, 5, 6, 7, 8
        );
        #[cfg_attr(rustfmt, skip)]
        let b = i8x16::new(
            0, 3, 2, 5, 4, 7, 6, 9,
            0, 3, 2, 5, 4, 7, 6, 9
        );
        #[cfg_attr(rustfmt, skip)]
        let e = i8x16::new(
            1, 3, 6, 8, 2, 4, 6, 8,
            3, 5, 7, 9, 3, 5, 7, 9,
        );
        let r: i8x16 = vpmaxq_s8(a.into_bits(), b.into_bits()).into_bits();
        assert_eq!(r, e);
    }

    #[simd_test(enable = "neon")]
    unsafe fn test_vpmaxq_s16() {
        let a = i16x8::new(1, -2, 3, 4, 5, 6, 7, 8);
        let b = i16x8::new(0, 3, 2, 5, 4, 7, 6, 9);
        let e = i16x8::new(1, 4, 6, 8, 3, 5, 7, 9);
        let r: i16x8 = vpmaxq_s16(a.into_bits(), b.into_bits()).into_bits();
        assert_eq!(r, e);
    }

    #[simd_test(enable = "neon")]
    unsafe fn test_vpmaxq_s32() {
        let a = i32x4::new(1, -2, 3, 4);
        let b = i32x4::new(0, 3, 2, 5);
        let e = i32x4::new(1, 4, 3, 5);
        let r: i32x4 = vpmaxq_s32(a.into_bits(), b.into_bits()).into_bits();
        assert_eq!(r, e);
    }

    #[simd_test(enable = "neon")]
    unsafe fn test_vpmaxq_u8() {
        #[cfg_attr(rustfmt, skip)]
        let a = u8x16::new(
            1, 2, 3, 4, 5, 6, 7, 8,
            1, 2, 3, 4, 5, 6, 7, 8
        );
        #[cfg_attr(rustfmt, skip)]
        let b = u8x16::new(
            0, 3, 2, 5, 4, 7, 6, 9,
            0, 3, 2, 5, 4, 7, 6, 9
        );
        #[cfg_attr(rustfmt, skip)]
        let e = u8x16::new(
            2, 4, 6, 8, 2, 4, 6, 8,
            3, 5, 7, 9, 3, 5, 7, 9,
        );
        let r: u8x16 = vpmaxq_u8(a.into_bits(), b.into_bits()).into_bits();
        assert_eq!(r, e);
    }

    #[simd_test(enable = "neon")]
    unsafe fn test_vpmaxq_u16() {
        let a = u16x8::new(1, 2, 3, 4, 5, 6, 7, 8);
        let b = u16x8::new(0, 3, 2, 5, 4, 7, 6, 9);
        let e = u16x8::new(2, 4, 6, 8, 3, 5, 7, 9);
        let r: u16x8 = vpmaxq_u16(a.into_bits(), b.into_bits()).into_bits();
        assert_eq!(r, e);
    }

    #[simd_test(enable = "neon")]
    unsafe fn test_vpmaxq_u32() {
        let a = u32x4::new(1, 2, 3, 4);
        let b = u32x4::new(0, 3, 2, 5);
        let e = u32x4::new(2, 4, 3, 5);
        let r: u32x4 = vpmaxq_u32(a.into_bits(), b.into_bits()).into_bits();
        assert_eq!(r, e);
    }

    #[simd_test(enable = "neon")]
    unsafe fn test_vpmax_f32() {
        let a = f32x4::new(1., -2., 3., 4.);
        let b = f32x4::new(0., 3., 2., 5.);
        let e = f32x4::new(1., 4., 3., 5.);
        let r: f32x4 = vpmaxq_f32(a.into_bits(), b.into_bits()).into_bits();
        assert_eq!(r, e);
    }

    #[simd_test(enable = "neon")]
    unsafe fn test_vpmax_f64() {
        let a = f64x2::new(1., -2.);
        let b = f64x2::new(0., 3.);
        let e = f64x2::new(1., 3.);
        let r: f64x2 = vpmaxq_f64(a.into_bits(), b.into_bits()).into_bits();
        assert_eq!(r, e);
    }
}
