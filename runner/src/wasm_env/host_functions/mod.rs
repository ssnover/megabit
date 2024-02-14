use super::PersistentData;
use extism::UserData;

mod display;
mod kv_store;

pub fn with_host_functions<'a>(
    builder: extism::PluginBuilder<'a>,
    user_data: &UserData<PersistentData>,
) -> extism::PluginBuilder<'a> {
    with_screen_functions(with_kv_functions(builder, user_data), user_data).with_function(
        "log",
        [extism::PTR, extism::PTR],
        [extism::PTR],
        extism::UserData::new(()),
        log,
    )
}

pub fn with_screen_functions<'a>(
    builder: extism::PluginBuilder<'a>,
    user_data: &UserData<PersistentData>,
) -> extism::PluginBuilder<'a> {
    builder
        .with_function(
            "write_region",
            [
                extism::PTR,
                extism::PTR,
                extism::PTR,
                extism::PTR,
                extism::PTR,
            ],
            [extism::PTR],
            user_data.clone(),
            write_region,
        )
        .with_function(
            "render",
            [extism::PTR],
            [extism::PTR],
            user_data.clone(),
            render,
        )
        .with_function(
            "set_monocolor_palette",
            [extism::PTR, extism::PTR],
            [extism::PTR],
            user_data.clone(),
            set_monocolor_palette,
        )
        .with_function(
            "get_display_info",
            [],
            [extism::PTR],
            user_data.clone(),
            get_display_info,
        )
}

pub fn with_kv_functions<'a>(
    builder: extism::PluginBuilder<'a>,
    user_data: &UserData<PersistentData>,
) -> extism::PluginBuilder<'a> {
    builder
        .with_function(
            "kv_store_read",
            [extism::PTR],
            [extism::PTR],
            user_data.clone(),
            kv_store_read,
        )
        .with_function(
            "kv_store_write",
            [extism::PTR, extism::PTR],
            [extism::PTR],
            user_data.clone(),
            kv_store_write,
        )
}

extism::host_fn!(pub write_region(user_data: PersistentData; position_x: u32, position_y: u32, width: u32, height: u32, buffer_data: Vec<u8>) {
    let data = user_data.get()?;
    let data = data.lock().unwrap();
    let mut screen_buffer = data.screen_buffer.borrow_mut();
    display::write_region(&mut screen_buffer, position_x, position_y, width, height, buffer_data)
});

extism::host_fn!(pub render(user_data: PersistentData; rows_to_update: Vec<u8>) {
    let data = user_data.get()?;
    let data = data.lock().unwrap();
    let screen_buffer = data.screen_buffer.borrow();
    let serial_conn = data.serial_conn.clone();
    display::render(&screen_buffer, serial_conn, rows_to_update)
});

extism::host_fn!(pub set_monocolor_palette(user_data: PersistentData; on_color: u32, off_color: u32) {
    let data = user_data.get()?;
    let data = data.lock().unwrap();
    let mut screen_buffer = data.screen_buffer.borrow_mut();
    display::set_monocolor_palette(&mut screen_buffer, (on_color & 0xffff) as u16, (off_color & 0xffff) as u16)
});

extism::host_fn!(pub get_display_info(user_data: PersistentData;) -> Vec<u8> {
    let data = user_data.get()?;
    let data = data.lock().unwrap();
    let screen_buffer = data.screen_buffer.borrow();
    let config = display::get_display_info(&screen_buffer)?;
    Ok([&(config.width as u32).to_be_bytes()[..], &(config.height as u32).to_be_bytes()[..], &(if config.is_rgb { 1u8 } else {0u8 }).to_be_bytes()[..]].concat())
});

extism::host_fn!(pub kv_store_read(user_data: PersistentData; key: String) -> Vec<u8> {
    let data = user_data.get()?;
    let data = data.lock().unwrap();
    let kv_store = data.kv_store.borrow();
    kv_store::read(&kv_store, key)
});

extism::host_fn!(pub kv_store_write(user_data: PersistentData; key: String, value: Vec<u8>) {
    let data = user_data.get()?;
    let data = data.lock().unwrap();
    let mut kv_store = data.kv_store.borrow_mut();
    kv_store::write(&mut kv_store, key, value)
});

extism::host_fn!(pub log(level: u32, line: String) {
    host::log(level, line)
});

mod host {
    pub fn log(level: u32, line: String) -> Result<(), extism::Error> {
        match level {
            0 => tracing::trace!("{line}"),
            1 => tracing::debug!("{line}"),
            2 => tracing::info!("{line}"),
            3 => tracing::warn!("{line}"),
            4 => tracing::error!("{line}"),
            level => {
                tracing::error!("Got a log call from app with invalid log level: {level}");
            }
        }

        Ok(())
    }
}
