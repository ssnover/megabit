use std::env;
use std::io::Write;
use std::path::PathBuf;

fn main() {
    let out = &PathBuf::from(env::var_os("OUT_DIR").unwrap());
    println!("cargo:rustc-link-search={}", out.display());

    coproc_build_utils::copy_linker_files_to_output_directory().unwrap();

    println!("cargo:rustc-link-arg-bins=--nmagic");
    println!("cargo:rustc-link-arg-bins=-Tbootloader.x");
    println!("cargo:rustc-link-arg-bins=-Tdevice.x");
    //println!("cargo:rustc-link-arg-bins=-Tdefmt.x");

    let version = env!("CARGO_PKG_VERSION");
    let version_parts = version.split('.').collect::<Vec<&str>>();
    assert_eq!(version_parts.len(), 3);
    let major = version_parts[0].parse::<u8>().unwrap();
    let minor = version_parts[1].parse::<u8>().unwrap();
    let patch = version_parts[2].parse::<u8>().unwrap();
    let version_data = [major, minor, patch, 0x00];
    let mut file =
        std::fs::File::create(format!("{}/bootloader_version.bin", out.display())).unwrap();
    file.write_all(&version_data).unwrap();
}
