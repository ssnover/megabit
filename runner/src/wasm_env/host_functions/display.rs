use super::super::ScreenBuffer;
use crate::{
    display::{DisplayConfiguration, MonocolorPalette},
    serial::SyncSerialConnection,
};

pub fn write_region(
    screen_buffer: &mut ScreenBuffer,
    position_x: u32,
    position_y: u32,
    width: u32,
    height: u32,
    buffer_data: Vec<u8>,
) -> Result<(), extism::Error> {
    for row in position_y..(position_y + height) {
        for col in position_x..(position_x + width) {
            let idx = (col - position_x) + (width * (row - position_y));
            screen_buffer.set_cell(
                row as usize,
                col as usize,
                (buffer_data[(idx / 8) as usize] & (1 << (idx % 8))) != 0,
            )?;
        }
    }
    Ok(())
}

pub fn write_region_rgb(
    screen_buffer: &mut ScreenBuffer,
    position_x: u32,
    position_y: u32,
    width: u32,
    height: u32,
    buffer_data: Vec<u8>,
) -> Result<(), extism::Error> {
    for row in position_y..(position_y + height) {
        for col in position_x..(position_x + width) {
            let idx = (((col - position_x) + (width * (row - position_y))) * 2) as usize;
            let value = u16::from_be_bytes(buffer_data[idx..idx + 2].try_into().unwrap());
            screen_buffer.set_cell_rgb(row as usize, col as usize, value)?;
        }
    }
    Ok(())
}

pub fn render(
    screen_buffer: &mut ScreenBuffer,
    serial_conn: SyncSerialConnection,
    rows: Vec<u8>,
) -> Result<(), extism::Error> {
    for row_number in rows {
        if screen_buffer.is_rgb() {
            let (row_data, dirty) = screen_buffer.get_row_rgb(row_number as usize)?;
            if dirty {
                serial_conn.update_row_rgb(row_number, row_data)?;
            }
        } else {
            let (row_data, dirty) = screen_buffer.get_row(row_number as usize)?;
            if dirty {
                serial_conn.update_row(row_number, row_data)?;
            }
        }
    }
    screen_buffer.clear_dirty_status();
    serial_conn.commit_render()?;

    Ok(())
}

pub fn set_monocolor_palette(
    screen_buffer: &mut ScreenBuffer,
    serial_conn: SyncSerialConnection,
    on_color: u16,
    off_color: u16,
) -> Result<(), extism::Error> {
    screen_buffer.set_palette(MonocolorPalette::new(on_color, off_color))?;
    serial_conn.set_monocolor_palette(on_color)?;

    Ok(())
}

pub fn get_display_info(
    screen_buffer: &ScreenBuffer,
) -> Result<DisplayConfiguration, extism::Error> {
    Ok(screen_buffer.display_config())
}
