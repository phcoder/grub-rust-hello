extern crate alloc;

use alloc::vec::Vec;
use alloc::vec;
use core::alloc::{GlobalAlloc, Layout};
use core::ffi::c_char;
use core::ffi::c_int;
use core::ffi::c_void;
use core::fmt::{self, Arguments, Write};

use alloc::ffi::CString;
use core::ffi::CStr;
use core::panic::PanicInfo;

#[macro_export]
#[cfg_attr(not(test), rustc_diagnostic_item = "print_macro")]
macro_rules! print {
    ($($arg:tt)*) => {{
        $crate::grub_lib::print_fmt(format_args!($($arg)*));
    }};
}

#[macro_export]
#[cfg_attr(not(test), rustc_diagnostic_item = "println_macro")]
macro_rules! println {
    () => {
        $crate::print!("\n")
    };
    ($($arg:tt)*) => {{
        $crate::grub_lib::print_fmt(format_args_nl!($($arg)*));
    }};
}

extern "C" {
    static grub_xputs: extern "C" fn(stri: *const c_char);
    fn grub_abort();
    fn grub_malloc(sz: usize) -> *mut u8;
    fn grub_free(ptr: *mut u8);
    fn grub_register_command_prio (name: *const c_char,
				   func: extern "C" fn (cmd: *const GrubCommand,
							argc: c_int, argv: *const *const c_char) ->ErrT,
				   summary: *const c_char,
				   description: *const c_char,
				   prio: c_int) -> *mut GrubCommand;
    fn grub_strlen (s: *const c_char) -> usize;
    fn grub_unregister_command (cmd: *const GrubCommand);
}

#[no_mangle]
pub extern "C" fn strlen(s: *const c_char) -> usize {
    return unsafe { grub_strlen(s) }; 
}


// TODO: Use code generation?
#[repr(C)]
struct GrubCommand {
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

pub type ErrT = u32;

struct GrubAllocator;

unsafe impl GlobalAlloc for GrubAllocator {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        return grub_malloc(layout.size());
    }

    unsafe fn dealloc(&self, ptr: *mut u8, _layout: Layout) {
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
    println!("Rust runtime panicked {info:?}");
    unsafe {
	grub_abort();
    }
    loop{}
}

extern "C" fn cmd_callback (cmd: *const GrubCommand,
			    argc: c_int, argv: *const *const c_char) -> ErrT {
    let mut argv_vec: Vec<&str> = vec![];
    for i in 0..argc {
	argv_vec.push(unsafe { CStr::from_ptr(*argv.add(i as usize)) }.to_str().unwrap());
    }
    let f = unsafe { *((&(*cmd).data) as *const _ as *const fn(&[&str]) -> ErrT) };
    return f (&argv_vec);
}

pub struct Command {
    name: CString,
    summary: CString,
    description: CString,
    cmd: *mut GrubCommand,
}

static mut commands: Vec<Command> = vec![];

impl Command {
    pub fn register(name: &str, cb: fn (argv: &[&str]) -> ErrT,
		    summary: &str, description: &str) {
	let mut ret = Command {
	    name: CString::new(name).unwrap(),
	    summary: CString::new(summary).unwrap(),
	    description: CString::new(description).unwrap(),
	    cmd: core::ptr::null_mut(),
	};
	unsafe {
	    ret.cmd = grub_register_command_prio (ret.name.as_ptr(),
						  cmd_callback,
						  ret.summary.as_ptr(),
						  ret.description.as_ptr(),
						  0);
	    (*ret.cmd).data = cb as *mut c_void;
	    commands.push(ret);
	}
    }

    pub fn unregister_all() {
	unsafe {
	    for cmd in commands.iter() {
		grub_unregister_command(cmd.cmd);
	    }
	}
    }
}

struct PutsWriter;

impl Write for PutsWriter {
    fn write_str(&mut self, s: &str) -> fmt::Result {
        xputs(s);
        Ok(())
    }
}

pub fn print_fmt(args: Arguments<'_>) {
    let mut w = PutsWriter;
    let _ = fmt::write(&mut w, args);
}
