fn main() {
    let x = 13;
    let y = &x;
    let z = &y as *const &i32 as *const usize;
    let ptr_bytes = unsafe { *z }; // the actual deref is fine, because we read the entire pointer at once
    let _ = ptr_bytes % 432; //~ ERROR: tried to access part of a pointer value as raw bytes
}
