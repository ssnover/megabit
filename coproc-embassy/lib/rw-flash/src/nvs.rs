use core::mem;

use crate::flash;
use memory::{flash_boot, flash_nvs};

const SECTOR_COUNT: usize = 2;
pub const SECTOR_SIZE: usize = flash_nvs::LENGTH as usize / SECTOR_COUNT;
const RECORD_SIZE: usize = 64;
const SECTOR_MAGIC: u32 = 0x4e565300; // "NVS\0"
const RECORD_MAGIC: u32 = 0x52454300; // "REC\0"

const RECORDS_PER_SECTOR: usize = (SECTOR_SIZE - mem::size_of::<SectorHeader>()) / RECORD_SIZE;

#[repr(C, packed)]
#[derive(Clone, Copy)]
pub struct SectorHeader {
    magic: u32,
    generation: u32,
    crc32: u32,
    _reserved: [u8; 256 - 12],
}
// Should be equivalent to the flash page size
const _: () = assert!(mem::size_of::<SectorHeader>() == 256);

impl SectorHeader {
    pub fn new(generation: u32) -> Self {
        let mut header = Self {
            magic: SECTOR_MAGIC,
            generation,
            crc32: 0,
            _reserved: [0; 244],
        };
        header.crc32 = header.compute_crc();
        header
    }

    fn compute_crc(&self) -> u32 {
        // Calculate the CRC of all members except the CRC field, which is last
        let bytes = unsafe {
            core::slice::from_raw_parts(
                self as *const _ as *const u8,
                mem::offset_of!(SectorHeader, crc32),
            )
        };
        crc32::crc32(bytes)
    }

    fn is_valid(&self) -> bool {
        self.magic == SECTOR_MAGIC && self.crc32 == self.compute_crc()
    }
}

#[repr(C, packed)]
#[derive(Clone, Copy)]
pub struct BootState {
    magic: u32,
    pub bootloader_version: u32,
    pub active_partition: u8,
    pub update_pending: bool,
    pub boot_attempts: u8,
    _pad: u8,
    pub update_version: u32,
    pub crc32: u32,
    _reserved: [u8; 44],
}

const _: () = assert!(mem::size_of::<BootState>() == RECORD_SIZE);

impl BootState {
    pub fn default(bootloader_version: u32) -> Self {
        let mut state = Self {
            magic: RECORD_MAGIC,
            bootloader_version,
            active_partition: 0,
            update_pending: false,
            boot_attempts: 0,
            _pad: 0,
            update_version: 0,
            crc32: 0,
            _reserved: [0; 44],
        };
        state.crc32 = state.compute_crc();
        state
    }

    pub fn increment_boot_attempts(&mut self) {
        self.boot_attempts += 1;
        self.crc32 = self.compute_crc();
    }

    pub fn revert(&mut self) {
        if self.active_partition == 0 {
            self.active_partition = 1;
        } else {
            self.active_partition = 0;
        }

        self.update_pending = false;
        self.boot_attempts = 0;
        self.crc32 = self.compute_crc();
    }

    fn is_valid(&self) -> bool {
        self.magic == RECORD_MAGIC && self.crc32 == self.compute_crc()
    }

    fn compute_crc(&self) -> u32 {
        // Calculate the CRC of all members except the CRC field, which is last
        let bytes = unsafe {
            core::slice::from_raw_parts(
                self as *const _ as *const u8,
                mem::offset_of!(BootState, crc32),
            )
        };
        crc32::crc32(bytes)
    }
}

pub struct NvsHandle<'a, F: flash::FlashStorage> {
    pub flash: &'a F,
}

impl<'a, F: flash::FlashStorage> NvsHandle<'a, F> {
    pub fn read_boot_state(&self) -> Option<BootState> {
        let sector = self.find_active_sector()?;
        self.read_latest_record(sector)
    }

    pub fn write_boot_state(&self, boot_state: &BootState) {
        let sector = self.find_active_sector().unwrap_or(0);
        let next_entry = self.find_next_free_entry(sector);

        if next_entry >= RECORDS_PER_SECTOR {
            let other = 1 - sector;
            self.compact(sector, other, boot_state);
        } else {
            self.append_record(sector, next_entry, boot_state);
        }
    }

    fn read_sector_header(&self, sector: usize) -> Option<SectorHeader> {
        let base_addr = sector_base_offset(sector);
        let header = unsafe { self.flash.read_as::<SectorHeader>(base_addr) };
        header.is_valid().then_some(header)
    }

    fn find_active_sector(&self) -> Option<usize> {
        let header0 = self.read_sector_header(0);
        let header1 = self.read_sector_header(1);

        match (header0, header1) {
            (Some(a), Some(b)) => {
                if a.generation.wrapping_sub(b.generation) < u32::MAX / 2 {
                    Some(0)
                } else {
                    Some(1)
                }
            }
            (Some(_), None) => Some(0),
            (None, Some(_)) => None,
            (None, None) => None,
        }
    }

    fn find_next_free_entry(&self, sector: usize) -> usize {
        for entry in 0..RECORDS_PER_SECTOR {
            let offset = record_offset(sector, entry);
            // The magic number is the first field
            const _: () = assert!(mem::offset_of!(BootState, magic) == 0);
            let magic = unsafe { self.flash.read_as::<u32>(offset) };
            if magic == 0xffffffff {
                return entry;
            }
        }
        // Indicates the sector is full
        RECORDS_PER_SECTOR
    }

    fn read_record(&self, sector: usize, entry: usize) -> Option<BootState> {
        let offset = record_offset(sector, entry);
        let record = unsafe { self.flash.read_as::<BootState>(offset) };
        record.is_valid().then_some(record)
    }

    fn read_latest_record(&self, sector: usize) -> Option<BootState> {
        let mut latest = None;
        for entry in 0..RECORDS_PER_SECTOR {
            if let Some(record) = self.read_record(sector, entry) {
                latest = Some(record);
            }
        }

        latest
    }

    fn append_record(&self, sector: usize, entry: usize, state: &BootState) {
        let offset = record_offset(sector, entry);
        let bytes =
            unsafe { core::slice::from_raw_parts(state as *const _ as *const u8, RECORD_SIZE) };
        unsafe { self.flash.write_page(offset, bytes) };
    }

    fn compact(&self, from_sector: usize, to_sector: usize, new_state: &BootState) {
        let old_generation = self
            .read_sector_header(from_sector)
            .map(|header| header.generation)
            .unwrap_or(0);

        let to_base = sector_base_offset(to_sector);
        unsafe { self.flash.erase_sector(to_base) };

        let header = SectorHeader::new(old_generation.wrapping_add(1));
        let header_bytes = unsafe {
            core::slice::from_raw_parts(
                &header as *const _ as *const u8,
                mem::size_of::<SectorHeader>(),
            )
        };
        unsafe { self.flash.write_page(to_base, header_bytes) };

        self.append_record(to_sector, 0, new_state);

        let from_base = sector_base_offset(from_sector);
        unsafe { self.flash.erase_sector(from_base) };
    }
}

fn sector_base_offset(sector: usize) -> u32 {
    (flash_nvs::ORIGIN - flash_boot::ORIGIN) + (sector * SECTOR_SIZE) as u32
}

fn record_offset(sector: usize, entry: usize) -> u32 {
    sector_base_offset(sector)
        + mem::size_of::<SectorHeader>() as u32
        + (entry * RECORD_SIZE) as u32
}
