extern crate alloc;

use alloc::vec::Vec;
use alloc::vec;
use alloc::string::String;
use alloc::string::ToString;
use core::alloc::{GlobalAlloc, Layout};
use core::ffi::c_char;
use core::ffi::c_int;
use core::ffi::c_void;
use core::fmt::{self, Arguments, Write};

use alloc::ffi::CString;
use core::ffi::CStr;
use core::panic::PanicInfo;

extern "C" {
    static grub_xputs: extern "C" fn(stri: *const c_char);
    fn grub_abort();
    fn grub_malloc(sz: usize) -> *mut u8;
    fn grub_free(ptr: *mut u8);
    fn grub_register_command_prio (name: *const c_char,
				   func: extern "C" fn (cmd: *const GrubCommand,
							argc: c_int, argv: *const *const c_char) -> u32,
				   summary: *const c_char,
				   description: *const c_char,
				   prio: c_int) -> *mut GrubCommand;
    fn grub_strlen (s: *const c_char) -> usize;
    fn grub_unregister_command (cmd: *const GrubCommand);
    fn grub_refresh ();
    fn grub_debug_enabled(cond: *const c_char) -> bool;
    fn grub_error (n: u32, fmt: *const c_char, args: ...) -> u32;
}

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

#[macro_export]
macro_rules! dprintln {
    ($cond:expr, $($args: expr),*) => {
	$crate::grub_lib::real_dprintln(file!(), line!(), $cond, format_args_nl!($($args)*));
    }
}

#[macro_export]
macro_rules! eformat {
    ($num:expr, $($args: expr),*) => {
	$crate::grub_lib::GrubError::new_fmt($num, format_args!($($args)*))
    }
}

#[macro_export]
macro_rules! format {
    ($($args: expr),*) => {
	$crate::grub_lib::format(format_args!($($args)*))
    }
}

pub fn format(args: Arguments<'_>) -> String {
    let mut w = StrWriter {output: "".to_string()};
    let _ = fmt::write(&mut w, args);

    return w.output;
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

// TODO: Use codegen here
pub enum ErrT {
    None = 0,
    TestFailure = 1,
    BadModule = 2,
    OutOfMemory = 3,
    BadFileType = 4,
    FileNotFound = 5,
    FileReadError = 6,
    BadFilename = 7,
    UnknownFs = 8,
    BadFs = 9,
    BadNumber = 10,
    OutOfRange = 11,
    UnknownDevice = 12,
    BadDevice = 13,
    ReadError = 14,
    WriteError = 15,
    UnknownCommand = 16,
    InvalidCommand = 17,
    BadArgument = 18,
    BadPartTable = 19,
    UnknownOs = 20,
    BadOs = 21,
    NoKernel = 22,
    BadFont = 23,
    NotImplementedYet = 24,
    SymlinkLoop = 25,
    BadCompressedData = 26,
    Menu = 27,
    Timeout = 28,
    Io = 29,
    AccessDenied = 30,
    Extractor = 31,
    NetBadAddress = 32,
    NetRouteLoop = 33,
    NetNoRoute = 34,
    NetNoAnswer = 35,
    NetNoCard = 36,
    Wait = 37,
    Bug = 38,
    NetPortClosed = 39,
    NetInvalidResponse = 40,
    NetUnknownError = 41,
    NetPacketTooBig = 42,
    NetNoDomain = 43,
    Eof = 44,
    BadSignature = 45,
    BadFirmware = 46,
    StillReferenced = 47,
    RecursionDepth = 48,
}

pub struct GrubError {
    errno: ErrT,
    errmsg: String,
}

impl GrubError {
    pub fn new(num: ErrT, msg: &str) -> Self {
	return GrubError{errno: num, errmsg: msg.to_string()};
    }

    pub fn new_fmt(num: ErrT, args: Arguments<'_>) -> Self {
	let mut w = StrWriter {output: "".to_string()};
	let _ = fmt::write(&mut w, args);

	return GrubError{errno: num, errmsg: w.output};
    }
}

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
			    argc: c_int, argv: *const *const c_char) -> u32 {
    let mut argv_vec: Vec<&str> = vec![];
    for i in 0..argc {
	argv_vec.push(unsafe { CStr::from_ptr(*argv.add(i as usize)) }.to_str().unwrap());
    }
    let f = unsafe { *((&(*cmd).data) as *const _ as *const fn(&[&str]) -> Result<(), GrubError>) };
    match f (&argv_vec) {
	Ok(_) => { return 0; }
	Err(e) => {
	    let c_errmsg = CString::new(e.errmsg).unwrap();

	    return unsafe {grub_error(e.errno as u32, CStr::from_bytes_until_nul(b"%s\0").unwrap().as_ptr(), c_errmsg.as_ptr())};
	}
    }
}

pub struct Command {
    name: CString,
    summary: CString,
    description: CString,
    cmd: *mut GrubCommand,
}

static mut commands: Vec<Command> = vec![];

impl Command {
    pub fn register(name: &str, cb: fn (argv: &[&str]) -> Result<(), GrubError>,
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

struct StrWriter {
    output: String,
}

impl Write for StrWriter {
    fn write_str(&mut self, s: &str) -> fmt::Result {
        self.output += s;
        Ok(())
    }
}

pub fn print_fmt(args: Arguments<'_>) {
    let mut w = PutsWriter;
    let _ = fmt::write(&mut w, args);
}

pub fn real_dprintln(file: &str, line: u32, cond: &str, args: Arguments<'_>) {
    let c_cond = CString::new(cond).unwrap();

    if !unsafe { grub_debug_enabled (c_cond.as_ptr()) } {
	return;
    }
    print!("{file}:{line}:{cond}: ");
    print_fmt(args);
    unsafe { grub_refresh (); }
}
