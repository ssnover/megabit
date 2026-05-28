#[cfg(feature = "hw")]
pub use hw::*;

pub trait FlashStorage {
    /// Read an element of data from flash storage. This requires that the type
    /// has a representation that ensures that it is consistent across compiler
    /// versions. Types should be primitive or be `#[repr(C, packed)]`
    unsafe fn read_as<T>(&self, offset: u32) -> T;

    /// Creates a slice from a region of flash memory.
    unsafe fn as_slice(&self, offset: u32, len: usize) -> &[u8];

    /// Erases an entire sector of NVS flash
    unsafe fn erase_sector(&self, flash_offset: u32);

    /// Write data to flash. The `flash_offset` must be word-aligned to 4 byte
    /// address. The `data` must not be longer so as to write over the end of
    /// the page (256 bytes).
    unsafe fn write_page(&self, flash_offset: u32, data: &[u8]);
}

#[cfg(feature = "hw")]
mod hw {

    use super::FlashStorage;
    use core::mem::{self, MaybeUninit};
    use memory::flash_boot;
    use rp2040_hal::rom_data;

    pub struct PicoFlash {
        sector_size: usize,
    }

    impl PicoFlash {
        pub fn new(sector_size: usize) -> Self {
            Self { sector_size }
        }
    }

    impl FlashStorage for PicoFlash {
        unsafe fn read_as<T>(&self, offset: u32) -> T {
            let mut t = MaybeUninit::<T>::uninit();
            let buf = unsafe {
                core::slice::from_raw_parts_mut(t.as_mut_ptr() as *mut u8, mem::size_of::<T>())
            };
            read(offset, buf);
            unsafe { t.assume_init() }
        }

        unsafe fn as_slice(&self, offset: u32, len: usize) -> &[u8] {
            core::slice::from_raw_parts((offset + flash_boot::ORIGIN) as *const u8, len)
        }

        unsafe fn erase_sector(&self, flash_offset: u32) {
            erase_sector(flash_offset, self.sector_size)
        }

        unsafe fn write_page(&self, flash_offset: u32, data: &[u8]) {
            write_page(flash_offset, data)
        }
    }

    fn read(offset: u32, buf: &mut [u8]) {
        let addr = offset + flash_boot::ORIGIN;
        let src = unsafe { core::slice::from_raw_parts(addr as *const u8, buf.len()) };
        buf.copy_from_slice(src);
    }

    #[inline(never)]
    #[unsafe(link_section = ".time_critical.flash_erase_sector")]
    unsafe fn erase_sector(flash_offset: u32, sector_size: usize) {
        cortex_m::interrupt::free(|_| unsafe {
            rom_data::connect_internal_flash();
            rom_data::flash_exit_xip();
            rom_data::flash_range_erase(flash_offset, sector_size, 1 << 12, 0x20);
            rom_data::flash_flush_cache();
            rom_data::flash_enter_cmd_xip();
        });
    }

    #[inline(never)]
    #[unsafe(link_section = ".time_critical.flash_write_page")]
    unsafe fn write_page(flash_offset: u32, data: &[u8]) {
        const FLASH_PAGE_SIZE: usize = 256;
        debug_assert!(data.len() <= FLASH_PAGE_SIZE);
        debug_assert!((data.as_ptr() as *const u32).is_aligned());
        // Ensures that we don't write over the end of the page
        debug_assert!(
            (flash_offset as usize).rem_euclid(FLASH_PAGE_SIZE) + data.len() <= FLASH_PAGE_SIZE
        );

        cortex_m::interrupt::free(|_| unsafe {
            rom_data::connect_internal_flash();
            rom_data::flash_exit_xip();
            rom_data::flash_range_program(flash_offset, data.as_ptr(), data.len());
            rom_data::flash_flush_cache();
            rom_data::flash_enter_cmd_xip();
        });
    }
}
