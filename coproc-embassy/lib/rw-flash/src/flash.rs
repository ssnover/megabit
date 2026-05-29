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
