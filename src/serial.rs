use abort_on_drop::ChildTask;
use async_channel::{Receiver, Sender};
use std::{
    io,
    path::{Path, PathBuf},
};
use tokio::io::{AsyncReadExt, AsyncWriteExt, ReadHalf, WriteHalf};
use tokio::sync::oneshot;
use tokio_serial::{SerialPortBuilderExt, SerialStream};

#[derive(Clone, Debug)]
pub enum SerialMessage {
    SetLedState(SetLedState),
    SetLedStateResponse(SetLedStateResponse),
    SetRgbState(SetRgbState),
    SetRgbStateResponse(SetRgbStateResponse),
    ReportButtonPress,
}

impl SerialMessage {
    pub fn to_bytes(self) -> Vec<u8> {
        let mut out = vec![];
        match self {
            SerialMessage::SetLedState(inner) => {
                out.push(0xde);
                out.push(0x00);
                out.append(&mut inner.to_bytes())
            }
            SerialMessage::SetLedStateResponse(inner) => {
                out.push(0xde);
                out.push(0x01);
                out.append(&mut inner.to_bytes())
            }
            SerialMessage::SetRgbState(inner) => {
                out.push(0xde);
                out.push(0x02);
                out.append(&mut inner.to_bytes())
            }
            SerialMessage::SetRgbStateResponse(inner) => {
                out.push(0xde);
                out.push(0x03);
                out.append(&mut inner.to_bytes())
            }
            SerialMessage::ReportButtonPress => {
                out.push(0xde);
                out.push(0x04);
            }
        }

        out
    }

    pub fn try_from_bytes(data: &[u8]) -> io::Result<Self> {
        if data.len() >= 2 {
            match (data[0], data[1]) {
                (0xde, 0x01) => Ok(SerialMessage::SetLedStateResponse(
                    SetLedStateResponse::try_from_bytes(&data[2..])?,
                )),
                (0xde, 0x03) => Ok(SerialMessage::SetRgbStateResponse(
                    SetRgbStateResponse::try_from_bytes(&data[2..])?,
                )),
                (0xde, 0x04) => Ok(SerialMessage::ReportButtonPress),
                _ => {
                    tracing::error!(
                        "Unexpected serial message kind 0x{:02x}{:02x}",
                        data[0],
                        data[1]
                    );
                    Err(io::ErrorKind::InvalidData.into())
                }
            }
        } else {
            Err(io::ErrorKind::InvalidData.into())
        }
    }
}

#[derive(Clone, Debug)]
#[repr(u8)]
pub enum Status {
    Success = 0,
    Failure = 1,
    InProgress = 2,
}

impl TryFrom<u8> for Status {
    type Error = io::Error;
    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value {
            0 => Ok(Status::Success),
            1 => Ok(Status::Failure),
            2 => Ok(Status::InProgress),
            _ => Err(io::ErrorKind::InvalidData.into()),
        }
    }
}

impl From<Status> for u8 {
    fn from(value: Status) -> Self {
        match value {
            Status::Success => 0,
            Status::Failure => 1,
            Status::InProgress => 2,
        }
    }
}

#[derive(Clone, Debug)]
pub struct SetLedState {
    new_state: bool,
}

impl SetLedState {
    pub fn to_bytes(self) -> Vec<u8> {
        vec![if self.new_state { 0x01 } else { 0x00 }]
    }
}

#[derive(Clone, Debug)]
pub struct SetLedStateResponse {
    status: Status,
}

impl SetLedStateResponse {
    fn to_bytes(self) -> Vec<u8> {
        vec![self.status.into()]
    }

    fn try_from_bytes(data: &[u8]) -> io::Result<Self> {
        if data.len() == 1 {
            Ok(Self {
                status: Status::try_from(data[0])?,
            })
        } else {
            Err(io::ErrorKind::InvalidData.into())
        }
    }
}

#[derive(Clone, Debug)]
pub struct SetRgbState {
    r: u8,
    g: u8,
    b: u8,
}

impl SetRgbState {
    pub fn to_bytes(self) -> Vec<u8> {
        vec![self.r, self.g, self.b]
    }
}

#[derive(Clone, Debug)]
pub struct SetRgbStateResponse {
    status: Status,
}

impl SetRgbStateResponse {
    fn to_bytes(self) -> Vec<u8> {
        vec![self.status.into()]
    }

    fn try_from_bytes(data: &[u8]) -> io::Result<Self> {
        if data.len() == 1 {
            Ok(Self {
                status: Status::try_from(data[0])?,
            })
        } else {
            Err(io::ErrorKind::InvalidData.into())
        }
    }
}

#[derive(Debug)]
enum SerialTaskRequest {
    SendMessage {
        msg: SerialMessage,
        response: oneshot::Sender<io::Result<()>>,
    },
}

pub struct SerialConnection {
    actor_tx: Sender<SerialTaskRequest>,
    _task_handle: ChildTask<()>,
}

impl SerialConnection {
    pub fn new(
        device_path: impl AsRef<Path>,
        incoming_msg_tx: Sender<SerialMessage>,
    ) -> SerialConnection {
        let (tx, rx) = async_channel::unbounded();
        let device_path = device_path.as_ref().to_path_buf();

        let handle = tokio::spawn(serial_task(device_path, rx, incoming_msg_tx));

        SerialConnection {
            actor_tx: tx,
            _task_handle: handle.into(),
        }
    }

    async fn send_message(&self, msg: SerialMessage) -> io::Result<()> {
        let (tx, rx) = oneshot::channel();
        self.actor_tx
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

    pub async fn set_led_state(&self, new_state: bool) -> io::Result<()> {
        self.send_message(SerialMessage::SetLedState(SetLedState { new_state }))
            .await
    }

    pub async fn set_rgb_state(&self, (r, g, b): (u8, u8, u8)) -> io::Result<()> {
        self.send_message(SerialMessage::SetRgbState(SetRgbState { r, g, b }))
            .await
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
        match serial_rx.read_buf(&mut incoming_serial_buffer).await {
            Ok(n) => {
                tracing::debug!("Received {n} bytes from the serial port");
                if let Ok(decoded_data) = cobs::decode_vec(&incoming_serial_buffer[..]) {
                    tracing::debug!(
                        "Decoded a payload of {} bytes from buffer of {} bytes",
                        decoded_data.len(),
                        incoming_serial_buffer.len()
                    );
                    if let Ok(msg) = SerialMessage::try_from_bytes(&decoded_data[..]) {
                        tracing::debug!("Decoded a message: {msg:?}");
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
            }
            Err(err) => {
                tracing::error!("Failed to read data from the serial port: {err}");
                return Err(err.into());
            }
        }
    }
}
