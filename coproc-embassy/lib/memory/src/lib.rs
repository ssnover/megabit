#![no_std]

pub mod flash_boot {
    pub const ORIGIN: u32 = 0x10000000;
    pub const LENGTH: u32 = 32 * 1024;
}

pub mod flash_app {
    use core::hint::unreachable_unchecked;

    use super::flash_boot;

    pub const APP_LENGTH: u32 = 992 * 1024;

    pub const ORIGIN: u32 = flash_boot::ORIGIN + flash_boot::LENGTH;
    pub const LENGTH: u32 = APP_LENGTH * 2;

    // The linker script prepends each application image with an image header
    // and then aligns the vector table at 256 byte increments. The image header
    // is less than 256 bytes, so the table is offset at just 256 bytes
    pub const VECTOR_TABLE_OFFSET: u32 = 256;

    #[repr(u8)]
    #[derive(Clone, Copy, Debug)]
    pub enum AppPartition {
        A = 0,
        B = 1,
    }

    impl AppPartition {
        pub fn from_u8(num: u8) -> Self {
            match num {
                0 => AppPartition::A,
                1 => AppPartition::B,
                _ => unsafe { unreachable_unchecked() },
            }
        }

        pub fn origin(&self) -> u32 {
            get_app_origin(*self)
        }

        pub fn boot_target_addr(&self) -> u32 {
            get_app_origin(*self) + VECTOR_TABLE_OFFSET
        }
    }

    pub fn get_app_origin(part: AppPartition) -> u32 {
        match part {
            AppPartition::A => ORIGIN,
            AppPartition::B => ORIGIN + APP_LENGTH,
        }
    }
}

pub mod flash_nvs {
    use super::flash_app;

    pub const ORIGIN: u32 = flash_app::ORIGIN + flash_app::LENGTH;
    pub const LENGTH: u32 = 32 * 1024;
}
