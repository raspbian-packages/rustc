//! PowerPC AltiVec intrinsics.
//!
//! AltiVec is a brandname trademarked by Freescale (previously Motorola) for
//! the standard `Category:Vector` part of the Power ISA v.2.03 specification.
//! This Category is also known as VMX (used by IBM), and "Velocity Engine" (a
//! brand name previously used by Apple).
//!
//! The references are: [POWER ISA v2.07B (for POWER8 & POWER8 with NVIDIA
//! NVlink)] and [POWER ISA v3.0B (for POWER9)].
//!
//! [POWER ISA v2.07B (for POWER8 & POWER8 with NVIDIA NVlink)]: https://ibm.box.com/s/jd5w15gz301s5b5dt375mshpq9c3lh4u
//! [POWER ISA v3.0B (for POWER9)]: https://ibm.box.com/s/1hzcwkwf8rbju5h9iyf44wm94amnlcrv

#![allow(non_camel_case_types)]

use coresimd::simd::*;
use coresimd::simd_llvm::*;

use mem;

#[cfg(test)]
use stdsimd_test::assert_instr;

types! {
    /// PowerPC-specific 128-bit wide vector of sixteen packed `i8`
    pub struct vector_signed_char(i8, i8, i8, i8, i8, i8, i8, i8,
                                  i8, i8, i8, i8, i8, i8, i8, i8);
    /// PowerPC-specific 128-bit wide vector of sixteen packed `u8`
    pub struct vector_unsigned_char(u8, u8, u8, u8, u8, u8, u8, u8,
                                    u8, u8, u8, u8, u8, u8, u8, u8);

    /// PowerPC-specific 128-bit wide vector mask of sixteen packed elements
    pub struct vector_bool_char(i8, i8, i8, i8, i8, i8, i8, i8,
                                i8, i8, i8, i8, i8, i8, i8, i8);
    /// PowerPC-specific 128-bit wide vector of eight packed `i16`
    pub struct vector_signed_short(i16, i16, i16, i16, i16, i16, i16, i16);
    /// PowerPC-specific 128-bit wide vector of eight packed `u16`
    pub struct vector_unsigned_short(u16, u16, u16, u16, u16, u16, u16, u16);
    /// PowerPC-specific 128-bit wide vector mask of eight packed elements
    pub struct vector_bool_short(i16, i16, i16, i16, i16, i16, i16, i16);
    // pub struct vector_pixel(???);
    /// PowerPC-specific 128-bit wide vector of four packed `i32`
    pub struct vector_signed_int(i32, i32, i32, i32);
    /// PowerPC-specific 128-bit wide vector of four packed `u32`
    pub struct vector_unsigned_int(u32, u32, u32, u32);
    /// PowerPC-specific 128-bit wide vector mask of four packed elements
    pub struct vector_bool_int(i32, i32, i32, i32);
    /// PowerPC-specific 128-bit wide vector of four packed `f32`
    pub struct vector_float(f32, f32, f32, f32);
}

#[allow(improper_ctypes)]
extern "C" {
    #[link_name = "llvm.ppc.altivec.vperm"]
    fn vperm(
        a: vector_signed_int, b: vector_signed_int, c: vector_unsigned_char,
    ) -> vector_signed_int;
    #[link_name = "llvm.ppc.altivec.vmhaddshs"]
    fn vmhaddshs(
        a: vector_signed_short, b: vector_signed_short, c: vector_signed_short,
    ) -> vector_signed_short;
    #[link_name = "llvm.ppc.altivec.vmhraddshs"]
    fn vmhraddshs(
        a: vector_signed_short, b: vector_signed_short, c: vector_signed_short,
    ) -> vector_signed_short;
    #[link_name = "llvm.ppc.altivec.vmsumuhs"]
    fn vmsumuhs(
        a: vector_unsigned_short, b: vector_unsigned_short,
        c: vector_unsigned_int,
    ) -> vector_unsigned_int;
    #[link_name = "llvm.ppc.altivec.vmsumshs"]
    fn vmsumshs(
        a: vector_signed_short, b: vector_signed_short, c: vector_signed_int,
    ) -> vector_signed_int;
    #[link_name = "llvm.ppc.altivec.vmsumubm"]
    fn vmsumubm(
        a: vector_unsigned_char, b: vector_unsigned_char,
        c: vector_unsigned_int,
    ) -> vector_unsigned_int;
    #[link_name = "llvm.ppc.altivec.vmsummbm"]
    fn vmsummbm(
        a: vector_signed_char, b: vector_unsigned_char, c: vector_signed_int,
    ) -> vector_signed_int;
    #[link_name = "llvm.ppc.altivec.vmsumuhm"]
    fn vmsumuhm(
        a: vector_unsigned_short, b: vector_unsigned_short,
        c: vector_unsigned_int,
    ) -> vector_unsigned_int;
    #[link_name = "llvm.ppc.altivec.vmsumshm"]
    fn vmsumshm(
        a: vector_signed_short, b: vector_signed_short, c: vector_signed_int,
    ) -> vector_signed_int;
    #[link_name = "llvm.ppc.altivec.vmaddfp"]
    fn vmaddfp(
        a: vector_float, b: vector_float, c: vector_float,
    ) -> vector_float;
    #[link_name = "llvm.ppc.altivec.vnmsubfp"]
    fn vnmsubfp(
        a: vector_float, b: vector_float, c: vector_float,
    ) -> vector_float;
    #[link_name = "llvm.ppc.altivec.vsum2sws"]
    fn vsum2sws(
        a: vector_signed_int, b: vector_signed_int,
    ) -> vector_signed_int;
    #[link_name = "llvm.ppc.altivec.vsum4ubs"]
    fn vsum4ubs(
        a: vector_unsigned_char, b: vector_unsigned_int,
    ) -> vector_unsigned_int;
    #[link_name = "llvm.ppc.altivec.vsum4sbs"]
    fn vsum4sbs(
        a: vector_signed_char, b: vector_signed_int,
    ) -> vector_signed_int;
    #[link_name = "llvm.ppc.altivec.vsum4shs"]
    fn vsum4shs(
        a: vector_signed_short, b: vector_signed_int,
    ) -> vector_signed_int;
    #[link_name = "llvm.ppc.altivec.vmuleub"]
    fn vmuleub(
        a: vector_unsigned_char, b: vector_unsigned_char,
    ) -> vector_unsigned_short;
    #[link_name = "llvm.ppc.altivec.vmulesb"]
    fn vmulesb(
        a: vector_signed_char, b: vector_signed_char,
    ) -> vector_signed_short;
    #[link_name = "llvm.ppc.altivec.vmuleuh"]
    fn vmuleuh(
        a: vector_unsigned_short, b: vector_unsigned_short,
    ) -> vector_unsigned_int;
    #[link_name = "llvm.ppc.altivec.vmulesh"]
    fn vmulesh(
        a: vector_signed_short, b: vector_signed_short,
    ) -> vector_signed_int;
    #[link_name = "llvm.ppc.altivec.vmuloub"]
    fn vmuloub(
        a: vector_unsigned_char, b: vector_unsigned_char,
    ) -> vector_unsigned_short;
    #[link_name = "llvm.ppc.altivec.vmulosb"]
    fn vmulosb(
        a: vector_signed_char, b: vector_signed_char,
    ) -> vector_signed_short;
    #[link_name = "llvm.ppc.altivec.vmulouh"]
    fn vmulouh(
        a: vector_unsigned_short, b: vector_unsigned_short,
    ) -> vector_unsigned_int;
    #[link_name = "llvm.ppc.altivec.vmulosh"]
    fn vmulosh(
        a: vector_signed_short, b: vector_signed_short,
    ) -> vector_signed_int;
}

mod sealed {

    use super::*;

