use crate::display::{DisplayCmdSender, COLUMNS};
use embassy_sync::{blocking_mutex::raw::NoopRawMutex, mutex::Mutex};

pub enum DisplayCommand {
    UpdateSingleCell(UpdateSingleCell),
    RowUpdate(RowUpdate),
    RowUpdateRgb(RowUpdateRgb),
    GetDisplayInfo,
    CommitRender,
    SetMonocolorPalette(SetMonocolorPalette),
}

pub struct UpdateSingleCell {
    pub row: u8,
    pub col: u8,
    pub value: bool,
}

pub struct RowUpdate {
    pub row: u8,
    pub row_data: [u8; COLUMNS / 8],
}

pub struct RowUpdateRgb {
    pub row: u8,
}

pub struct SetMonocolorPalette {
    pub color: u16,
}

pub struct DisplayCmdRouter {
    request_sender: DisplayCmdSender,
    rgb_enabled: bool,
    row_data_buffer: &'static Mutex<NoopRawMutex, [u16; COLUMNS]>,
}

impl DisplayCmdRouter {
    pub fn new(
        request_sender: DisplayCmdSender,
        row_data_buffer: &'static Mutex<NoopRawMutex, [u16; COLUMNS]>,
        rgb_enabled: bool,
    ) -> Self {
        Self {
            request_sender,
            row_data_buffer,
            rgb_enabled,
        }
    }

    pub async fn handle_update_single_cell(&self, payload: &[u8]) {
        let row = payload[0];
        let col = payload[1];
        let value = payload[2] != 0;
        self.request_sender
            .send(DisplayCommand::UpdateSingleCell(UpdateSingleCell {
                row,
                col,
                value,
            }))
            .await;
    }

    pub async fn handle_row_update(&self, payload: &[u8]) {
        let row = payload[0];
        let _row_data_len = payload[1];
        let mut row_data = [0u8; COLUMNS / 8];
        row_data
            .iter_mut()
            .zip(&payload[2..])
            .for_each(|(dst, src)| {
                *dst = *src;
            });
        self.request_sender
            .send(DisplayCommand::RowUpdate(RowUpdate { row, row_data }))
            .await;
    }

    pub async fn handle_get_display_info(&self) {
        self.request_sender
            .send(DisplayCommand::GetDisplayInfo)
            .await;
    }

    pub async fn handle_row_update_rgb(&self, payload: &[u8]) {
        let row = payload[0];
        {
            let mut row_buf = self.row_data_buffer.lock().await;
            row_buf
                .iter_mut()
                .zip((2..payload.len()).into_iter().step_by(2))
                .for_each(|(dst, src_idx)| {
                    *dst = u16::from_be_bytes([payload[src_idx], payload[src_idx + 1]])
                });
        }
        if self.rgb_enabled {
            self.request_sender
                .send(DisplayCommand::RowUpdateRgb(RowUpdateRgb { row }))
                .await;
        }
    }

    pub async fn handle_request_commit_render(&self) {
        self.request_sender.send(DisplayCommand::CommitRender).await;
    }

    pub async fn handle_set_monocolor_palette(&self, payload: &[u8]) {
        let color = u16::from_be_bytes([payload[0], payload[1]]);
        self.request_sender
            .send(DisplayCommand::SetMonocolorPalette(SetMonocolorPalette {
                color,
            }))
            .await;
    }
}
