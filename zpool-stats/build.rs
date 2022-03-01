fn main() {
    if cfg!(target_os = "freebsd") {
        println!("cargo:rustc-link-lib=spl");
        println!("cargo:rustc-link-lib=zutil");
    }
}