    #[inline]
    #[target_feature(enable = "altivec")]
    #[cfg_attr(test, assert_instr(vmuleub))]
    unsafe fn vec_vmuleub(
        a: vector_unsigned_char, b: vector_unsigned_char,
    ) -> vector_unsigned_short {
        vmuleub(a, b)
    }
    #[inline]
    #[target_feature(enable = "altivec")]
    #[cfg_attr(test, assert_instr(vmulesb))]
    unsafe fn vec_vmulesb(
        a: vector_signed_char, b: vector_signed_char,
    ) -> vector_signed_short {
        vmulesb(a, b)
    }
    #[inline]
    #[target_feature(enable = "altivec")]
    #[cfg_attr(test, assert_instr(vmuleuh))]
    unsafe fn vec_vmuleuh(
        a: vector_unsigned_short, b: vector_unsigned_short,
    ) -> vector_unsigned_int {
        vmuleuh(a, b)
    }
    #[inline]
    #[target_feature(enable = "altivec")]
    #[cfg_attr(test, assert_instr(vmulesh))]
    unsafe fn vec_vmulesh(
        a: vector_signed_short, b: vector_signed_short,
    ) -> vector_signed_int {
        vmulesh(a, b)
    }

    pub trait VectorMule<Result> {
        unsafe fn vec_mule(self, b: Self) -> Result;
    }

    impl VectorMule<vector_unsigned_short> for vector_unsigned_char {
        #[inline]
        #[target_feature(enable = "altivec")]
        unsafe fn vec_mule(self, b: Self) -> vector_unsigned_short {
            vmuleub(self, b)
        }
    }
    impl VectorMule<vector_signed_short> for vector_signed_char {
        #[inline]
        #[target_feature(enable = "altivec")]
        unsafe fn vec_mule(self, b: Self) -> vector_signed_short {
            vmulesb(self, b)
        }
    }
    impl VectorMule<vector_unsigned_int> for vector_unsigned_short {
        #[inline]
        #[target_feature(enable = "altivec")]
        unsafe fn vec_mule(self, b: Self) -> vector_unsigned_int {
            vmuleuh(self, b)
        }
    }
    impl VectorMule<vector_signed_int> for vector_signed_short {
        #[inline]
        #[target_feature(enable = "altivec")]
        unsafe fn vec_mule(self, b: Self) -> vector_signed_int {
            vmulesh(self, b)
        }
    }

    #[inline]
    #[target_feature(enable = "altivec")]
    #[cfg_attr(test, assert_instr(vmuloub))]
    unsafe fn vec_vmuloub(
        a: vector_unsigned_char, b: vector_unsigned_char,
    ) -> vector_unsigned_short {
        vmuloub(a, b)
    }
    #[inline]
    #[target_feature(enable = "altivec")]
    #[cfg_attr(test, assert_instr(vmulosb))]
    unsafe fn vec_vmulosb(
        a: vector_signed_char, b: vector_signed_char,
    ) -> vector_signed_short {
        vmulosb(a, b)
    }
    #[inline]
    #[target_feature(enable = "altivec")]
    #[cfg_attr(test, assert_instr(vmulouh))]
    unsafe fn vec_vmulouh(
        a: vector_unsigned_short, b: vector_unsigned_short,
    ) -> vector_unsigned_int {
        vmulouh(a, b)
    }
    #[inline]
    #[target_feature(enable = "altivec")]
    #[cfg_attr(test, assert_instr(vmulosh))]
    unsafe fn vec_vmulosh(
        a: vector_signed_short, b: vector_signed_short,
    ) -> vector_signed_int {
        vmulosh(a, b)
    }

    pub trait VectorMulo<Result> {
        unsafe fn vec_mulo(self, b: Self) -> Result;
    }

    impl VectorMulo<vector_unsigned_short> for vector_unsigned_char {
        #[inline]
        #[target_feature(enable = "altivec")]
        unsafe fn vec_mulo(self, b: Self) -> vector_unsigned_short {
            vmuloub(self, b)
        }
    }
    impl VectorMulo<vector_signed_short> for vector_signed_char {
        #[inline]
        #[target_feature(enable = "altivec")]
        unsafe fn vec_mulo(self, b: Self) -> vector_signed_short {
            vmulosb(self, b)
        }
    }
    impl VectorMulo<vector_unsigned_int> for vector_unsigned_short {
        #[inline]
        #[target_feature(enable = "altivec")]
        unsafe fn vec_mulo(self, b: Self) -> vector_unsigned_int {
            vmulouh(self, b)
        }
    }
    impl VectorMulo<vector_signed_int> for vector_signed_short {
        #[inline]
        #[target_feature(enable = "altivec")]
        unsafe fn vec_mulo(self, b: Self) -> vector_signed_int {
            vmulosh(self, b)
        }
    }

    #[inline]
    #[target_feature(enable = "altivec")]
    #[cfg_attr(test, assert_instr(vsum4ubs))]
    unsafe fn vec_vsum4ubs(
        a: vector_unsigned_char, b: vector_unsigned_int,
    ) -> vector_unsigned_int {
        vsum4ubs(a, b)
    }

    #[inline]
    #[target_feature(enable = "altivec")]
    #[cfg_attr(test, assert_instr(vsum4sbs))]
    unsafe fn vec_vsum4sbs(
        a: vector_signed_char, b: vector_signed_int,
    ) -> vector_signed_int {
        vsum4sbs(a, b)
    }

    #[inline]
    #[target_feature(enable = "altivec")]
    #[cfg_attr(test, assert_instr(vsum4shs))]
    unsafe fn vec_vsum4shs(
        a: vector_signed_short, b: vector_signed_int,
    ) -> vector_signed_int {
        vsum4shs(a, b)
    }

    pub trait VectorSum4s<Other> {
        unsafe fn vec_sum4s(self, b: Other) -> Other;
    }

    impl VectorSum4s<vector_unsigned_int> for vector_unsigned_char {
        #[inline]
        #[target_feature(enable = "altivec")]
        unsafe fn vec_sum4s(
            self, b: vector_unsigned_int,
        ) -> vector_unsigned_int {
            vsum4ubs(self, b)
        }
    }

    impl VectorSum4s<vector_signed_int> for vector_signed_char {
        #[inline]
        #[target_feature(enable = "altivec")]
        unsafe fn vec_sum4s(self, b: vector_signed_int) -> vector_signed_int {
            vsum4sbs(self, b)
        }
    }

    impl VectorSum4s<vector_signed_int> for vector_signed_short {
        #[inline]
        #[target_feature(enable = "altivec")]
        unsafe fn vec_sum4s(self, b: vector_signed_int) -> vector_signed_int {
            vsum4shs(self, b)
        }
    }

    #[inline]
    #[target_feature(enable = "altivec")]
    #[cfg_attr(test, assert_instr(vsum2sws))]
    unsafe fn vec_vsum2sws(
        a: vector_signed_int, b: vector_signed_int,
    ) -> vector_signed_int {
        vsum2sws(a, b)
    }

    #[inline]
    #[target_feature(enable = "altivec")]
    #[cfg_attr(test, assert_instr(vnmsubfp))]
    unsafe fn vec_vnmsubfp(
        a: vector_float, b: vector_float, c: vector_float,
    ) -> vector_float {
        vnmsubfp(a, b, c)
    }

    #[inline]
    #[target_feature(enable = "altivec")]
    #[cfg_attr(test, assert_instr(vmaddfp))]
    unsafe fn vec_vmaddfp(
        a: vector_float, b: vector_float, c: vector_float,
    ) -> vector_float {
        vmaddfp(a, b, c)
    }

    #[inline]
    #[target_feature(enable = "altivec")]
    #[cfg_attr(test, assert_instr(vmsumubm))]
    unsafe fn vec_vmsumubm(
        a: vector_unsigned_char, b: vector_unsigned_char,
        c: vector_unsigned_int,
    ) -> vector_unsigned_int {
        vmsumubm(a, b, c)
    }

