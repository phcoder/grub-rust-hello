extern crate alloc;
use alloc::ffi::CString;
use core::panic::PanicInfo;

extern "C" {
    static grub_xputs: extern "C" fn(stri: *const u8);
    pub fn grub_abort();
}

pub fn xputs(val: &str) {
/*    let c_to_print = CString::new(val).unwrap();

    unsafe {
	grub_xputs(c_to_print.as_ptr());
    }*/
}

#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    unsafe {
	grub_abort(); // TODO: Use grub_fatal and better error message
    }
    loop{}
}
