use async_channel::{Receiver, Sender};
use megabit_serial_protocol::*;
use std::{
    future::Future,
    io,
    path::{Path, PathBuf},
    time::{Duration, Instant},
};
use tokio::{
    io::{AsyncReadExt, AsyncWriteExt, ReadHalf, WriteHalf},
    sync::oneshot,
};
use tokio_serial::{SerialPortBuilderExt, SerialStream};

use self::msg_inbox::{InboxHandle, MessageInbox};

mod msg_inbox;

#[derive(Debug)]
enum SerialTaskRequest {
    SendMessage {
        msg: SerialMessage,
        response: oneshot::Sender<io::Result<()>>,
    },
}

pub fn start_serial_task(
    device_path: impl AsRef<Path>,
) -> (SerialConnection, Box<dyn Future<Output = ()> + Send + Sync>) {
    let (msg_tx, msg_rx) = async_channel::unbounded();
    let (tx, rx) = async_channel::unbounded();
    let device_path = device_path.as_ref().to_path_buf();

    let serial_future = serial_task(device_path, rx, msg_tx);
    let _ping_task = {
        let tx = tx.clone();
        async move {
            loop {
                tokio::time::sleep(Duration::from_millis(333)).await;
                if let Err(err) =
                    SerialConnection::send_message_inner(&tx, SerialMessage::Ping).await
                {
                    tracing::error!("Failed to send ping to device: {err}");
                    break;
                }
            }
        }
    };

    let message_inbox = MessageInbox::new(msg_rx.clone(), Some(Duration::from_secs(5)));
    let inbox_handle = message_inbox.get_handle();
    let message_inbox_task = message_inbox.run();

    let serial_task = async move {
        tokio::join!(serial_future, message_inbox_task);
    };

    (
        SerialConnection {
            actor_tx: tx,
            inbox_handle,
        },
        Box::new(serial_task),
    )
}

#[derive(Clone)]
pub struct SerialConnection {
    actor_tx: Sender<SerialTaskRequest>,
    inbox_handle: InboxHandle,
}

impl SerialConnection {
    async fn send_message(&self, msg: SerialMessage) -> io::Result<()> {
        Self::send_message_inner(&self.actor_tx, msg).await
    }

    async fn send_message_inner(
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
        matcher: Box<dyn Fn(&SerialMessage) -> bool>,
        timeout: Option<Duration>,
    ) -> Option<SerialMessage> {
        self.inbox_handle.wait_for_message(matcher, timeout).await
    }

    pub fn check_for_message_since(
        &self,
        matcher: Box<dyn Fn(&SerialMessage) -> bool>,
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
pub struct SyncSerialConnection {
    inner: SerialConnection,
    rt: tokio::runtime::Handle,
}

impl SyncSerialConnection {
    pub fn new(conn: SerialConnection, rt: tokio::runtime::Handle) -> Self {
        Self { inner: conn, rt }
    }

    pub fn wait_for_message(
        &self,
        matcher: Box<dyn Fn(&SerialMessage) -> bool>,
        timeout: Option<Duration>,
    ) -> Option<SerialMessage> {
        self.rt
            .block_on(async { self.inner.wait_for_message(matcher, timeout).await })
    }

    pub fn check_for_message_since(
        &self,
        matcher: Box<dyn Fn(&SerialMessage) -> bool>,
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

async fn serial_task(
    device_path: PathBuf,
    request_rx: Receiver<SerialTaskRequest>,
    incoming_msg_tx: Sender<SerialMessage>,
) {
    tracing::info!("Starting serial task");
    let serial_port =
        match tokio_serial::new(device_path.to_str().unwrap(), 230400).open_native_async() {
            Ok(serial) => serial,
            Err(err) => {
                tracing::error!(
                    "Failed to open serial port {}: {err}",
                    device_path.display()
                );
                return;
            }
        };
    tracing::info!("Opened serial port: {}", device_path.display());
    let (serial_rx, serial_tx) = tokio::io::split(serial_port);

    tokio::select! {
        res = handle_requests(serial_tx, request_rx) => {
            if let Err(err) = res {
                tracing::error!("Serial task request handling exited with error: {err}");
            } else {
                tracing::info!("Serial task request handling exited");
            }
        },
        res = handle_serial_msgs(serial_rx, incoming_msg_tx) => {
            if let Err(err) = res {
                tracing::error!("Serial task serial message handling exited with error: {err}");
            } else {
                tracing::info!("Serial task serial message handling exited");
            }
        },
    };
}

async fn handle_requests(
    mut serial_tx: WriteHalf<SerialStream>,
    request_rx: Receiver<SerialTaskRequest>,
) -> anyhow::Result<()> {
    while let Ok(msg) = request_rx.recv().await {
        match msg {
            SerialTaskRequest::SendMessage { msg, response } => {
                tracing::trace!("Send message: {}", msg.as_ref());
                let payload = msg.to_bytes();
                let mut payload = cobs::encode_vec(&payload[..]);
                payload.push(0x00);
                let _ = response.send(serial_tx.write_all(&payload[..]).await);
            }
        }
    }

    Ok(())
}

async fn handle_serial_msgs(
    mut serial_rx: ReadHalf<SerialStream>,
    incoming_msg_tx: Sender<SerialMessage>,
) -> anyhow::Result<()> {
    let mut incoming_serial_buffer = Vec::with_capacity(1024);
    loop {
        if incoming_serial_buffer.len() >= 3 {
            if let Ok(decoded_data) = cobs::decode_vec(&incoming_serial_buffer[..]) {
                tracing::trace!(
                    "Decoded a payload of {} bytes from buffer of {} bytes",
                    decoded_data.len(),
                    incoming_serial_buffer.len()
                );
                if let Ok(msg) = SerialMessage::try_from_bytes(&decoded_data[..]) {
                    tracing::trace!("Decoded a message: {msg:?}");
                    if let Err(err) = incoming_msg_tx.send(msg).await {
                        tracing::error!("Failed to forward deserialized device message: {err}");
                        return Err(err.into());
                    }
                }
                let (encoded_len, _) = incoming_serial_buffer
                    .iter()
                    .enumerate()
                    .find(|(_idx, elem)| **elem == 0x00)
                    .expect("Need a terminator to have a valid COBS payload");
                incoming_serial_buffer =
                    Vec::from_iter(incoming_serial_buffer.into_iter().skip(encoded_len + 1));
            }
        } else {
            match serial_rx.read_buf(&mut incoming_serial_buffer).await {
                Ok(n) => {
                    tracing::trace!("Received {n} bytes from the serial port");
                }
                Err(err) => {
                    tracing::error!("Failed to read data from the serial port: {err}");
                    return Err(err.into());
                }
            }
        }
    }
}