    #[inline]
    #[target_feature(enable = "altivec")]
    #[cfg_attr(test, assert_instr(vmsummbm))]
    unsafe fn vec_vmsummbm(
        a: vector_signed_char, b: vector_unsigned_char, c: vector_signed_int,
    ) -> vector_signed_int {
        vmsummbm(a, b, c)
    }

    #[inline]
    #[target_feature(enable = "altivec")]
    #[cfg_attr(test, assert_instr(vmsumuhm))]
    unsafe fn vec_vmsumuhm(
        a: vector_unsigned_short, b: vector_unsigned_short,
        c: vector_unsigned_int,
    ) -> vector_unsigned_int {
        vmsumuhm(a, b, c)
    }

    #[inline]
    #[target_feature(enable = "altivec")]
    #[cfg_attr(test, assert_instr(vmsumshm))]
    unsafe fn vec_vmsumshm(
        a: vector_signed_short, b: vector_signed_short, c: vector_signed_int,
    ) -> vector_signed_int {
        vmsumshm(a, b, c)
    }

    pub trait VectorMsum<B, Other> {
        unsafe fn vec_msum(self, b: B, c: Other) -> Other;
    }

    impl VectorMsum<vector_unsigned_char, vector_unsigned_int>
        for vector_unsigned_char
    {
        #[inline]
        #[target_feature(enable = "altivec")]
        unsafe fn vec_msum(
            self, b: vector_unsigned_char, c: vector_unsigned_int,
        ) -> vector_unsigned_int {
            vmsumubm(self, b, c)
        }
    }

    impl VectorMsum<vector_unsigned_char, vector_signed_int>
        for vector_signed_char
    {
        #[inline]
        #[target_feature(enable = "altivec")]
        unsafe fn vec_msum(
            self, b: vector_unsigned_char, c: vector_signed_int,
        ) -> vector_signed_int {
            vmsummbm(self, b, c)
        }
    }

    impl VectorMsum<vector_unsigned_short, vector_unsigned_int>
        for vector_unsigned_short
    {
        #[inline]
        #[target_feature(enable = "altivec")]
        unsafe fn vec_msum(
            self, b: vector_unsigned_short, c: vector_unsigned_int,
        ) -> vector_unsigned_int {
            vmsumuhm(self, b, c)
        }
    }

    impl VectorMsum<vector_signed_short, vector_signed_int>
        for vector_signed_short
    {
        #[inline]
        #[target_feature(enable = "altivec")]
        unsafe fn vec_msum(
            self, b: vector_signed_short, c: vector_signed_int,
        ) -> vector_signed_int {
            vmsumshm(self, b, c)
        }
    }

    #[inline]
    #[target_feature(enable = "altivec")]
    #[cfg_attr(test, assert_instr(vmsumuhs))]
    unsafe fn vec_vmsumuhs(
        a: vector_unsigned_short, b: vector_unsigned_short,
        c: vector_unsigned_int,
    ) -> vector_unsigned_int {
        vmsumuhs(a, b, c)
    }

    #[inline]
    #[target_feature(enable = "altivec")]
    #[cfg_attr(test, assert_instr(vmsumshs))]
    unsafe fn vec_vmsumshs(
        a: vector_signed_short, b: vector_signed_short, c: vector_signed_int,
    ) -> vector_signed_int {
        vmsumshs(a, b, c)
    }

    pub trait VectorMsums<Other> {
        unsafe fn vec_msums(self, b: Self, c: Other) -> Other;
    }

    impl VectorMsums<vector_unsigned_int> for vector_unsigned_short {
        #[inline]
        #[target_feature(enable = "altivec")]
        unsafe fn vec_msums(
            self, b: Self, c: vector_unsigned_int,
        ) -> vector_unsigned_int {
            vmsumuhs(self, b, c)
        }
    }

    impl VectorMsums<vector_signed_int> for vector_signed_short {
        #[inline]
        #[target_feature(enable = "altivec")]
        unsafe fn vec_msums(
            self, b: Self, c: vector_signed_int,
        ) -> vector_signed_int {
            vmsumshs(self, b, c)
        }
    }

    #[inline]
    #[target_feature(enable = "altivec")]
    #[cfg_attr(test, assert_instr(vperm))]
    unsafe fn vec_vperm(
        a: vector_signed_int, b: vector_signed_int, c: vector_unsigned_char,
    ) -> vector_signed_int {
        vperm(a, b, c)
    }

    pub trait VectorPerm {
        unsafe fn vec_vperm(self, b: Self, c: vector_unsigned_char) -> Self;
    }

    macro_rules! vector_perm {
        {$impl: ident} => {
            impl VectorPerm for $impl {
            #[inline]
            #[target_feature(enable = "altivec")]
            unsafe fn vec_vperm(self, b: Self, c: vector_unsigned_char) -> Self {
                    mem::transmute(vec_vperm(mem::transmute(self), mem::transmute(b), c))
                }
            }
        }
    }

    vector_perm!{ vector_signed_char }
    vector_perm!{ vector_unsigned_char }
    vector_perm!{ vector_bool_char }

    vector_perm!{ vector_signed_short }
    vector_perm!{ vector_unsigned_short }
    vector_perm!{ vector_bool_short }

    vector_perm!{ vector_signed_int }
    vector_perm!{ vector_unsigned_int }
    vector_perm!{ vector_bool_int }

    vector_perm!{ vector_float }

    pub trait VectorAdd<Other> {
        type Result;
        unsafe fn vec_add(self, other: Other) -> Self::Result;
    }

    #[inline]
    #[target_feature(enable = "altivec")]
    #[cfg_attr(test, assert_instr(vaddubm))]
    pub unsafe fn vec_add_bc_sc(
        a: vector_bool_char, b: vector_signed_char,
    ) -> vector_signed_char {
        simd_add(::mem::transmute(a), b)
    }
    impl VectorAdd<vector_signed_char> for vector_bool_char {
        type Result = vector_signed_char;
        #[inline]
        #[target_feature(enable = "altivec")]
        unsafe fn vec_add(self, other: vector_signed_char) -> Self::Result {
            vec_add_bc_sc(self, other)
        }
    }
    impl VectorAdd<vector_bool_char> for vector_signed_char {
        type Result = vector_signed_char;
        #[inline]
        #[target_feature(enable = "altivec")]
        unsafe fn vec_add(self, other: vector_bool_char) -> Self::Result {
            other.vec_add(self)
        }
    }

    #[inline]
    #[target_feature(enable = "altivec")]
    #[cfg_attr(test, assert_instr(vaddubm))]
    pub unsafe fn vec_add_sc_sc(
        a: vector_signed_char, b: vector_signed_char,
    ) -> vector_signed_char {
        simd_add(a, b)
    }
    impl VectorAdd<vector_signed_char> for vector_signed_char {
        type Result = vector_signed_char;
        #[inline]
        #[target_feature(enable = "altivec")]
        unsafe fn vec_add(self, other: vector_signed_char) -> Self::Result {
            vec_add_sc_sc(self, other)
        }
    }

    #[inline]
    #[target_feature(enable = "altivec")]
    #[cfg_attr(test, assert_instr(vaddubm))]
    pub unsafe fn vec_add_bc_uc(
        a: vector_bool_char, b: vector_unsigned_char,
    ) -> vector_unsigned_char {
        simd_add(::mem::transmute(a), b)
    }
    impl VectorAdd<vector_unsigned_char> for vector_bool_char {
        type Result = vector_unsigned_char;
        #[inline]
        #[target_feature(enable = "altivec")]
        unsafe fn vec_add(self, other: vector_unsigned_char) -> Self::Result {
            vec_add_bc_uc(self, other)
        }
    }
    impl VectorAdd<vector_bool_char> for vector_unsigned_char {
        type Result = vector_unsigned_char;
        #[inline]
        #[target_feature(enable = "altivec")]
        unsafe fn vec_add(self, other: vector_bool_char) -> Self::Result {
            other.vec_add(self)
        }
    }

