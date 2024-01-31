use super::PersistentData;

extism::host_fn!(pub write_region(user_data: PersistentData; position_x: u32, position_y: u32, width: u32, height: u32, buffer_data: Vec<u8>) {
    let data = user_data.get()?;
    let data = data.lock().unwrap();
    let mut screen_buffer = data.screen_buffer.borrow_mut();
    host::write_region(&mut screen_buffer, position_x, position_y, width, height, buffer_data)
});

extism::host_fn!(pub render(user_data: PersistentData; rows_to_update: Vec<u8>) {
    let data = user_data.get()?;
    let data = data.lock().unwrap();
    let screen_buffer = data.screen_buffer.borrow();
    let serial_conn = data.serial_conn.clone();
    host::render(&screen_buffer, serial_conn, rows_to_update)
});

mod host {
    use crate::serial::SyncSerialConnection;

    use super::super::{ScreenBuffer, SCREEN_HEIGHT, SCREEN_WIDTH};

    pub fn write_region(
        screen_buffer: &mut ScreenBuffer,
        position_x: u32,
        position_y: u32,
        width: u32,
        height: u32,
        buffer_data: Vec<u8>,
    ) -> Result<(), extism::Error> {
        if (position_x + width <= SCREEN_WIDTH as u32)
            || (position_y + height <= SCREEN_HEIGHT as u32)
        {
            for row in position_y..(position_y + height) {
                for col in position_x..(position_x + width) {
                    let idx = (col - position_x) + (width * (row - position_y));
                    screen_buffer[row as usize][col as usize] =
                        (buffer_data[(idx / 8) as usize] & (1 << (idx % 8))) != 0;
                }
            }
            Ok(())
        } else {
            Err(std::io::Error::from(std::io::ErrorKind::InvalidInput).into())
        }
    }

    pub fn render(
        screen_buffer: &ScreenBuffer,
        serial_conn: SyncSerialConnection,
        rows: Vec<u8>,
    ) -> Result<(), extism::Error> {
        for row_number in rows {
            let row_number = row_number as usize;
            if (0..SCREEN_HEIGHT).contains(&row_number) {
                let row_data = &screen_buffer[row_number];
                serial_conn.update_row(row_number as u8, Vec::from(row_data))?;
            }
        }

        Ok(())
    }
}
