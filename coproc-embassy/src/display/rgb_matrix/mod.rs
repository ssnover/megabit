use super::DisplayCmdReceiver;
use crate::{
    msg_router::{
        cmds::*,
        display_cmd_router::{DisplayCommand, RowUpdate, RowUpdateRgb, UpdateSingleCell},
    },
    usb::UsbResponder,
};

mod driver;
pub use driver::{DriverHandle, DriverPins, WaveshareDriver};

pub const COLUMNS: usize = 64;
pub const ROWS: usize = 32;
pub const DISPLAY_CMD_QUEUE_SIZE: usize = 8;

pub struct DisplayCommandHandler<R: UsbResponder + 'static> {
    responder: &'static R,
    cmd_rx: DisplayCmdReceiver,
    monocolor: u16,
    driver: DriverHandle,
}

impl<R: UsbResponder + 'static> DisplayCommandHandler<R> {
    pub fn new(
        responder: &'static R,
        cmd_rx: DisplayCmdReceiver,
        (r, g, b): (u8, u8, u8),
        driver: DriverHandle,
    ) -> Self {
        let monocolor =
            (((r & 0xf8) as u16) << 7) | (((g & 0xf8) as u16) << 2) | ((b & 0xf8) as u16 >> 3);
        Self {
            responder,
            cmd_rx,
            monocolor,
            driver,
        }
    }

    pub async fn run(&mut self) {
        loop {
            let cmd = self.cmd_rx.receive().await;
            self.handle_cmd(&cmd).await;
        }
    }

    async fn handle_cmd(&mut self, cmd: &DisplayCommand) {
        match cmd {
            DisplayCommand::UpdateSingleCell(UpdateSingleCell { row, col, value }) => {
                self.driver
                    .set_cell(*row, *col, if *value { self.monocolor } else { 0x00 })
                    .await;
                let response_buf = [
                    set_single_cell_response::MAJOR,
                    set_single_cell_response::MINOR,
                    0x00,
                ];
                self.responder.send(&response_buf).await.unwrap();
            }
            DisplayCommand::RowUpdate(RowUpdate {
                row: _,
                row_data: _,
            }) => {
                let response_buf = [update_row_response::MAJOR, update_row_response::MINOR, 0x01];
                self.responder.send(&response_buf).await.unwrap();
            }
            DisplayCommand::RowUpdateRgb(RowUpdateRgb { row: _ }) => {
                let response_buf = [
                    update_row_rgb_response::MAJOR,
                    update_row_rgb_response::MINOR,
                    0x01,
                ];
                self.responder.send(&response_buf).await.unwrap();
            }
            DisplayCommand::GetDisplayInfo => {
                let mut response_buf = [0u8; 11];
                response_buf[0] = get_display_info_response::MAJOR;
                response_buf[1] = get_display_info_response::MINOR;
                response_buf[2..6]
                    .iter_mut()
                    .zip(((COLUMNS) as u32).to_be_bytes().into_iter())
                    .for_each(|(dst, src)| *dst = src);
                response_buf[6..10]
                    .iter_mut()
                    .zip(((ROWS) as u32).to_be_bytes().into_iter())
                    .for_each(|(dst, src)| *dst = src);
                response_buf[10] = 0x01; // true
                self.responder.send(&response_buf).await.unwrap();
            }
            DisplayCommand::CommitRender => {
                let response_buf = [
                    commit_render_response::MAJOR,
                    commit_render_response::MINOR,
                    0x01,
                ];
                self.responder.send(&response_buf).await.unwrap();
            }
        }
    }
}
