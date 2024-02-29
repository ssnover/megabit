use crate::{
    msg_router::display_cmd_router::{DisplayCommand, RowUpdate, UpdateSingleCell},
    usb::UsbResponder,
};
use core::future::Future;
use embassy_sync::{blocking_mutex::raw::NoopRawMutex, channel::Receiver};

mod driver;
pub use driver::{DotMatrix, COLUMNS, COLUMN_DATA_SIZE, ROWS};

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
    cmd_rx: Receiver<'static, NoopRawMutex, DisplayCommand, 1>,
}

impl<D: DotMatrixDriver<COLUMN_DATA_SIZE>, R: UsbResponder + 'static> DisplayCommandHandler<D, R> {
    pub fn new(
        driver: D,
        responder: &'static R,
        cmd_rx: Receiver<'static, NoopRawMutex, DisplayCommand, 1>,
    ) -> Self {
        Self {
            driver,
            responder,
            cmd_rx,
        }
    }

    pub async fn try_handle_cmd(&mut self) {
        while let Ok(cmd) = self.cmd_rx.try_receive() {
            self.handle_cmd(&cmd).await
        }
    }

    async fn handle_cmd(&mut self, cmd: &DisplayCommand) {
        match cmd {
            DisplayCommand::UpdateSingleCell(UpdateSingleCell { row, col, value }) => {
                self.driver
                    .set_pixel(*row as usize, *col as usize, *value)
                    .await
                    .unwrap();
                let response_buf = [0xa0, 0x51];
                self.responder.send(&response_buf).await.unwrap();
            }
            DisplayCommand::RowUpdate(RowUpdate { row, row_data }) => {
                self.driver
                    .update_row(*row as usize, *row_data)
                    .await
                    .unwrap();
                let response_buf = [0xa0, 0x01, 0x00];
                self.responder.send(&response_buf).await.unwrap();
            }
        }
    }
}
