use async_channel::{Receiver, Sender};
use std::io;
use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
    net::tcp::{OwnedReadHalf, OwnedWriteHalf},
};

pub async fn connect(
    runner_port: u16,
    from_ws_rx: Receiver<Vec<u8>>,
    to_ws_tx: Sender<Vec<u8>>,
) -> io::Result<()> {
    loop {
        let (stream_reader, stream_writer) =
            match tokio::net::TcpStream::connect(("127.0.0.1", runner_port)).await {
                Ok(stream) => stream.into_split(),
                Err(err) => {
                    if let io::ErrorKind::ConnectionRefused = err.kind() {
                        continue;
                    } else {
                        tracing::error!("Failed to connect to runner: {err:?}");
                        return Err(err);
                    }
                }
            };
        tokio::select! {
            _ = writer_task(from_ws_rx.clone(), stream_writer) => {
                tracing::debug!("Runner client writer exited");
            },
            _ = reader_task(to_ws_tx.clone(), stream_reader) => {
                tracing::debug!("Runner client reader exited");
            }
        }
    }
}

async fn writer_task(
    from_ws_rx: Receiver<Vec<u8>>,
    mut stream_writer: OwnedWriteHalf,
) -> io::Result<()> {
    loop {
        match from_ws_rx.recv().await {
            Ok(data) => stream_writer.write_all(&data[..]).await?,
            Err(err) => {
                tracing::error!("Failed to receive from websocket: {err:?}");
                break Err(io::ErrorKind::NotConnected.into());
            }
        }
    }
}

async fn reader_task(
    to_ws_tx: Sender<Vec<u8>>,
    mut stream_reader: OwnedReadHalf,
) -> io::Result<()> {
    let mut read_buf = [0u8; 1024];
    loop {
        let bytes_read = stream_reader.read(&mut read_buf[..]).await?;
        let packet = Vec::from(&read_buf[..bytes_read]);
        if let Err(err) = to_ws_tx.send(packet).await {
            tracing::error!("Failed to send to websocket: {err:?}");
            break Err(io::ErrorKind::NotConnected.into());
        }
    }
}