    #[inline]
    #[target_feature(enable = "altivec")]
    #[cfg_attr(test, assert_instr(vaddubm))]
    pub unsafe fn vec_add_uc_uc(
        a: vector_unsigned_char, b: vector_unsigned_char,
    ) -> vector_unsigned_char {
        simd_add(a, b)
    }
    impl VectorAdd<vector_unsigned_char> for vector_unsigned_char {
        type Result = vector_unsigned_char;
        #[inline]
        #[target_feature(enable = "altivec")]
        unsafe fn vec_add(self, other: vector_unsigned_char) -> Self::Result {
            vec_add_uc_uc(self, other)
        }
    }

    #[inline]
    #[target_feature(enable = "altivec")]
    #[cfg_attr(test, assert_instr(vadduhm))]
    pub unsafe fn vec_add_bs_ss(
        a: vector_bool_short, b: vector_signed_short,
    ) -> vector_signed_short {
        let a: i16x8 = ::mem::transmute(a);
        let a: vector_signed_short = simd_cast(a);
        simd_add(a, b)
    }

    impl VectorAdd<vector_signed_short> for vector_bool_short {
        type Result = vector_signed_short;
        #[inline]
        #[target_feature(enable = "altivec")]
        unsafe fn vec_add(self, other: vector_signed_short) -> Self::Result {
            vec_add_bs_ss(self, other)
        }
    }
    impl VectorAdd<vector_bool_short> for vector_signed_short {
        type Result = vector_signed_short;
        #[inline]
        #[target_feature(enable = "altivec")]
        unsafe fn vec_add(self, other: vector_bool_short) -> Self::Result {
            other.vec_add(self)
        }
    }

    #[inline]
    #[target_feature(enable = "altivec")]
    #[cfg_attr(test, assert_instr(vadduhm))]
    pub unsafe fn vec_add_ss_ss(
        a: vector_signed_short, b: vector_signed_short,
    ) -> vector_signed_short {
        simd_add(a, b)
    }
    impl VectorAdd<vector_signed_short> for vector_signed_short {
        type Result = vector_signed_short;
        #[inline]
        #[target_feature(enable = "altivec")]
        unsafe fn vec_add(self, other: vector_signed_short) -> Self::Result {
            vec_add_ss_ss(self, other)
        }
    }

    #[inline]
    #[target_feature(enable = "altivec")]
    #[cfg_attr(test, assert_instr(vadduhm))]
    pub unsafe fn vec_add_bs_us(
        a: vector_bool_short, b: vector_unsigned_short,
    ) -> vector_unsigned_short {
        let a: i16x8 = ::mem::transmute(a);
        let a: vector_unsigned_short = simd_cast(a);
        simd_add(a, b)
    }
    impl VectorAdd<vector_unsigned_short> for vector_bool_short {
        type Result = vector_unsigned_short;
        #[inline]
        #[target_feature(enable = "altivec")]
        unsafe fn vec_add(self, other: vector_unsigned_short) -> Self::Result {
            vec_add_bs_us(self, other)
        }
    }
    impl VectorAdd<vector_bool_short> for vector_unsigned_short {
        type Result = vector_unsigned_short;
        #[inline]
        #[target_feature(enable = "altivec")]
        unsafe fn vec_add(self, other: vector_bool_short) -> Self::Result {
            other.vec_add(self)
        }
    }

    #[inline]
    #[target_feature(enable = "altivec")]
    #[cfg_attr(test, assert_instr(vadduhm))]
    pub unsafe fn vec_add_us_us(
        a: vector_unsigned_short, b: vector_unsigned_short,
    ) -> vector_unsigned_short {
        simd_add(a, b)
    }

    impl VectorAdd<vector_unsigned_short> for vector_unsigned_short {
        type Result = vector_unsigned_short;
        #[inline]
        #[target_feature(enable = "altivec")]
        unsafe fn vec_add(self, other: vector_unsigned_short) -> Self::Result {
            vec_add_us_us(self, other)
        }
    }

    #[inline]
    #[target_feature(enable = "altivec")]
    #[cfg_attr(test, assert_instr(vadduwm))]
    pub unsafe fn vec_add_bi_si(
        a: vector_bool_int, b: vector_signed_int,
    ) -> vector_signed_int {
        let a: i32x4 = ::mem::transmute(a);
        let a: vector_signed_int = simd_cast(a);
        simd_add(a, b)
    }
    impl VectorAdd<vector_signed_int> for vector_bool_int {
        type Result = vector_signed_int;
        #[inline]
        #[target_feature(enable = "altivec")]
        unsafe fn vec_add(self, other: vector_signed_int) -> Self::Result {
            vec_add_bi_si(self, other)
        }
    }
    impl VectorAdd<vector_bool_int> for vector_signed_int {
        type Result = vector_signed_int;
        #[inline]
        #[target_feature(enable = "altivec")]
        unsafe fn vec_add(self, other: vector_bool_int) -> Self::Result {
            other.vec_add(self)
        }
    }

    #[inline]
    #[target_feature(enable = "altivec")]
    #[cfg_attr(test, assert_instr(vadduwm))]
    pub unsafe fn vec_add_si_si(
        a: vector_signed_int, b: vector_signed_int,
    ) -> vector_signed_int {
        simd_add(a, b)
    }
    impl VectorAdd<vector_signed_int> for vector_signed_int {
        type Result = vector_signed_int;
        #[inline]
        #[target_feature(enable = "altivec")]
        unsafe fn vec_add(self, other: vector_signed_int) -> Self::Result {
            vec_add_si_si(self, other)
        }
    }

    #[inline]
    #[target_feature(enable = "altivec")]
    #[cfg_attr(test, assert_instr(vadduwm))]
    pub unsafe fn vec_add_bi_ui(
        a: vector_bool_int, b: vector_unsigned_int,
    ) -> vector_unsigned_int {
        let a: i32x4 = ::mem::transmute(a);
        let a: vector_unsigned_int = simd_cast(a);
        simd_add(a, b)
    }
    impl VectorAdd<vector_unsigned_int> for vector_bool_int {
        type Result = vector_unsigned_int;
        #[inline]
        #[target_feature(enable = "altivec")]
        unsafe fn vec_add(self, other: vector_unsigned_int) -> Self::Result {
            vec_add_bi_ui(self, other)
        }
    }
    impl VectorAdd<vector_bool_int> for vector_unsigned_int {
        type Result = vector_unsigned_int;
        #[inline]
        #[target_feature(enable = "altivec")]
        unsafe fn vec_add(self, other: vector_bool_int) -> Self::Result {
            other.vec_add(self)
        }
    }

    #[inline]
    #[target_feature(enable = "altivec")]
    #[cfg_attr(test, assert_instr(vadduwm))]
    pub unsafe fn vec_add_ui_ui(
        a: vector_unsigned_int, b: vector_unsigned_int,
    ) -> vector_unsigned_int {
        simd_add(a, b)
    }
    impl VectorAdd<vector_unsigned_int> for vector_unsigned_int {
        type Result = vector_unsigned_int;
        #[inline]
        #[target_feature(enable = "altivec")]
        unsafe fn vec_add(self, other: vector_unsigned_int) -> Self::Result {
            vec_add_ui_ui(self, other)
        }
    }

    #[inline]
    #[target_feature(enable = "altivec")]
    #[cfg_attr(test, assert_instr(xvaddsp))]
    pub unsafe fn vec_add_float_float(
        a: vector_float, b: vector_float,
    ) -> vector_float {
        simd_add(a, b)
    }

