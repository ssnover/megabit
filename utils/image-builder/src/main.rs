use std::path::{Path, PathBuf};
use std::process::Command;

use clap::Parser;
use rw_flash::image::ImageHeader;

#[derive(Parser, Clone, Debug)]
struct Args {
    #[arg(long)]
    ws: Option<PathBuf>,

    #[arg(long)]
    combined: bool,

    #[arg(long, short = 'n')]
    dry_run: bool,
}

fn main() {
    let Args {
        ws: ws_dir,
        combined,
        dry_run,
    } = Args::parse();
    let ws_dir = ws_dir.unwrap_or(std::env::current_dir().unwrap());
    let bootloader_bin = if combined {
        // Build the bootloader
        let bootloader_elf = run_cargo_build(&ws_dir, "bootloader", dry_run);
        // Convert the bootloader to a bin
        let bootloader_bin = PathBuf::from(format!(
            "{}/bootloader.bin",
            bootloader_elf.parent().unwrap().display()
        ));
        convert_elf_to_bin(&bootloader_elf, &bootloader_bin, dry_run);
        Some(bootloader_bin)
    } else {
        None
    };
    // Build the app image
    let app_elf = run_cargo_build(&ws_dir, "megabit-coproc-app", dry_run);
    // Convert the app image to a bin
    let app_bin = PathBuf::from(format!("{}/app.bin", app_elf.parent().unwrap().display()));
    convert_elf_to_bin(&app_elf, &app_bin, dry_run);

    let final_bin = bootloader_bin
        .map({
            let app_bin = app_bin.clone();
            |bin| {
                let app_data = std::fs::read(app_bin).unwrap();
                // Create an app image header
                let app_crc = crc32::crc32(&app_data);
                let header = ImageHeader::new(0x00010000, app_data.len() as u32, app_crc);
                let header_bytes = header.to_bytes();
                // Combine them into a single bin
                let bootloader_data = std::fs::read(&bin).unwrap();
                let bootloader_padding_len =
                    memory::flash_boot::LENGTH as usize - bootloader_data.len();
                let padding = (0..bootloader_padding_len).map(|_| 0xFF);
                let combined_bin: Vec<_> = bootloader_data
                    .into_iter()
                    .chain(padding)
                    .chain(header_bytes.into_iter())
                    .chain(app_data.into_iter())
                    .collect();
                let combined_bin_path =
                    PathBuf::from(format!("{}/combined.bin", bin.parent().unwrap().display()));
                std::fs::write(&combined_bin_path, &combined_bin).unwrap();
                combined_bin_path
            }
        })
        .unwrap_or(app_bin);

    let out = if combined {
        "./coproc-boot-and-app.bin"
    } else {
        "./coproc-app.bin"
    };
    std::fs::copy(final_bin, out).unwrap();
}

// Runs a build and returns the path to the elf file produced
fn run_cargo_build(manifest_dir: impl AsRef<Path>, bin: &str, dry_run: bool) -> PathBuf {
    let target = "thumbv6m-none-eabi";
    let mut cmd = Command::new("cargo");
    cmd.args([
        "build",
        "--release",
        "--bin",
        bin,
        "--target",
        target,
        "--manifest-path",
        format!("{}/Cargo.toml", manifest_dir.as_ref().display()).as_str(),
    ]);
    run_command(cmd, dry_run);

    format!(
        "{}/target/{target}/release/{bin}",
        manifest_dir.as_ref().display()
    )
    .into()
}

fn convert_elf_to_bin(elf: impl AsRef<Path>, bin: impl AsRef<Path>, dry_run: bool) {
    let mut cmd = Command::new("arm-none-eabi-objcopy");
    cmd.args([
        "-O",
        "binary",
        format!("{}", elf.as_ref().display()).as_str(),
        format!("{}", bin.as_ref().display()).as_str(),
    ]);
    run_command(cmd, dry_run);
}

fn run_command(mut cmd: Command, dry_run: bool) {
    cmd.stdout(std::io::stdout());
    cmd.stderr(std::io::stderr());
    println!("===================================================");
    println!(
        "Running command: {}",
        format!("{:?}", cmd).replace("\"", "")
    );
    if dry_run {
        println!("(Skipping as this a dry run)");
    } else {
        let output = cmd.output().unwrap();
        if output.status.code().unwrap() != 0 {
            eprintln!("Failed to run command, exited: {}", output.status);
            std::process::exit(1);
        }
    }

    println!("===================================================");
    println!();
}
