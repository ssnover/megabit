use embedded_graphics::{
    mono_font::{ascii::FONT_5X8, MonoTextStyle},
    pixelcolor::BinaryColor,
    prelude::*,
};
use extism_pdk::*;
use megabit_app_sdk::{
    display::{self, pack_monocolor_data, Color, MonocolorBuffer},
    megabit_wasm_app, MegabitApp,
};

const COLORS: [Color; 4] = [Color::RED, Color::GREEN, Color::BLUE, Color::WHITE];

#[megabit_wasm_app]
struct HelloWorldApp {
    color_idx: usize,
    is_rgb: bool,
    buffer: MonocolorBuffer,
}

impl MegabitApp for HelloWorldApp {
    fn setup(display_cfg: display::DisplayConfiguration) -> FnResult<Self> {
        if display_cfg.is_rgb {
            display::set_monocolor_palette(Color::RED, Color::BLACK)?;
        }

        let mut buffer = MonocolorBuffer::new(display_cfg.width, display_cfg.height);

        let text = embedded_graphics::text::Text::new(
            "Hello",
            Point::new(0, 7),
            MonoTextStyle::new(&FONT_5X8, BinaryColor::On),
        );
        text.draw(&mut buffer).unwrap();

        let text = embedded_graphics::text::Text::new(
            "world!",
            Point::new(0, 7 + 8),
            MonoTextStyle::new(&FONT_5X8, BinaryColor::On),
        );
        text.draw(&mut buffer).unwrap();

        Ok(Self {
            color_idx: 0,
            is_rgb: display_cfg.is_rgb,
            buffer,
        })
    }

    fn run(&mut self) -> FnResult<()> {
        let rows_to_update = 0..self.buffer.size().height as u8;
        let packed_data = pack_monocolor_data(self.buffer.get_data());

        display::write_region(
            (0, 0),
            (self.buffer.size().width, self.buffer.size().height),
            packed_data,
        )?;
        display::render(rows_to_update)?;

        if self.is_rgb {
            self.color_idx = (self.color_idx + 1) % COLORS.len();
            display::set_monocolor_palette(COLORS[self.color_idx], Color::BLACK)?;
        }
        Ok(())
    }
}