    impl VectorAdd<vector_float> for vector_float {
        type Result = vector_float;
        #[inline]
        #[target_feature(enable = "altivec")]
        unsafe fn vec_add(self, other: vector_float) -> Self::Result {
            vec_add_float_float(self, other)
        }
    }

    pub trait VectorMladd<Other> {
        type Result;
        unsafe fn vec_mladd(self, b: Other, c: Other) -> Self::Result;
    }

    #[inline]
    #[target_feature(enable = "altivec")]
    #[cfg_attr(test, assert_instr(vmladduhm))]
    unsafe fn mladd(a: i16x8, b: i16x8, c: i16x8) -> i16x8 {
        simd_add(simd_mul(a, b), c)
    }

    macro_rules! vector_mladd {
        ($a: ident, $bc: ident, $d: ident) => {
            impl VectorMladd<$bc> for $a {
                type Result = $d;
                #[inline]
                #[target_feature(enable = "altivec")]
                unsafe fn vec_mladd(self, b: $bc, c: $bc) -> Self::Result {
                    let a: i16x8 = ::mem::transmute(self);
                    let b: i16x8 = ::mem::transmute(b);
                    let c: i16x8 = ::mem::transmute(c);

                    ::mem::transmute(mladd(a, b, c))
                }
            }
        };
    }

    vector_mladd! { vector_unsigned_short, vector_unsigned_short, vector_unsigned_short }
    vector_mladd! { vector_unsigned_short, vector_signed_short, vector_signed_short }
    vector_mladd! { vector_signed_short, vector_unsigned_short, vector_signed_short }
    vector_mladd! { vector_signed_short, vector_signed_short, vector_signed_short }
}

/// Vector add.
#[inline]
#[target_feature(enable = "altivec")]
pub unsafe fn vec_add<T, U>(a: T, b: U) -> <T as sealed::VectorAdd<U>>::Result
where
    T: sealed::VectorAdd<U>,
{
    a.vec_add(b)
}

/// Endian-biased intrinsics
#[cfg(target_endian = "little")]
mod endian {
    use super::*;
    /// Vector permute.
    #[inline]
    #[target_feature(enable = "altivec")]
    pub unsafe fn vec_perm<T>(a: T, b: T, c: vector_unsigned_char) -> T
    where
        T: sealed::VectorPerm,
    {
        // vperm has big-endian bias
        //
        // Xor the mask and flip the arguments
        let d = ::mem::transmute(u8x16::new(
            255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255,
            255, 255, 255,
        ));
        let c = simd_xor(c, d);

        b.vec_vperm(a, c)
    }

    /// Vector Sum Across Partial (1/2) Saturated
    #[inline]
    #[target_feature(enable = "altivec")]
    pub unsafe fn vec_sum2s(
        a: vector_signed_int, b: vector_signed_int,
    ) -> vector_signed_int {
        // vsum2sws has big-endian bias
        //
        // swap the even b elements with the odd ones
        let flip = ::mem::transmute(u8x16::new(
            4, 5, 6, 7, 0, 1, 2, 3, 12, 13, 14, 15, 8, 9, 10, 11,
        ));
        let b = vec_perm(b, b, flip);
        let c = vsum2sws(a, b);

        vec_perm(c, c, flip)
    }

    // Even and Odd are swapped in little-endian
    /// Vector Multiply Even
    #[inline]
    #[target_feature(enable = "altivec")]
    pub unsafe fn vec_mule<T, U>(a: T, b: T) -> U
    where
        T: sealed::VectorMulo<U>,
    {
        a.vec_mulo(b)
    }
    /// Vector Multiply Odd
    #[inline]
    #[target_feature(enable = "altivec")]
    pub unsafe fn vec_mulo<T, U>(a: T, b: T) -> U
    where
        T: sealed::VectorMule<U>,
    {
        a.vec_mule(b)
    }
}

/// Vector Multiply Add Saturated
#[inline]
#[target_feature(enable = "altivec")]
#[cfg_attr(test, assert_instr(vmhaddshs))]
pub unsafe fn vec_madds(
    a: vector_signed_short, b: vector_signed_short, c: vector_signed_short,
) -> vector_signed_short {
    vmhaddshs(a, b, c)
}

/// Vector Multiply Low and Add Unsigned Half Word
#[inline]
#[target_feature(enable = "altivec")]
pub unsafe fn vec_mladd<T, U>(
    a: T, b: U, c: U,
) -> <T as sealed::VectorMladd<U>>::Result
where
    T: sealed::VectorMladd<U>,
{
    a.vec_mladd(b, c)
}

/// Vector Multiply Round and Add Saturated
#[inline]
#[target_feature(enable = "altivec")]
#[cfg_attr(test, assert_instr(vmhraddshs))]
pub unsafe fn vec_mradds(
    a: vector_signed_short, b: vector_signed_short, c: vector_signed_short,
) -> vector_signed_short {
    vmhraddshs(a, b, c)
}

/// Vector Multiply Sum
#[inline]
#[target_feature(enable = "altivec")]
pub unsafe fn vec_msum<T, B, U>(a: T, b: B, c: U) -> U
where
    T: sealed::VectorMsum<B, U>,
{
    a.vec_msum(b, c)
}

/// Vector Multiply Sum Saturated
#[inline]
#[target_feature(enable = "altivec")]
pub unsafe fn vec_msums<T, U>(a: T, b: T, c: U) -> U
where
    T: sealed::VectorMsums<U>,
{
    a.vec_msums(b, c)
}

/// Vector Multiply Add
#[inline]
#[target_feature(enable = "altivec")]
pub unsafe fn vec_madd(
    a: vector_float, b: vector_float, c: vector_float,
) -> vector_float {
    vmaddfp(a, b, c)
}

/// Vector Negative Multiply Subtract
#[inline]
#[target_feature(enable = "altivec")]
pub unsafe fn vec_nmsub(
    a: vector_float, b: vector_float, c: vector_float,
) -> vector_float {
    vnmsubfp(a, b, c)
}

/// Vector Sum Across Partial (1/4) Saturated
#[inline]
#[target_feature(enable = "altivec")]
pub unsafe fn vec_sum4s<T, U>(a: T, b: U) -> U
where
    T: sealed::VectorSum4s<U>,
{
    a.vec_sum4s(b)
}

#[cfg(target_endian = "big")]
mod endian {
    use super::*;
    /// Vector permute.
    #[inline]
    #[target_feature(enable = "altivec")]
    pub unsafe fn vec_perm<T>(a: T, b: T, c: vector_unsigned_char) -> T
    where
        T: sealed::VectorPerm,
    {
        a.vec_vperm(b, c)
    }

    /// Vector Sum Across Partial (1/2) Saturated
    #[inline]
    #[target_feature(enable = "altivec")]
    pub unsafe fn vec_sum2s(
        a: vector_signed_int, b: vector_signed_int,
    ) -> vector_signed_int {
        vsum2sws(a, b)
    }

    /// Vector Multiply Even
    #[inline]
    #[target_feature(enable = "altivec")]
    pub unsafe fn vec_mule<T, U>(a: T, b: T) -> U
    where
        T: sealed::VectorMule<U>,
    {
        a.vec_mule(b)
    }
    /// Vector Multiply Odd
    #[inline]
    #[target_feature(enable = "altivec")]
    pub unsafe fn vec_mulo<T, U>(a: T, b: T) -> U
    where
        T: sealed::VectorMulo<U>,
    {
        a.vec_mulo(b)
    }

}

pub use self::endian::*;

#[cfg(test)]
mod tests {
    #[cfg(target_arch = "powerpc")]
    use coresimd::arch::powerpc::*;

    #[cfg(target_arch = "powerpc64")]
    use coresimd::arch::powerpc64::*;

    use coresimd::simd::*;
    use stdsimd_test::simd_test;

