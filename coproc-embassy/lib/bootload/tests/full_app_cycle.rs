#[test]
fn full_app_cycle() -> std::io::Result<()> {
    let flash = TestFlash::new();
    let nvs = nvs::NvsHandle { flash: &flash };

    // Start by verifying that on start up with a completely blank app image that the bootlaoder
    // attempts to create a boot state entry
    assert!(nvs.read_boot_state().is_none());
    let partition = bootload::detect_partition_to_run(&nvs, &flash, 0x00010000, 3);
    // There should be a new boot state value, but no valid image so it will fail to find a partition
    assert!(
        partition.is_none(),
        "App partition should be none, but is: {partition:?}"
    );
    // Verify a boot state entry was created
    let last_entry = nvs.read_boot_state().unwrap();
    assert_eq!(last_entry.active_partition, 0);
    assert_eq!(last_entry.update_pending, false);
    assert_eq!(last_entry.boot_attempts, 1);

    // Now let's write an app to partition A
    flash.write_app_image(AppPartition::A, 0x00010000);

    let partition = bootload::detect_partition_to_run(&nvs, &flash, 0x00010000, 3).unwrap();
    assert!(matches!(partition, AppPartition::A));

    // Now let's write another app to partition B
    flash.write_app_image(AppPartition::B, 0x00020000);
    let partition = bootload::detect_partition_to_run(&nvs, &flash, 0x00010000, 3).unwrap();
    // Verify that writing a new app image is not sufficient
    assert!(matches!(partition, AppPartition::A));

    flash.write_boot_state(&nvs, AppPartition::B);
    let partition = bootload::detect_partition_to_run(&nvs, &flash, 0x00010000, 3).unwrap();
    assert!(matches!(partition, AppPartition::B));

    Ok(())
}

use std::{
    cell::RefCell,
    cmp::min,
    mem::{self, MaybeUninit},
};

use memory::{flash_app::AppPartition, flash_boot};
use rw_flash::{
    flash::FlashStorage,
    image::ImageHeader,
    nvs::{self, SECTOR_SIZE, SectorHeader},
};

pub struct TestFlash {
    mem: RefCell<Box<[u8; 2 * 1024 * 1024]>>,
    origin_offset: usize,
    sector_size: usize,
}

impl TestFlash {
    pub fn new() -> Self {
        Self::initialize_with_blank_nvs()
    }

    fn initialize_with_blank_nvs() -> Self {
        let flash = Self {
            origin_offset: memory::flash_boot::ORIGIN as usize,
            mem: RefCell::new(Box::new([0xFFu8; 2 * 1024 * 1024])),
            sector_size: SECTOR_SIZE,
        };
        // The first page of NVS region should be a sector header
        // We'll just initialize one
        let header = SectorHeader::new(0);
        flash.write_sector_header(header, 0);
        flash
    }

    fn write_sector_header(&self, header: SectorHeader, sector: u8) {
        let bytes = unsafe {
            core::slice::from_raw_parts(
                &header as *const _ as *const u8,
                mem::size_of::<SectorHeader>(),
            )
        };
        let offset = (memory::flash_nvs::ORIGIN - memory::flash_boot::ORIGIN)
            + (sector as u32 * self.sector_size as u32);
        unsafe { self.write_page(offset, bytes) };
    }

    fn write_boot_state(&self, nvs: &nvs::NvsHandle<'_, Self>, partition: AppPartition) {
        let mut boot_state = nvs::BootState::default(0x00010000);
        boot_state.active_partition = partition as u8;
        nvs.write_boot_state(&boot_state);
    }

    fn generate_app_image(&self, version: u32) -> Vec<u8> {
        const LEN: usize = 1028;
        let version = version.to_le_bytes();
        let mut image_bytes = vec![0u8; LEN];
        for idx in (0..image_bytes.len()).step_by(4) {
            for byte in 0..4 {
                image_bytes[idx + byte] = version[byte];
            }
        }

        image_bytes
    }

    fn write_app_image(&self, partition: AppPartition, version: u32) {
        let image = self.generate_app_image(version);
        let crc = crc32::crc32(&image);
        let image_header = ImageHeader::new(version, image.len() as u32, crc);
        let header_bytes = unsafe {
            core::slice::from_raw_parts(
                &image_header as *const _ as *const u8,
                mem::size_of::<ImageHeader>(),
            )
        };
        unsafe { self.write_page(partition.origin() - flash_boot::ORIGIN, &header_bytes) };

        let app_offset = partition.boot_target_addr() - flash_boot::ORIGIN;
        for page in 0..image.len().div_ceil(256) {
            let image_offset = page * 256;
            let flash_offset = app_offset + image_offset as u32;
            let end_image_section = image_offset + min(256, image.len() - image_offset);
            unsafe { self.write_page(flash_offset, &image[image_offset..end_image_section]) };
        }
    }
}

impl FlashStorage for TestFlash {
    unsafe fn read_as<T>(&self, offset: u32) -> T {
        let offset = offset as usize;
        assert!(offset.is_multiple_of(4));
        assert!(
            offset < self.mem.borrow().len(),
            "Offset: {offset:0x} is greater than {:0x}",
            self.mem.borrow().len()
        );
        let mut t = MaybeUninit::<T>::uninit();
        let buf = unsafe {
            core::slice::from_raw_parts_mut(t.as_mut_ptr() as *mut u8, core::mem::size_of::<T>())
        };
        buf.copy_from_slice(&self.mem.borrow()[offset..offset + buf.len()]);
        unsafe { t.assume_init() }
    }

    unsafe fn as_slice(&self, offset: u32, len: usize) -> &[u8] {
        let offset = offset as usize;
        unsafe {
            core::slice::from_raw_parts(self.mem.borrow().as_ptr().offset(offset as isize), len)
        }
    }

    unsafe fn erase_sector(&self, flash_offset: u32) {
        const FLASH_RESET_VALUE: u8 = 0xFF;
        assert!(flash_offset.is_multiple_of(256));
        let flash_offset = flash_offset as usize - self.origin_offset;

        for byte in self
            .mem
            .borrow_mut()
            .iter_mut()
            .skip(flash_offset)
            .take(self.sector_size)
        {
            *byte = FLASH_RESET_VALUE;
        }
    }

    unsafe fn write_page(&self, flash_offset: u32, data: &[u8]) {
        println!("Write page flash offset: {flash_offset:0x}");
        let flash_offset = flash_offset as usize;
        const FLASH_PAGE_SIZE: usize = 256;
        assert!(
            flash_offset < self.mem.borrow().len(),
            "Invalid flash offset: {flash_offset:0x} is greater than {:0x}",
            self.mem.borrow().len()
        );
        assert!(data.len() <= FLASH_PAGE_SIZE);
        assert!(flash_offset.rem_euclid(FLASH_PAGE_SIZE) + data.len() <= FLASH_PAGE_SIZE);
        for (idx, byte) in self
            .mem
            .borrow_mut()
            .iter_mut()
            .skip(flash_offset)
            .take(data.len())
            .enumerate()
        {
            *byte = data[idx];
        }
    }
}
