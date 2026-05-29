use core::mem;

use memory::{
    flash_app::{self, AppPartition, get_app_origin},
    flash_boot,
};

use crate::flash::FlashStorage;

pub const IMAGE_MAGIC: u32 = 0x494d4700; // 'IMG\0'
pub const IMAGE_HEADER_VERSION: u32 = 0;

#[repr(C, packed)]
pub struct ImageHeader {
    magic: u32,
    header_version: u32,
    image_version: u32,
    app_size: u32,
    crc32: u32,
    _reserved: [u8; 236],
}

const _: () = assert!(mem::size_of::<ImageHeader>() == 256);

impl ImageHeader {
    pub fn new(image_version: u32, app_size: u32, crc32: u32) -> Self {
        Self {
            magic: IMAGE_MAGIC,
            header_version: IMAGE_HEADER_VERSION,
            image_version,
            app_size,
            crc32,
            _reserved: [0u8; 236],
        }
    }

    pub fn is_valid(&self) -> bool {
        self.magic == IMAGE_MAGIC
            && self.app_size < (flash_app::APP_LENGTH - flash_app::VECTOR_TABLE_OFFSET)
    }

    #[cfg(feature = "with-std")]
    pub fn to_bytes(&self) -> Vec<u8> {
        use std::io::Write;

        let mut out = vec![0u8; core::mem::size_of::<Self>()];
        let mut writer = std::io::Cursor::new(&mut out);
        writer.write(&self.magic.to_le_bytes()).unwrap();
        writer.write(&self.header_version.to_le_bytes()).unwrap();
        writer.write(&self.image_version.to_le_bytes()).unwrap();
        writer.write(&self.app_size.to_le_bytes()).unwrap();
        writer.write(&self.crc32.to_le_bytes()).unwrap();

        out
    }
}

pub fn read_image_header(partition: AppPartition, f: &impl FlashStorage) -> ImageHeader {
    let flash_offset = get_app_origin(partition) - flash_boot::ORIGIN;
    unsafe { f.read_as::<ImageHeader>(flash_offset) }
}

pub fn verify_image(partition: AppPartition, header: &ImageHeader, f: &impl FlashStorage) -> bool {
    let flash_offset = get_app_origin(partition) - flash_boot::ORIGIN;
    let image_offset = flash_offset + flash_app::VECTOR_TABLE_OFFSET;
    let image_bytes = unsafe { f.as_slice(image_offset, header.app_size as usize) };
    let crc = crc32::crc32(image_bytes);
    crc == header.crc32
}
