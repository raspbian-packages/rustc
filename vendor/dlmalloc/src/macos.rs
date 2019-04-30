extern crate libc;

use core::ptr;

pub unsafe fn alloc(size: usize) -> (*mut u8, usize, u32) {
    let addr = libc::mmap(0 as *mut _,
                          size,
                          libc::PROT_WRITE | libc::PROT_READ,
                          libc::MAP_ANON | libc::MAP_PRIVATE,
                          -1,
                          0);
    if addr == libc::MAP_FAILED {
        (ptr::null_mut(), 0, 0)
    } else {
        (addr as *mut u8, size, 0)
    }
}

pub unsafe fn remap(_ptr: *mut u8, _oldsize: usize, _newsize: usize, _can_move: bool)
    -> *mut u8
{
    ptr::null_mut()
}

pub unsafe fn free_part(ptr: *mut u8, oldsize: usize, newsize: usize) -> bool {
    libc::munmap(ptr.offset(newsize as isize) as *mut _, oldsize - newsize) == 0
}

pub unsafe fn free(ptr: *mut u8, size: usize) -> bool {
    libc::munmap(ptr as *mut _, size) == 0
}

pub fn can_release_part(_flags: u32) -> bool {
    true
}

#[cfg(feature = "global")]
static mut LOCK: libc::pthread_mutex_t = libc::PTHREAD_MUTEX_INITIALIZER;

#[cfg(feature = "global")]
pub fn acquire_global_lock() {
    unsafe {
        assert_eq!(libc::pthread_mutex_lock(&mut LOCK), 0)
    }
}

#[cfg(feature = "global")]
pub fn release_global_lock() {
    unsafe {
        assert_eq!(libc::pthread_mutex_unlock(&mut LOCK), 0)
    }
}

pub fn allocates_zeros() -> bool {
    true
}

pub fn page_size() -> usize {
    4096
}
