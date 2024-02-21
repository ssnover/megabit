use embedded_graphics::{draw_target::DrawTarget, geometry::Point, Pixel};
use extism_pdk::*;
use megabit_app_sdk::{
    display::{self, render, write_region_rgb, Color, RgbBuffer},
    kv_store,
};
use png::ColorType;
use std::io::BufReader;

mod frames {
    pub const FRAME_0: &[u8] = include_bytes!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/data/",
        "target-0.png"
    ));
    pub const FRAME_1: &[u8] = include_bytes!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/data/",
        "target-1.png"
    ));
    pub const FRAME_2: &[u8] = include_bytes!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/data/",
        "target-2.png"
    ));
    pub const FRAME_3: &[u8] = include_bytes!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/data/",
        "target-3.png"
    ));
    pub const FRAME_4: &[u8] = include_bytes!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/data/",
        "target-4.png"
    ));
    pub const FRAME_5: &[u8] = include_bytes!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/data/",
        "target-5.png"
    ));
    pub const FRAME_6: &[u8] = include_bytes!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/data/",
        "target-6.png"
    ));
    pub const FRAME_7: &[u8] = include_bytes!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/data/",
        "target-7.png"
    ));
    pub const FRAME_8: &[u8] = include_bytes!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/data/",
        "target-8.png"
    ));
    pub const FRAME_9: &[u8] = include_bytes!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/data/",
        "target-9.png"
    ));
    pub const FRAME_10: &[u8] = include_bytes!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/data/",
        "target-10.png"
    ));
    pub const FRAME_11: &[u8] = include_bytes!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/data/",
        "target-11.png"
    ));

    pub const ALL: [&[u8]; 12] = [
        FRAME_0, FRAME_1, FRAME_2, FRAME_3, FRAME_4, FRAME_5, FRAME_6, FRAME_7, FRAME_8, FRAME_9,
        FRAME_10, FRAME_11,
    ];
}

mod vars {
    pub const FRAME_NUMBER: &str = "FRAME";
}

#[plugin_fn]
pub fn setup() -> FnResult<()> {
    kv_store::write(vars::FRAME_NUMBER, 0)?;

    Ok(())
}

#[plugin_fn]
pub fn run() -> FnResult<()> {
    let frame_number: u32 = kv_store::read(vars::FRAME_NUMBER).unwrap().unwrap_or(0);
    let current_frame = BufReader::new(frames::ALL[frame_number as usize]);
    let decoder = png::Decoder::new(current_frame);
    let mut reader = decoder.read_info().unwrap();

    let mut buf = vec![0; reader.output_buffer_size()];
    let info = reader.next_frame(&mut buf).unwrap();
    let image_bytes = &buf[..info.buffer_size()];
    assert!(matches!(info.color_type, ColorType::Indexed));
    assert!(!reader.info().interlaced);
    let palette = reader.info().palette.as_ref().unwrap();

    let display_cfg = display::get_display_info()?;
    let mut display_buffer = RgbBuffer::new(display_cfg.width, display_cfg.height);
    let _ = display_buffer.draw_iter(
        image_bytes
            .into_iter()
            .enumerate()
            .map(|(idx, byte)| [(2 * idx, *byte >> 4), ((2 * idx) + 1, *byte & 0xf)])
            .flatten()
            .map(|(idx, pixel)| {
                Pixel(
                    Point::new(
                        (idx % display_cfg.width) as i32,
                        (idx / display_cfg.width) as i32,
                    ),
                    from_palette(pixel, palette).unwrap(),
                )
            }),
    );

    write_region_rgb(
        (0, 0),
        (display_cfg.width as u32, display_cfg.height as u32),
        Vec::from(display_buffer.get_data()),
    )?;
    render(0..display_cfg.height as u8)?;

    kv_store::write(vars::FRAME_NUMBER, (frame_number + 1) % 12)?;

    Ok(())
}

fn from_palette(index: u8, palette: &[u8]) -> Option<Color> {
    let index = index as usize;
    Some(Color::from_rgb(
        palette[index * 3],
        palette[(index * 3) + 1],
        palette[(index * 3) + 2],
    ))
}
