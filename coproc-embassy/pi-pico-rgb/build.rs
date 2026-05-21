use std::env;
use std::path::PathBuf;

fn main() {
    let out = &PathBuf::from(env::var_os("OUT_DIR").unwrap());
    println!("cargo:rustc-link-search={}", out.display());

    copy_linker_files_to_output_directory().unwrap();

    println!("cargo:rustc-link-arg-bins=--nmagic");
    println!("cargo:rustc-link-arg-bins=-Tapp-a.x");
    println!("cargo:rustc-link-arg-bins=-Tdefmt.x");
}

fn copy_linker_files_to_output_directory() -> std::io::Result<()> {
    let linker_file_dir = PathBuf::from(format!(
        "{}/../link",
        env::var_os("CARGO_MANIFEST_DIR")
            .unwrap()
            .into_string()
            .unwrap()
    ));
    let out_dir = PathBuf::from(env::var_os("OUT_DIR").unwrap());

    for entry in linker_file_dir.read_dir()? {
        let Ok(entry) = entry else {
            continue;
        };
        if entry.path().is_file() {
            std::fs::copy(
                entry.path(),
                format!("{}/{}", out_dir.display(), entry.file_name().display()),
            )?;
        }
    }

    Ok(())
}
