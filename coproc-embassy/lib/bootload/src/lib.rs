#![no_std]

use memory::flash_app::AppPartition;
use rw_flash::{
    flash::FlashStorage,
    image::{self, verify_image},
    nvs::{self, BootState},
};

pub fn detect_partition_to_run<'a, F: FlashStorage>(
    nvs: &nvs::NvsHandle<'a, F>,
    flash: &'a F,
    bootloader_version: u32,
    max_boot_attempts: u8,
) -> Option<AppPartition> {
    let mut state = nvs
        .read_boot_state()
        .unwrap_or_else(|| BootState::default(bootloader_version));

    // Increment the counter prior to boot
    state.increment_boot_attempts();
    nvs.write_boot_state(&state);

    if state.boot_attempts > max_boot_attempts {
        state.revert();
        nvs.write_boot_state(&state);
    }

    let partition = AppPartition::from_u8(state.active_partition);

    // Verify the image by reading the appropriate image header, then checking crc32
    let image_header = image::read_image_header(partition, flash);
    if image_header.is_valid() {
        if verify_image(partition, &image_header, flash) {
            return Some(partition);
        }
    }

    None
}