    macro_rules! test_vec_perm {
        {$name:ident,
         $shorttype:ident, $longtype:ident,
         [$($a:expr),+], [$($b:expr),+], [$($c:expr),+], [$($d:expr),+]} => {
            #[simd_test(enable = "altivec")]
            unsafe fn $name() {
                let a: $longtype = ::mem::transmute($shorttype::new($($a),+));
                let b: $longtype = ::mem::transmute($shorttype::new($($b),+));
                let c: vector_unsigned_char = ::mem::transmute(u8x16::new($($c),+));
                let d = $shorttype::new($($d),+);

                let r: $shorttype = ::mem::transmute(vec_perm(a, b, c));
                assert_eq!(d, r);
            }
        }
    }

    test_vec_perm!{test_vec_perm_u8x16,
    u8x16, vector_unsigned_char,
    [0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15],
    [100, 101, 102, 103, 104, 105, 106, 107, 108, 109, 110, 111, 112, 113, 114, 115],
    [0x00, 0x01, 0x10, 0x11, 0x02, 0x03, 0x12, 0x13,
     0x04, 0x05, 0x14, 0x15, 0x06, 0x07, 0x16, 0x17],
    [0, 1, 100, 101, 2, 3, 102, 103, 4, 5, 104, 105, 6, 7, 106, 107]}
    test_vec_perm!{test_vec_perm_i8x16,
    i8x16, vector_signed_char,
    [0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15],
    [100, 101, 102, 103, 104, 105, 106, 107, 108, 109, 110, 111, 112, 113, 114, 115],
    [0x00, 0x01, 0x10, 0x11, 0x02, 0x03, 0x12, 0x13,
     0x04, 0x05, 0x14, 0x15, 0x06, 0x07, 0x16, 0x17],
    [0, 1, 100, 101, 2, 3, 102, 103, 4, 5, 104, 105, 6, 7, 106, 107]}

    test_vec_perm!{test_vec_perm_m8x16,
    m8x16, vector_bool_char,
    [false, false, false, false, false, false, false, false, false, false, false, false, false, false, false, false],
    [true, true, true, true, true, true, true, true, true, true, true, true, true, true, true, true],
    [0x00, 0x01, 0x10, 0x11, 0x02, 0x03, 0x12, 0x13,
     0x04, 0x05, 0x14, 0x15, 0x06, 0x07, 0x16, 0x17],
    [false, false, true, true, false, false, true, true, false, false, true, true, false, false, true, true]}
    test_vec_perm!{test_vec_perm_u16x8,
    u16x8, vector_unsigned_short,
    [0, 1, 2, 3, 4, 5, 6, 7],
    [10, 11, 12, 13, 14, 15, 16, 17],
    [0x00, 0x01, 0x10, 0x11, 0x02, 0x03, 0x12, 0x13,
     0x04, 0x05, 0x14, 0x15, 0x06, 0x07, 0x16, 0x17],
    [0, 10, 1, 11, 2, 12, 3, 13]}
    test_vec_perm!{test_vec_perm_i16x8,
    i16x8, vector_signed_short,
    [0, 1, 2, 3, 4, 5, 6, 7],
    [10, 11, 12, 13, 14, 15, 16, 17],
    [0x00, 0x01, 0x10, 0x11, 0x02, 0x03, 0x12, 0x13,
     0x04, 0x05, 0x14, 0x15, 0x06, 0x07, 0x16, 0x17],
    [0, 10, 1, 11, 2, 12, 3, 13]}
    test_vec_perm!{test_vec_perm_m16x8,
    m16x8, vector_bool_short,
    [false, false, false, false, false, false, false, false],
    [true, true, true, true, true, true, true, true],
    [0x00, 0x01, 0x10, 0x11, 0x02, 0x03, 0x12, 0x13,
     0x04, 0x05, 0x14, 0x15, 0x06, 0x07, 0x16, 0x17],
    [false, true, false, true, false, true, false, true]}

    test_vec_perm!{test_vec_perm_u32x4,
    u32x4, vector_unsigned_int,
    [0, 1, 2, 3],
    [10, 11, 12, 13],
    [0x00, 0x01, 0x02, 0x03, 0x10, 0x11, 0x12, 0x13,
     0x04, 0x05, 0x06, 0x07, 0x14, 0x15, 0x16, 0x17],
    [0, 10, 1, 11]}
    test_vec_perm!{test_vec_perm_i32x4,
    i32x4, vector_signed_int,
    [0, 1, 2, 3],
    [10, 11, 12, 13],
    [0x00, 0x01, 0x02, 0x03, 0x10, 0x11, 0x12, 0x13,
     0x04, 0x05, 0x06, 0x07, 0x14, 0x15, 0x16, 0x17],
    [0, 10, 1, 11]}
    test_vec_perm!{test_vec_perm_m32x4,
    m32x4, vector_bool_int,
    [false, false, false, false],
    [true, true, true, true],
    [0x00, 0x01, 0x02, 0x03, 0x10, 0x11, 0x12, 0x13,
     0x04, 0x05, 0x06, 0x07, 0x14, 0x15, 0x16, 0x17],
    [false, true, false, true]}
    test_vec_perm!{test_vec_perm_f32x4,
    f32x4, vector_float,
    [0.0, 1.0, 2.0, 3.0],
    [1.0, 1.1, 1.2, 1.3],
    [0x00, 0x01, 0x02, 0x03, 0x10, 0x11, 0x12, 0x13,
     0x04, 0x05, 0x06, 0x07, 0x14, 0x15, 0x16, 0x17],
    [0.0, 1.0, 1.0, 1.1]}

    #[simd_test(enable = "altivec")]
    unsafe fn test_vec_madds() {
        let a: vector_signed_short = ::mem::transmute(i16x8::new(
            0 * 256,
            1 * 256,
            2 * 256,
            3 * 256,
            4 * 256,
            5 * 256,
            6 * 256,
            7 * 256,
        ));
        let b: vector_signed_short = ::mem::transmute(i16x8::new(
            256, 256, 256, 256, 256, 256, 256, 256,
        ));
        let c: vector_signed_short =
            ::mem::transmute(i16x8::new(0, 1, 2, 3, 4, 5, 6, 7));

        let d = i16x8::new(0, 3, 6, 9, 12, 15, 18, 21);

        assert_eq!(d, ::mem::transmute(vec_madds(a, b, c)));
    }

    #[simd_test(enable = "altivec")]
    unsafe fn test_vec_madd_float() {
        let a: vector_float = ::mem::transmute(f32x4::new(0.1, 0.2, 0.3, 0.4));
        let b: vector_float = ::mem::transmute(f32x4::new(0.1, 0.2, 0.3, 0.4));
        let c: vector_float = ::mem::transmute(f32x4::new(0.1, 0.2, 0.3, 0.4));
        let d = f32x4::new(
            0.1 * 0.1 + 0.1,
            0.2 * 0.2 + 0.2,
            0.3 * 0.3 + 0.3,
            0.4 * 0.4 + 0.4,
        );

        assert_eq!(d, ::mem::transmute(vec_madd(a, b, c)));
    }

    #[simd_test(enable = "altivec")]
    unsafe fn test_vec_nmsub_float() {
        let a: vector_float = ::mem::transmute(f32x4::new(0.1, 0.2, 0.3, 0.4));
        let b: vector_float = ::mem::transmute(f32x4::new(0.1, 0.2, 0.3, 0.4));
        let c: vector_float = ::mem::transmute(f32x4::new(0.1, 0.2, 0.3, 0.4));
        let d = f32x4::new(
            -(0.1 * 0.1 - 0.1),
            -(0.2 * 0.2 - 0.2),
            -(0.3 * 0.3 - 0.3),
            -(0.4 * 0.4 - 0.4),
        );
        assert_eq!(d, ::mem::transmute(vec_nmsub(a, b, c)));
    }

