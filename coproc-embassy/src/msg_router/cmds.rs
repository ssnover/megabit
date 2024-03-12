pub mod update_row {
    pub const MAJOR: u8 = 0xa0;
    pub const MINOR: u8 = 0x00;
}

pub mod update_row_response {
    pub const MAJOR: u8 = 0xa0;
    pub const MINOR: u8 = 0x01;
}

pub mod update_row_rgb {
    pub const MAJOR: u8 = 0xa0;
    pub const MINOR: u8 = 0x02;
}

pub mod update_row_rgb_response {
    pub const MAJOR: u8 = 0xa0;
    pub const MINOR: u8 = 0x03;
}

pub mod get_display_info {
    pub const MAJOR: u8 = 0xa0;
    pub const MINOR: u8 = 0x04;
}

pub mod get_display_info_response {
    pub const MAJOR: u8 = 0xa0;
    pub const MINOR: u8 = 0x05;
}

pub mod request_commit_render {
    pub const MAJOR: u8 = 0xa0;
    pub const MINOR: u8 = 0x06;
}

pub mod commit_render_response {
    pub const MAJOR: u8 = 0xa0;
    pub const MINOR: u8 = 0x07;
}

pub mod set_monocolor_palette {
    pub const MAJOR: u8 = 0xa0;
    pub const MINOR: u8 = 0x08;
}

pub mod set_monocolor_palette_response {
    pub const MAJOR: u8 = 0xa0;
    pub const MINOR: u8 = 0x09;
}

pub mod set_single_cell {
    pub const MAJOR: u8 = 0xa0;
    pub const MINOR: u8 = 0x50;
}

pub mod set_single_cell_response {
    pub const MAJOR: u8 = 0xa0;
    pub const MINOR: u8 = 0x51;
}

pub mod set_led_state {
    pub const MAJOR: u8 = 0xde;
    pub const MINOR: u8 = 0x00;
}

pub mod set_led_state_response {
    pub const MAJOR: u8 = 0xde;
    pub const MINOR: u8 = 0x01;
}

pub mod set_rgb_state {
    pub const MAJOR: u8 = 0xde;
    pub const MINOR: u8 = 0x02;
}

pub mod set_rgb_state_response {
    pub const MAJOR: u8 = 0xde;
    pub const MINOR: u8 = 0x03;
}

pub mod report_button_press {
    pub const MAJOR: u8 = 0xde;
    pub const MINOR: u8 = 0x04;
}

pub mod ping {
    pub const MAJOR: u8 = 0xde;
    pub const MINOR: u8 = 0xfe;
}

pub mod ping_response {
    pub const MAJOR: u8 = 0xde;
    pub const MINOR: u8 = 0xff;
}
