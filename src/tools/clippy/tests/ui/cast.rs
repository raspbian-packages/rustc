


#[warn(cast_precision_loss, cast_possible_truncation, cast_sign_loss, cast_possible_wrap, cast_lossless)]
#[allow(no_effect, unnecessary_operation)]
fn main() {
    // Test cast_precision_loss
    1i32 as f32;
    1i64 as f32;
    1i64 as f64;
    1u32 as f32;
    1u64 as f32;
    1u64 as f64;
    // Test cast_possible_truncation
    1f32 as i32;
    1f32 as u32;
    1f64 as f32;
    1i32 as i8;
    1i32 as u8;
    1f64 as isize;
    1f64 as usize;
    // Test cast_possible_wrap
    1u8 as i8;
    1u16 as i16;
    1u32 as i32;
    1u64 as i64;
    1usize as isize;
    // Test cast_lossless with casts from floating-point types
    1.0f32 as f64;
    // Test cast_lossless with an expression wrapped in parens
    (1u8 + 1u8) as u16;
    // Test cast_sign_loss
    1i32 as u32;
    1isize as usize;
    // Extra checks for *size
    // Test cast_unnecessary
    1i32 as i32;
    1f32 as f32;
    false as bool;
    &1i32 as &i32;
    // Should not trigger
    let v = vec!(1);
    &v as &[i32];
    1.0 as f64;
    1 as u64;
}
