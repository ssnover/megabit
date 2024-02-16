use extism_pdk::*;

#[host_fn]
extern "ExtismHost" {
    pub fn write_region(
        position_x: u32,
        position_y: u32,
        width: u32,
        height: u32,
        input_data: Vec<u8>,
    ) -> ();
    pub fn render(rows_to_update: Vec<u8>) -> ();
    pub fn set_monocolor_palette(on_color: u32, off_color: u32) -> ();
    pub fn get_display_info() -> Vec<u8>;

    pub fn kv_store_read(key: String) -> Vec<u8>;
    pub fn kv_store_write(key: String, value: Vec<u8>) -> ();

    pub fn log(level: u32, line: String) -> ();
}
