#![feature(extern_types)]

extern crate alloc;

use alloc::vec::Vec;
use alloc::vec;
use alloc::string::String;
use core::alloc::{GlobalAlloc, Layout};
use core::ffi::c_char;
use core::ffi::c_int;
use core::ffi::c_void;

use alloc::ffi::CString;
use core::ffi::CStr;
use core::panic::PanicInfo;

extern "C" {
    static grub_xputs: extern "C" fn(stri: *const c_char);
    pub fn grub_abort();
    pub fn grub_malloc(sz: usize) -> *mut u8;
    pub fn grub_free(ptr: *mut u8);
    pub fn grub_register_command_prio (name: *const c_char,
				       func: fn (cmd: *const GrubCommand,
						 argc: c_int, argv: *const *const c_char) ->err_t,
				       summary: *const c_char,
				       description: *const c_char,
				       prio: c_int) -> *mut GrubCommand;
    pub fn grub_strlen (s: *const c_char) -> usize;
    pub fn grub_unregister_command (cmd: *const GrubCommand);
}

#[no_mangle]
pub extern "C" fn strlen(s: *const c_char) -> usize {
    return unsafe { grub_strlen(s) }; 
}


// TODO: Use code generation?
#[repr(C)]
pub struct GrubCommand {
    next: *mut GrubCommand,
    prev: *mut *mut GrubCommand,
    name: *const c_char,
    prio: c_int,
    func: *const c_void,
    flags: u32,
    summary: *const c_char,
    description: *const c_char,
    data: *const c_void,
}

pub type GrubCommandPtr = *const GrubCommand;
pub type err_t = u32;

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

fn cmd_callback (cmd: *const GrubCommand,
		 argc: c_int, argv: *const *const c_char) -> err_t {
    let mut argv_vec: Vec<&str> = vec![];
    for i in 0..argc {
	argv_vec.push(unsafe { CStr::from_ptr(*argv.add(i as usize)) }.to_str().unwrap());
    }

    return unsafe{(*((*cmd).data as *const fn (argc: usize, argv: &[&str]) -> err_t))} (argc as usize, &argv_vec);
}

pub fn register_command (name: &str, cb: fn (argc: usize, argv: &[&str]) -> err_t,
			 summary: &str, description: &str) -> GrubCommandPtr {
    unsafe {
	let cmd = grub_register_command_prio (CString::new(name).unwrap().as_ptr(),
					      cmd_callback,
					      CString::new(summary).unwrap().as_ptr(),
					      CString::new(description).unwrap().as_ptr(),
					      0);
	(*cmd).data = cb as *mut c_void;
	return cmd;
    }
}

pub fn unregister_command (cmd: GrubCommandPtr) {
    unsafe { grub_unregister_command(cmd); }
}
