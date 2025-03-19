extern crate alloc;

use core::alloc::{GlobalAlloc, Layout};
use core::ffi::c_char;

use alloc::ffi::CString;
use core::panic::PanicInfo;

extern "C" {
    static grub_xputs: extern "C" fn(stri: *const c_char);
    pub fn grub_abort();
    pub fn grub_malloc(sz: usize) -> *mut u8;
    pub fn grub_free(ptr: *mut u8);
}


struct GrubAllocator;

unsafe impl GlobalAlloc for GrubAllocator {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        return grub_malloc(layout.size());
    }

    unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout) {
        grub_free(ptr);
    }
}

#[global_allocator]
static GLOBAL: GrubAllocator = GrubAllocator;

pub fn xputs(val: &str) {
    let c_to_print = CString::new(val).unwrap();

    unsafe {
	grub_xputs(c_to_print.as_ptr());
    }
}

#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    unsafe {
	grub_abort(); // TODO: Use grub_fatal and better error message
    }
    loop{}
}
