use abort_on_drop::ChildTask;
use async_channel::{Receiver, Sender};
use megabit_serial_protocol::*;
use std::{
    io,
    path::{Path, PathBuf},
    time::Duration,
};
use tokio::io::{AsyncReadExt, AsyncWriteExt, ReadHalf, WriteHalf};
use tokio::sync::oneshot;
use tokio_serial::{SerialPortBuilderExt, SerialStream};

#[derive(Debug)]
enum SerialTaskRequest {
    SendMessage {
        msg: SerialMessage,
        response: oneshot::Sender<io::Result<()>>,
    },
}

pub struct SerialConnection {
    actor_tx: Sender<SerialTaskRequest>,
    _serial_task: ChildTask<()>,
    _ping_task: ChildTask<()>,
}

impl SerialConnection {
    pub fn new(
        device_path: impl AsRef<Path>,
        incoming_msg_tx: Sender<SerialMessage>,
    ) -> SerialConnection {
        let (tx, rx) = async_channel::unbounded();
        let device_path = device_path.as_ref().to_path_buf();

        let handle = tokio::spawn(serial_task(device_path, rx, incoming_msg_tx));
        let ping_task = {
            let tx = tx.clone();
            tokio::spawn(async move {
                loop {
                    tokio::time::sleep(Duration::from_millis(333)).await;
                    if let Err(err) = Self::send_message_inner(&tx, SerialMessage::Ping).await {
                        tracing::error!("Failed to send ping to device: {err}");
                        break;
                    }
                }
            })
        };

        SerialConnection {
            actor_tx: tx,
            _serial_task: handle.into(),
            _ping_task: ping_task.into(),
        }
    }

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

    pub async fn set_led_state(&self, new_state: bool) -> io::Result<()> {
        self.send_message(SerialMessage::SetLedState(SetLedState { new_state }))
            .await
    }

    pub async fn set_rgb_state(&self, (r, g, b): (u8, u8, u8)) -> io::Result<()> {
        self.send_message(SerialMessage::SetRgbState(SetRgbState { r, g, b }))
            .await
    }

    pub async fn update_row(&self, row_number: u8, row_data: Vec<bool>) -> io::Result<()> {
        let data = pack_bools_to_bytes(&row_data[..]);
        self.send_message(SerialMessage::UpdateRow(UpdateRow {
            row_number,
            row_data_len: row_data.len() as u8,
            row_data: data,
        }))
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
                tracing::trace!("Received {n} bytes from the serial port");
                if let Ok(decoded_data) = cobs::decode_vec(&incoming_serial_buffer[..]) {
                    tracing::trace!(
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
