// Validation makes this fail in the wrong place
// compile-flags: -Zmir-emit-validate=0

fn main() {
    let g = unsafe {
        std::mem::transmute::<usize, fn(i32)>(42)
    };

    g(42) //~ ERROR a memory access tried to interpret some bytes as a pointer
}
