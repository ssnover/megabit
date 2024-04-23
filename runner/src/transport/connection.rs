use super::{msg_inbox::InboxHandle, tasks::SerialTaskRequest};
use async_channel::Sender;
use megabit_serial_protocol::*;
use std::{
    io,
    time::{Duration, Instant},
};
use tokio::sync::oneshot;

#[derive(Clone)]
pub struct Connection {
    pub actor_tx: Sender<SerialTaskRequest>,
    pub inbox_handle: InboxHandle,
}

impl Connection {
    async fn send_message(&self, msg: SerialMessage) -> io::Result<()> {
        Self::send_message_inner(&self.actor_tx, msg).await
    }

    pub async fn send_message_inner(
        actor_tx: &Sender<SerialTaskRequest>,
        msg: SerialMessage,
    ) -> io::Result<()> {
        let (tx, rx) = oneshot::channel();
        actor_tx
            .send(SerialTaskRequest::SendMessage { msg, response: tx })
            .await
            .map_err(|err| {
                tracing::error!("Failed to send message to serial task: {err}");
                io::ErrorKind::NotConnected
            })?;
        rx.await.map_err(|err| {
            tracing::error!("Failed to get response back for request: {err}");
            io::ErrorKind::UnexpectedEof
        })?
    }

    pub async fn wait_for_message(
        &self,
        matcher: Box<dyn Fn(&SerialMessage) -> bool + Send + Sync>,
        timeout: Option<Duration>,
    ) -> Option<SerialMessage> {
        let mut inbox_handle = self.inbox_handle.clone();
        inbox_handle.wait_for_message(matcher, timeout).await
    }

    pub fn check_for_message_since(
        &self,
        matcher: Box<dyn Fn(&SerialMessage) -> bool + Send + Sync>,
        start_time: Instant,
    ) -> Option<SerialMessage> {
        self.inbox_handle
            .check_for_message_since(matcher, start_time)
    }

    pub async fn set_led_state(&self, new_state: bool) -> io::Result<SetLedStateResponse> {
        self.send_message(SerialMessage::SetLedState(SetLedState { new_state }))
            .await?;
        let msg = self
            .wait_for_message(
                Box::new(|msg| matches!(msg, &SerialMessage::SetLedStateResponse(_))),
                None,
            )
            .await;
        match msg {
            Some(SerialMessage::SetLedStateResponse(response)) => Ok(response),
            Some(_) => panic!("Got unexpected message type: {msg:?}"),
            None => Err(io::ErrorKind::ConnectionAborted.into()),
        }
    }

    pub async fn set_rgb_state(&self, (r, g, b): (u8, u8, u8)) -> io::Result<SetRgbStateResponse> {
        self.send_message(SerialMessage::SetRgbState(SetRgbState { r, g, b }))
            .await?;
        let msg = self
            .wait_for_message(
                Box::new(|msg| matches!(msg, &SerialMessage::SetRgbStateResponse(_))),
                None,
            )
            .await;
        match msg {
            Some(SerialMessage::SetRgbStateResponse(response)) => Ok(response),
            Some(_) => panic!("Got unexpected message type: {msg:?}"),
            None => Err(io::ErrorKind::ConnectionAborted.into()),
        }
    }

    pub async fn update_row(
        &self,
        row_number: u8,
        row_data: Vec<bool>,
    ) -> io::Result<UpdateRowResponse> {
        let data = pack_bools_to_bytes(&row_data[..]);
        self.send_message(SerialMessage::UpdateRow(UpdateRow {
            row_number,
            row_data_len: row_data.len() as u8,
            row_data: data,
        }))
        .await?;
        let msg = self
            .wait_for_message(
                Box::new(|msg| matches!(msg, &SerialMessage::UpdateRowResponse(_))),
                None,
            )
            .await;
        match msg {
            Some(SerialMessage::UpdateRowResponse(response)) => Ok(response),
            Some(_) => panic!("Got unexpected message type: {msg:?}"),
            None => Err(io::ErrorKind::ConnectionAborted.into()),
        }
    }

    pub async fn update_row_rgb(
        &self,
        row_number: u8,
        row_data: Vec<u16>,
    ) -> io::Result<UpdateRowRgbResponse> {
        self.send_message(SerialMessage::UpdateRowRgb(UpdateRowRgb {
            row_number,
            row_data_len: row_data.len() as u8,
            row_data,
        }))
        .await?;
        let msg = self
            .wait_for_message(
                Box::new(|msg| matches!(msg, &SerialMessage::UpdateRowRgbResponse(_))),
                None,
            )
            .await;
        match msg {
            Some(SerialMessage::UpdateRowRgbResponse(response)) => Ok(response),
            Some(_) => panic!("Got unexpected message type: {msg:?}"),
            None => Err(io::ErrorKind::ConnectionAborted.into()),
        }
    }

