#![no_main]
#![no_std]
#![allow(unsafe_op_in_unsafe_fn)]

use bootload::detect_partition_to_run;
use panic_halt as _;
use rp_pac as _;

use rw_flash::{flash, nvs};

const BOOTLOADER_VERSION: &[u8] =
    include_bytes!(concat!(env!("OUT_DIR"), "/bootloader_version.bin"));

const fn bootloader_version() -> u32 {
    (BOOTLOADER_VERSION[0] as u32) << 24
        | (BOOTLOADER_VERSION[1] as u32) << 16
        | (BOOTLOADER_VERSION[2] as u32) << 8
        | (BOOTLOADER_VERSION[3] as u32)
}

const MAX_BOOT_ATTEMPTS: u8 = 3;

#[rp2040_hal::entry]
fn main() -> ! {
    let flash_storage = flash::PicoFlash::new(nvs::SECTOR_SIZE);
    let nvs = nvs::NvsHandle {
        flash: &flash_storage,
    };

    let Some(boot_target_addr) = detect_partition_to_run(
        &nvs,
        &flash_storage,
        bootloader_version(),
        MAX_BOOT_ATTEMPTS,
    )
    .map(|part| part.boot_target_addr()) else {
        // We couldn't find a partition, best to reset
        rp2040_hal::reset();
    };

    unsafe { boot_jump_to_addr(boot_target_addr) }
}

unsafe fn boot_jump_to_addr(addr: u32) -> ! {
    let peripherals = cortex_m::Peripherals::steal();
    peripherals.SCB.vtor.write(addr);

    cortex_m::asm::bootload(addr as *const u32)
}
