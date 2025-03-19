#![no_std]
#![no_main]
#![feature(extern_types)]

mod grub_lib;

#[link_section = ".modname"]
#[no_mangle]
pub static GRUB_MODNAME: [u8; 11] = [b'r', b'u', b's', b't', b'_', b'h', b'e',
				     b'l', b'l', b'o', b'\0'];
#[link_section = ".module_license"]
#[no_mangle]
pub static GRUB_LICENSE: [u8; 15] = [b'L', b'I', b'C', b'E', b'N', b'S', b'E', b'=', b'G', b'P', b'L', b'v', b'3', b'+', b'\0'];


pub fn rust_hello (_argc: usize, _argv: &[&str]) -> grub_lib::ErrT {
    grub_lib::xputs("Hello, world\n");
    return 0;
}

#[no_mangle]
pub extern "C" fn grub_mod_init() {
    grub_lib::xputs("Hello");

    grub_lib::Command::register("rust_hello", rust_hello,
				"Rust hello", "Say hello from Rust.");
}

#[no_mangle]
pub extern "C" fn grub_mod_fini() {
    grub_lib::Command::unregister_all();
}
