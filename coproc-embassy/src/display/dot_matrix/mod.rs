use super::DisplayCmdReceiver;
use crate::{
    msg_router::{
        cmds::*,
        display_cmd_router::{DisplayCommand, RowUpdate, RowUpdateRgb, UpdateSingleCell},
    },
    usb::UsbResponder,
};
use core::future::Future;

mod driver;
pub use driver::{DotMatrix, COLUMNS, COLUMN_DATA_SIZE, ROWS};

pub const DISPLAY_CMD_QUEUE_SIZE: usize = 8;

pub trait DotMatrixDriver<const COLUMN_DATA_SIZE: usize> {
    type Error: core::fmt::Debug;

    fn set_pixel(
        &mut self,
        row: usize,
        col: usize,
        state: bool,
    ) -> impl Future<Output = Result<(), Self::Error>>;

    fn update_row(
        &mut self,
        row: usize,
        row_data: [u8; COLUMN_DATA_SIZE],
    ) -> impl Future<Output = Result<(), Self::Error>>;
}

pub struct DisplayCommandHandler<D: DotMatrixDriver<COLUMN_DATA_SIZE>, R: UsbResponder + 'static> {
    driver: D,
    responder: &'static R,
    cmd_rx: DisplayCmdReceiver,
}

impl<D: DotMatrixDriver<COLUMN_DATA_SIZE>, R: UsbResponder + 'static> DisplayCommandHandler<D, R> {
    pub fn new(driver: D, responder: &'static R, cmd_rx: DisplayCmdReceiver) -> Self {
        Self {
            driver,
            responder,
            cmd_rx,
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
                    .set_pixel(*row as usize, *col as usize, *value)
                    .await
                    .unwrap();
                let response_buf = [
                    set_single_cell_response::MAJOR,
                    set_single_cell_response::MINOR,
                    0x00,
                ];
                self.responder.send(&response_buf).await.unwrap();
            }
            DisplayCommand::RowUpdate(RowUpdate { row, row_data }) => {
                self.driver
                    .update_row(*row as usize, *row_data)
                    .await
                    .unwrap();
                let response_buf = [update_row_response::MAJOR, update_row_response::MINOR, 0x00];
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
                response_buf[10] = 0x00; // false
                self.responder.send(&response_buf).await.unwrap();
            }
            DisplayCommand::CommitRender => {
                let response_buf = [
                    commit_render_response::MAJOR,
                    commit_render_response::MINOR,
                    0x00,
                ];
                self.responder.send(&response_buf).await.unwrap();
            }
            DisplayCommand::SetMonocolorPalette(_) => {
                let response_buf = [
                    set_monocolor_palette_response::MAJOR,
                    set_monocolor_palette_response::MINOR,
                    0x01,
                ];
                self.responder.send(&response_buf).await.unwrap();
            }
        }
    }
}