    #[simd_test(enable = "altivec")]
    unsafe fn test_vec_mradds() {
        let a: vector_signed_short = ::mem::transmute(i16x8::new(
            0 * 256,
            1 * 256,
            2 * 256,
            3 * 256,
            4 * 256,
            5 * 256,
            6 * 256,
            7 * 256,
        ));
        let b: vector_signed_short = ::mem::transmute(i16x8::new(
            256, 256, 256, 256, 256, 256, 256, 256,
        ));
        let c: vector_signed_short = ::mem::transmute(i16x8::new(
            0,
            1,
            2,
            3,
            4,
            5,
            6,
            i16::max_value() - 1,
        ));

        let d = i16x8::new(0, 3, 6, 9, 12, 15, 18, i16::max_value());

        assert_eq!(d, ::mem::transmute(vec_mradds(a, b, c)));
    }

    macro_rules! test_vec_mladd {
        {$name:ident, $sa:ident, $la:ident, $sbc:ident, $lbc:ident, $sd:ident,
            [$($a:expr),+], [$($b:expr),+], [$($c:expr),+], [$($d:expr),+]} => {
            #[simd_test(enable = "altivec")]
            unsafe fn $name() {
                let a: $la = ::mem::transmute($sa::new($($a),+));
                let b: $lbc = ::mem::transmute($sbc::new($($b),+));
                let c = ::mem::transmute($sbc::new($($c),+));
                let d = $sd::new($($d),+);

                assert_eq!(d, ::mem::transmute(vec_mladd(a, b, c)));
            }
        }
    }

    test_vec_mladd! { test_vec_mladd_u16x8_u16x8, u16x8, vector_unsigned_short, u16x8, vector_unsigned_short, u16x8,
        [0, 1, 2, 3, 4, 5, 6, 7], [0, 1, 2, 3, 4, 5, 6, 7], [0, 1, 2, 3, 4, 5, 6, 7], [0, 2, 6, 12, 20, 30, 42, 56]
    }
    test_vec_mladd! { test_vec_mladd_u16x8_i16x8, u16x8, vector_unsigned_short, i16x8, vector_unsigned_short, i16x8,
        [0, 1, 2, 3, 4, 5, 6, 7], [0, 1, 2, 3, 4, 5, 6, 7], [0, 1, 2, 3, 4, 5, 6, 7], [0, 2, 6, 12, 20, 30, 42, 56]
    }
    test_vec_mladd! { test_vec_mladd_i16x8_u16x8, i16x8, vector_signed_short, u16x8, vector_unsigned_short, i16x8,
        [0, 1, 2, 3, 4, 5, 6, 7], [0, 1, 2, 3, 4, 5, 6, 7], [0, 1, 2, 3, 4, 5, 6, 7], [0, 2, 6, 12, 20, 30, 42, 56]
    }
    test_vec_mladd! { test_vec_mladd_i16x8_i16x8, i16x8, vector_signed_short, i16x8, vector_unsigned_short, i16x8,
        [0, 1, 2, 3, 4, 5, 6, 7], [0, 1, 2, 3, 4, 5, 6, 7], [0, 1, 2, 3, 4, 5, 6, 7], [0, 2, 6, 12, 20, 30, 42, 56]
    }

    #[simd_test(enable = "altivec")]
    unsafe fn test_vec_msum_unsigned_char() {
        let a: vector_unsigned_char = ::mem::transmute(u8x16::new(
            0, 1, 2, 3, 4, 5, 6, 7, 0, 1, 2, 3, 4, 5, 6, 7,
        ));
        let b: vector_unsigned_char = ::mem::transmute(u8x16::new(
            255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255,
            255, 255, 255,
        ));
        let c: vector_unsigned_int = ::mem::transmute(u32x4::new(0, 1, 2, 3));
        let d = u32x4::new(
            (0 + 1 + 2 + 3) * 255 + 0,
            (4 + 5 + 6 + 7) * 255 + 1,
            (0 + 1 + 2 + 3) * 255 + 2,
            (4 + 5 + 6 + 7) * 255 + 3,
        );

        assert_eq!(d, ::mem::transmute(vec_msum(a, b, c)));
    }

    #[simd_test(enable = "altivec")]
    unsafe fn test_vec_msum_signed_char() {
        let a: vector_signed_char = ::mem::transmute(i8x16::new(
            0, -1, 2, -3, 1, -1, 1, -1, 0, 1, 2, 3, 4, -5, -6, -7,
        ));
        let b: vector_unsigned_char = ::mem::transmute(i8x16::new(
            1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1,
        ));
        let c: vector_signed_int = ::mem::transmute(u32x4::new(0, 1, 2, 3));
        let d = i32x4::new(
            (0 - 1 + 2 - 3) + 0,
            (0) + 1,
            (0 + 1 + 2 + 3) + 2,
            (4 - 5 - 6 - 7) + 3,
        );

        assert_eq!(d, ::mem::transmute(vec_msum(a, b, c)));
    }
    #[simd_test(enable = "altivec")]
    unsafe fn test_vec_msum_unsigned_short() {
        let a: vector_unsigned_short = ::mem::transmute(u16x8::new(
            0 * 256,
            1 * 256,
            2 * 256,
            3 * 256,
            4 * 256,
            5 * 256,
            6 * 256,
            7 * 256,
        ));
        let b: vector_unsigned_short = ::mem::transmute(u16x8::new(
            256, 256, 256, 256, 256, 256, 256, 256,
        ));
        let c: vector_unsigned_int = ::mem::transmute(u32x4::new(0, 1, 2, 3));
        let d = u32x4::new(
            (0 + 1) * 256 * 256 + 0,
            (2 + 3) * 256 * 256 + 1,
            (4 + 5) * 256 * 256 + 2,
            (6 + 7) * 256 * 256 + 3,
        );

        assert_eq!(d, ::mem::transmute(vec_msum(a, b, c)));
    }

    #[simd_test(enable = "altivec")]
    unsafe fn test_vec_msum_signed_short() {
        let a: vector_signed_short = ::mem::transmute(i16x8::new(
            0 * 256,
            -1 * 256,
            2 * 256,
            -3 * 256,
            4 * 256,
            -5 * 256,
            6 * 256,
            -7 * 256,
        ));
        let b: vector_signed_short = ::mem::transmute(i16x8::new(
            256, 256, 256, 256, 256, 256, 256, 256,
        ));
        let c: vector_signed_int = ::mem::transmute(i32x4::new(0, 1, 2, 3));
        let d = i32x4::new(
            (0 - 1) * 256 * 256 + 0,
            (2 - 3) * 256 * 256 + 1,
            (4 - 5) * 256 * 256 + 2,
            (6 - 7) * 256 * 256 + 3,
        );

        assert_eq!(d, ::mem::transmute(vec_msum(a, b, c)));
    }

    #[simd_test(enable = "altivec")]
    unsafe fn test_vec_msums_unsigned() {
        let a: vector_unsigned_short = ::mem::transmute(u16x8::new(
            0 * 256,
            1 * 256,
            2 * 256,
            3 * 256,
            4 * 256,
            5 * 256,
            6 * 256,
            7 * 256,
        ));
        let b: vector_unsigned_short = ::mem::transmute(u16x8::new(
            256, 256, 256, 256, 256, 256, 256, 256,
        ));
        let c: vector_unsigned_int = ::mem::transmute(u32x4::new(0, 1, 2, 3));
        let d = u32x4::new(
            (0 + 1) * 256 * 256 + 0,
            (2 + 3) * 256 * 256 + 1,
            (4 + 5) * 256 * 256 + 2,
            (6 + 7) * 256 * 256 + 3,
        );

        assert_eq!(d, ::mem::transmute(vec_msums(a, b, c)));
    }

