//! RDRAND and RDSEED instructions for returning random numbers from an Intel
//! on-chip hardware random number generator which has been seeded by an
//! on-chip entropy source.

extern "platform-intrinsic" {
    fn x86_rdrand64_step() -> (u64, i32);
    fn x86_rdseed64_step() -> (u64, i32);
}

#[cfg(test)]
use stdsimd_test::assert_instr;

/// Read a hardware generated 64-bit random value and store the result in val.
/// Return 1 if a random value was generated, and 0 otherwise.
#[inline]
#[target_feature(enable = "rdrand")]
#[cfg_attr(test, assert_instr(rdrand))]
#[cfg_attr(feature = "cargo-clippy", allow(stutter))]
pub unsafe fn _rdrand64_step(val: &mut u64) -> i32 {
    let (v, flag) = x86_rdrand64_step();
    *val = v;
    flag
}

/// Read a 64-bit NIST SP800-90B and SP800-90C compliant random value and store
/// in val. Return 1 if a random value was generated, and 0 otherwise.
#[inline]
#[target_feature(enable = "rdseed")]
#[cfg_attr(test, assert_instr(rdseed))]
pub unsafe fn _rdseed64_step(val: &mut u64) -> i32 {
    let (v, flag) = x86_rdseed64_step();
    *val = v;
    flag
}
