#![no_std]
#![no_main]
#![feature(extern_types)]
#![feature(rustc_attrs)]
#![feature(format_args_nl)]

mod grub_lib;

#[link_section = ".modname"]
#[no_mangle]
pub static GRUB_MODNAME: [u8; 11] = *b"rust_hello\0";
#[link_section = ".module_license"]
#[no_mangle]
pub static GRUB_LICENSE: [u8; 15] = *b"LICENSE=GPLv3+\0";


pub fn rust_hello (argv: &[&str]) -> grub_lib::ErrT {
    println!("Hello, world argv={argv:?}");
    return 0;
}

#[no_mangle]
pub extern "C" fn grub_mod_init() {
    grub_lib::Command::register("rust_hello", rust_hello,
				"Rust hello", "Say hello from Rust.");
}

#[no_mangle]
pub extern "C" fn grub_mod_fini() {
    grub_lib::Command::unregister_all();
}
