fn main() {
    println!("cargo:rustc-link-arg=-Wl,-u,GRUB_MODNAME");
    println!("cargo:rustc-link-arg=-Wl,-u,GRUB_LICENSE");
}
