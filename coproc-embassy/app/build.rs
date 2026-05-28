use std::env;
use std::path::PathBuf;

fn main() {
    let out = &PathBuf::from(env::var_os("OUT_DIR").unwrap());
    println!("cargo:rustc-link-search={}", out.display());

    coproc_build_utils::copy_linker_files_to_output_directory().unwrap();

    println!("cargo:rustc-link-arg-bins=--nmagic");
    println!("cargo:rustc-link-arg-bins=-Tapp-a.x");
    println!("cargo:rustc-link-arg-bins=-Tdefmt.x");
}