    pub async fn get_display_info(&self) -> io::Result<GetDisplayInfoResponse> {
        self.send_message(SerialMessage::GetDisplayInfo(GetDisplayInfo))
            .await?;
        let msg = self
            .wait_for_message(
                Box::new(|msg| matches!(msg, &SerialMessage::GetDisplayInfoResponse(_))),
                None,
            )
            .await;
        match msg {
            Some(SerialMessage::GetDisplayInfoResponse(response)) => Ok(response),
            Some(_) => panic!("Got unexpected message type: {msg:?}"),
            None => Err(io::ErrorKind::ConnectionAborted.into()),
        }
    }

    pub async fn commit_render(&self) -> io::Result<CommitRenderResponse> {
        self.send_message(SerialMessage::RequestCommitRender(RequestCommitRender {}))
            .await?;
        let msg = self
            .wait_for_message(
                Box::new(|msg| matches!(msg, &SerialMessage::CommitRenderResponse(_))),
                None,
            )
            .await;
        match msg {
            Some(SerialMessage::CommitRenderResponse(response)) => Ok(response),
            Some(_) => unreachable!(),
            None => Err(io::ErrorKind::ConnectionAborted.into()),
        }
    }

    pub async fn set_single_cell(
        &self,
        row: u8,
        col: u8,
        value: bool,
    ) -> io::Result<SetSingleCellResponse> {
        self.send_message(SerialMessage::SetSingleCell(SetSingleCell {
            row,
            col,
            value,
        }))
        .await?;
        let msg = self
            .wait_for_message(
                Box::new(|msg| matches!(msg, &SerialMessage::SetSingleCellResponse(_))),
                None,
            )
            .await;
        match msg {
            Some(SerialMessage::SetSingleCellResponse(response)) => Ok(response),
            Some(_) => unreachable!(),
            None => Err(io::ErrorKind::ConnectionAborted.into()),
        }
    }

    pub async fn set_monocolor_palette(
        &self,
        color: u16,
    ) -> io::Result<SetMonocolorPaletteResponse> {
        self.send_message(SerialMessage::SetMonocolorPalette(SetMonocolorPalette {
            color,
        }))
        .await?;
        let msg = self
            .wait_for_message(
                Box::new(|msg| matches!(msg, &SerialMessage::SetMonocolorPaletteResponse(_))),
                None,
            )
            .await;
        match msg {
            Some(SerialMessage::SetMonocolorPaletteResponse(response)) => Ok(response),
            Some(_) => unreachable!(),
            None => Err(io::ErrorKind::ConnectionAborted.into()),
        }
    }
}

#[derive(Clone)]
pub struct SyncConnection {
    inner: Connection,
    rt: tokio::runtime::Handle,
}

impl SyncConnection {
    pub fn new(conn: Connection, rt: tokio::runtime::Handle) -> Self {
        Self { inner: conn, rt }
    }

    pub fn wait_for_message(
        &self,
        matcher: Box<dyn Fn(&SerialMessage) -> bool + Send + Sync>,
        timeout: Option<Duration>,
    ) -> Option<SerialMessage> {
        self.rt
            .block_on(async { self.inner.wait_for_message(matcher, timeout).await })
    }

    pub fn check_for_message_since(
        &self,
        matcher: Box<dyn Fn(&SerialMessage) -> bool + Send + Sync>,
        start_time: Instant,
    ) -> Option<SerialMessage> {
        self.inner.check_for_message_since(matcher, start_time)
    }

    pub fn set_led_state(&self, new_state: bool) -> io::Result<SetLedStateResponse> {
        self.rt.block_on(self.inner.set_led_state(new_state))
    }

    pub fn set_rgb_state(&self, (r, g, b): (u8, u8, u8)) -> io::Result<SetRgbStateResponse> {
        self.rt.block_on(self.inner.set_rgb_state((r, g, b)))
    }

    pub fn update_row(&self, row_number: u8, row_data: Vec<bool>) -> io::Result<UpdateRowResponse> {
        self.rt
            .block_on(self.inner.update_row(row_number, row_data))
    }

    pub fn update_row_rgb(
        &self,
        row_number: u8,
        row_data: Vec<u16>,
    ) -> io::Result<UpdateRowRgbResponse> {
        self.rt
            .block_on(self.inner.update_row_rgb(row_number, row_data))
    }

    pub fn get_display_info(&self) -> io::Result<GetDisplayInfoResponse> {
        self.rt.block_on(self.inner.get_display_info())
    }

    pub fn commit_render(&self) -> io::Result<CommitRenderResponse> {
        self.rt.block_on(self.inner.commit_render())
    }

    pub fn set_monocolor_palette(&self, color: u16) -> io::Result<SetMonocolorPaletteResponse> {
        self.rt.block_on(self.inner.set_monocolor_palette(color))
    }
}