    #[simd_test(enable = "altivec")]
    unsafe fn test_vec_msums_signed() {
        let a: vector_signed_short = ::mem::transmute(i16x8::new(
            0 * 256,
            -1 * 256,
            2 * 256,
            -3 * 256,
            4 * 256,
            -5 * 256,
            6 * 256,
            -7 * 256,
        ));
        let b: vector_signed_short = ::mem::transmute(i16x8::new(
            256, 256, 256, 256, 256, 256, 256, 256,
        ));
        let c: vector_signed_int = ::mem::transmute(i32x4::new(0, 1, 2, 3));
        let d = i32x4::new(
            (0 - 1) * 256 * 256 + 0,
            (2 - 3) * 256 * 256 + 1,
            (4 - 5) * 256 * 256 + 2,
            (6 - 7) * 256 * 256 + 3,
        );

        assert_eq!(d, ::mem::transmute(vec_msums(a, b, c)));
    }

    #[simd_test(enable = "altivec")]
    unsafe fn test_vec_sum2s() {
        let a: vector_signed_int = ::mem::transmute(i32x4::new(0, 1, 2, 3));
        let b: vector_signed_int = ::mem::transmute(i32x4::new(0, 1, 2, 3));
        let d = i32x4::new(0, 0 + 1 + 1, 0, 2 + 3 + 3);

        assert_eq!(d, ::mem::transmute(vec_sum2s(a, b)));
    }

    #[simd_test(enable = "altivec")]
    unsafe fn test_vec_sum4s_unsigned_char() {
        let a: vector_unsigned_char = ::mem::transmute(u8x16::new(
            0, 1, 2, 3, 4, 5, 6, 7, 0, 1, 2, 3, 4, 5, 6, 7,
        ));
        let b: vector_unsigned_int = ::mem::transmute(u32x4::new(0, 1, 2, 3));
        let d = u32x4::new(
            0 + 1 + 2 + 3 + 0,
            4 + 5 + 6 + 7 + 1,
            0 + 1 + 2 + 3 + 2,
            4 + 5 + 6 + 7 + 3,
        );

        assert_eq!(d, ::mem::transmute(vec_sum4s(a, b)));
    }
    #[simd_test(enable = "altivec")]
    unsafe fn test_vec_sum4s_signed_char() {
        let a: vector_signed_char = ::mem::transmute(i8x16::new(
            0, 1, 2, 3, 4, 5, 6, 7, 0, 1, 2, 3, 4, 5, 6, 7,
        ));
        let b: vector_signed_int = ::mem::transmute(i32x4::new(0, 1, 2, 3));
        let d = i32x4::new(
            0 + 1 + 2 + 3 + 0,
            4 + 5 + 6 + 7 + 1,
            0 + 1 + 2 + 3 + 2,
            4 + 5 + 6 + 7 + 3,
        );

        assert_eq!(d, ::mem::transmute(vec_sum4s(a, b)));
    }
    #[simd_test(enable = "altivec")]
    unsafe fn test_vec_sum4s_signed_short() {
        let a: vector_signed_short =
            ::mem::transmute(i16x8::new(0, 1, 2, 3, 4, 5, 6, 7));
        let b: vector_signed_int = ::mem::transmute(i32x4::new(0, 1, 2, 3));
        let d = i32x4::new(0 + 1 + 0, 2 + 3 + 1, 4 + 5 + 2, 6 + 7 + 3);

        assert_eq!(d, ::mem::transmute(vec_sum4s(a, b)));
    }

    #[simd_test(enable = "altivec")]
    unsafe fn test_vec_mule_unsigned_char() {
        let a: vector_unsigned_char = ::mem::transmute(u8x16::new(
            0, 1, 2, 3, 4, 5, 6, 7, 0, 1, 2, 3, 4, 5, 6, 7,
        ));
        let d =
            u16x8::new(0 * 0, 2 * 2, 4 * 4, 6 * 6, 0 * 0, 2 * 2, 4 * 4, 6 * 6);

        assert_eq!(d, ::mem::transmute(vec_mule(a, a)));
    }

    #[simd_test(enable = "altivec")]
    unsafe fn test_vec_mule_signed_char() {
        let a: vector_signed_char = ::mem::transmute(i8x16::new(
            0, 1, -2, 3, -4, 5, -6, 7, 0, 1, 2, 3, 4, 5, 6, 7,
        ));
        let d =
            i16x8::new(0 * 0, 2 * 2, 4 * 4, 6 * 6, 0 * 0, 2 * 2, 4 * 4, 6 * 6);

        assert_eq!(d, ::mem::transmute(vec_mule(a, a)));
    }

    #[simd_test(enable = "altivec")]
    unsafe fn test_vec_mule_unsigned_short() {
        let a: vector_unsigned_short =
            ::mem::transmute(u16x8::new(0, 1, 2, 3, 4, 5, 6, 7));
        let d = u32x4::new(0 * 0, 2 * 2, 4 * 4, 6 * 6);

        assert_eq!(d, ::mem::transmute(vec_mule(a, a)));
    }

    #[simd_test(enable = "altivec")]
    unsafe fn test_vec_mule_signed_short() {
        let a: vector_signed_short =
            ::mem::transmute(i16x8::new(0, 1, -2, 3, -4, 5, -6, 7));
        let d = i32x4::new(0 * 0, 2 * 2, 4 * 4, 6 * 6);

        assert_eq!(d, ::mem::transmute(vec_mule(a, a)));
    }

    #[simd_test(enable = "altivec")]
    unsafe fn test_vec_mulo_unsigned_char() {
        let a: vector_unsigned_char = ::mem::transmute(u8x16::new(
            0, 1, 2, 3, 4, 5, 6, 7, 0, 1, 2, 3, 4, 5, 6, 7,
        ));
        let d =
            u16x8::new(1 * 1, 3 * 3, 5 * 5, 7 * 7, 1 * 1, 3 * 3, 5 * 5, 7 * 7);

        assert_eq!(d, ::mem::transmute(vec_mulo(a, a)));
    }

    #[simd_test(enable = "altivec")]
    unsafe fn test_vec_mulo_signed_char() {
        let a: vector_signed_char = ::mem::transmute(i8x16::new(
            0, 1, -2, 3, -4, 5, -6, 7, 0, 1, 2, 3, 4, 5, 6, 7,
        ));
        let d =
            i16x8::new(1 * 1, 3 * 3, 5 * 5, 7 * 7, 1 * 1, 3 * 3, 5 * 5, 7 * 7);

        assert_eq!(d, ::mem::transmute(vec_mulo(a, a)));
    }

    #[simd_test(enable = "altivec")]
    unsafe fn test_vec_mulo_unsigned_short() {
        let a: vector_unsigned_short =
            ::mem::transmute(u16x8::new(0, 1, 2, 3, 4, 5, 6, 7));
        let d = u32x4::new(1 * 1, 3 * 3, 5 * 5, 7 * 7);

        assert_eq!(d, ::mem::transmute(vec_mulo(a, a)));
    }

    #[simd_test(enable = "altivec")]
    unsafe fn test_vec_mulo_signed_short() {
        let a: vector_signed_short =
            ::mem::transmute(i16x8::new(0, 1, -2, 3, -4, 5, -6, 7));
        let d = i32x4::new(1 * 1, 3 * 3, 5 * 5, 7 * 7);

        assert_eq!(d, ::mem::transmute(vec_mulo(a, a)));
    }

    #[simd_test(enable = "altivec")]
    unsafe fn vec_add_i32x4_i32x4() {
        let x = i32x4::new(1, 2, 3, 4);
        let y = i32x4::new(4, 3, 2, 1);
        let x: vector_signed_int = ::mem::transmute(x);
        let y: vector_signed_int = ::mem::transmute(y);
        let z = vec_add(x, y);
        assert_eq!(i32x4::splat(5), ::mem::transmute(z));
    }
}
