use super::{connection::Connection, msg_inbox::MessageInbox, DeviceTransport};
use async_channel::{Receiver, Sender};
use megabit_serial_protocol::*;
use std::{future::Future, io, time::Duration};
use tokio::{
    io::{AsyncRead, AsyncReadExt, AsyncWrite, AsyncWriteExt},
    sync::oneshot,
};

#[derive(Debug)]
pub enum SerialTaskRequest {
    SendMessage {
        msg: SerialMessage,
        response: oneshot::Sender<io::Result<()>>,
    },
}

pub fn start_transport_task(
    transport_info: DeviceTransport,
) -> (Connection, Box<dyn Future<Output = ()> + Send + Sync>) {
    let (msg_tx, msg_rx) = async_channel::unbounded();
    let (tx, rx) = async_channel::unbounded();

    let serial_future = transport_task(transport_info, rx, msg_tx);
    let _ping_task = {
        let tx = tx.clone();
        async move {
            loop {
                tokio::time::sleep(Duration::from_millis(333)).await;
                if let Err(err) = Connection::send_message_inner(&tx, SerialMessage::Ping).await {
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
        Connection {
            actor_tx: tx,
            inbox_handle,
        },
        Box::new(serial_task),
    )
}

async fn transport_task(
    info: DeviceTransport,
    request_rx: Receiver<SerialTaskRequest>,
    incoming_msg_tx: Sender<SerialMessage>,
) {
    tracing::info!("Starting serial task");
    let transport = super::connect(info).await;

    let (transport_rx, transport_tx) = tokio::io::split(transport);

    tokio::select! {
        res = handle_requests(transport_tx, request_rx) => {
            if let Err(err) = res {
                tracing::error!("Serial task request handling exited with error: {err}");
            } else {
                tracing::info!("Serial task request handling exited");
            }
        },
        res = handle_serial_msgs(transport_rx, incoming_msg_tx) => {
            if let Err(err) = res {
                tracing::error!("Serial task serial message handling exited with error: {err}");
            } else {
                tracing::info!("Serial task serial message handling exited");
            }
        },
    };
}

async fn handle_requests(
    mut serial_tx: impl AsyncWrite + Unpin,
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
    mut serial_rx: impl AsyncRead + Unpin,
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
