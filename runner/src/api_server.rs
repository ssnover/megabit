use async_channel::{Receiver, Sender, TryRecvError};
use megabit_runner_msgs::ConsoleMessage;
use std::{
    io::{self},
    net::SocketAddr,
    sync::Arc,
};
use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
    net::{
        tcp::{OwnedReadHalf, OwnedWriteHalf},
        TcpListener, TcpStream,
    },
    task::JoinHandle,
};

#[derive(Clone, Debug)]
pub struct ApiServerHandle {
    tx: Sender<ConsoleMessage>,
    rx: Receiver<ConsoleMessage>,
    _handle: Arc<JoinHandle<()>>,
}

impl ApiServerHandle {
    pub fn get_next(&self) -> anyhow::Result<Option<ConsoleMessage>> {
        match self.rx.try_recv() {
            Ok(msg) => Ok(Some(msg)),
            Err(TryRecvError::Empty) => Ok(None),
            Err(TryRecvError::Closed) => Err(anyhow::anyhow!("API server sender closed")),
        }
    }

    pub async fn next(&self) -> anyhow::Result<ConsoleMessage> {
        Ok(self.rx.recv().await?)
    }

    pub fn send_blocking(&self, msg: ConsoleMessage) -> anyhow::Result<()> {
        Ok(self.tx.send_blocking(msg)?)
    }

    pub async fn send(&self, msg: ConsoleMessage) -> anyhow::Result<()> {
        Ok(self.tx.send(msg).await?)
    }
}

pub fn start(api_port: u16, rt: tokio::runtime::Handle) -> ApiServerHandle {
    tracing::info!("Starting API server");
    let (server_tx, server_rx) = async_channel::unbounded();
    let (tx, rx) = async_channel::bounded(100);
    let handle = rt.spawn(listen_for_api_commands(api_port, rt.clone(), tx, server_rx));

    ApiServerHandle {
        tx: server_tx,
        rx,
        _handle: Arc::new(handle),
    }
}

async fn listen_for_api_commands(
    port: u16,
    rt: tokio::runtime::Handle,
    tx: Sender<ConsoleMessage>,
    rx: Receiver<ConsoleMessage>,
) {
    tracing::debug!("Starting listener context");
    if let Err(err) = listen_context(port, rt, tx, rx).await {
        tracing::error!("API server shutting down: {err:?}");
    }
}

async fn listen_context(
    port: u16,
    rt: tokio::runtime::Handle,
    tx: Sender<ConsoleMessage>,
    rx: Receiver<ConsoleMessage>,
) -> io::Result<()> {
    let addr = format!("0.0.0.0:{port}");
    let listener = TcpListener::bind(addr).await?;
    tracing::info!(
        "Binding API server to listen at {}",
        listener.local_addr().unwrap()
    );

    tracing::debug!("Waiting for connections");
    loop {
        let (stream, peer) = listener.accept().await?;
        tracing::info!("Received connection from client at {peer}");
        // TODO: async_channel::Receiver consume messages, they don't each get a copy, this prevents multiple connections from stably working
        // Perhaps this could be solved with an additional task which takes ownership of the receiver and broadcasts to all of another type
        // of spmc channel
        rt.spawn(connection_context(peer, stream, tx.clone(), rx.clone()));
    }
}

async fn connection_context(
    peer: SocketAddr,
    stream: TcpStream,
    tx: Sender<ConsoleMessage>,
    rx: Receiver<ConsoleMessage>,
) {
    tracing::debug!("Creating connection to peer {peer}");

    let (reader, writer) = stream.into_split();
    tokio::join!(
        connection_listen_context(peer, reader, tx),
        connection_report_context(peer, writer, rx)
    );
}

async fn connection_listen_context(
    peer: SocketAddr,
    mut reader: OwnedReadHalf,
    tx: Sender<ConsoleMessage>,
) {
    let mut buffer_fill = 0;
    let mut byte_buffer = Vec::with_capacity(1024 * 16);
    loop {
        match reader.read_buf(&mut byte_buffer).await {
            Ok(bytes_read) => {
                buffer_fill += bytes_read;
            }
            Err(err) => {
                tracing::error!("Failed to read bytes from connection, disconnecting: {err:?}");
                break;
            }
        }
        tracing::trace!("Got some data ({buffer_fill} bytes)");

        if let Some(whole_json_length) = check_for_json(&byte_buffer[..]) {
            tracing::trace!("Got a JSON message");
            let msg = &byte_buffer[..whole_json_length];
            let msg = match serde_json::from_slice(msg) {
                Ok(msg) => Some(msg),
                Err(err) => {
                    tracing::error!(
                        "Failed to parse JSON packet {}: {err:?}",
                        std::str::from_utf8(msg).expect("Non-UTF8 encoded data")
                    );
                    None
                }
            };

            byte_buffer = Vec::from_iter(byte_buffer.into_iter().skip(whole_json_length));

            match msg {
                Some(msg) => {
                    tracing::debug!("Received console message: {msg:?}");
                    match tx.send(msg).await {
                        Ok(()) => {}
                        Err(err) => {
                            tracing::error!(
                        "Connection unable to forward parse console message from peer {peer}: {err:?}"
                    );
                            break;
                        }
                    }
                }
                None => {
                    continue;
                }
            }
        } else {
            tracing::debug!("Could not parse a JSON message");
        }
    }

    tracing::info!("Exiting connection");
}

async fn connection_report_context(
    peer: SocketAddr,
    mut writer: OwnedWriteHalf,
    rx: Receiver<ConsoleMessage>,
) {
    loop {
        match rx.recv().await {
            Ok(console_msg) => {
                tracing::debug!("Sending message {console_msg:?} to peer {peer}");
                let msg =
                    serde_json::to_vec(&console_msg).expect("Able to serialize console message");
                let mut attempts = 0;
                while attempts < 3 {
                    if let Err(err) = writer.write_all(&msg).await {
                        attempts += 1;
                        tracing::error!(
                            "Failed to send console message to peer {peer}: {err:?} ({attempts}/3)"
                        );
                    } else {
                        break;
                    }
                }
            }
            Err(_err) => {
                tracing::error!(
                    "The console message report stream has closed, exiting report context"
                );
            }
        }
    }
}

fn check_for_json(data: &[u8]) -> Option<usize> {
    if data.len() < 2 {
        None
    } else if data[0] != b'{' {
        None
    } else {
        let mut counted_brackets = 0;
        let mut data_consumed = 0;
        for byte in data {
            data_consumed += 1;
            if *byte == b'{' {
                counted_brackets += 1;
            } else if *byte == b'}' {
                counted_brackets -= 1;
            } else {
                continue;
            }
            if counted_brackets < 0 {
                return None;
            } else if counted_brackets == 0 {
                break;
            }
        }
        if counted_brackets == 0 {
            Some(data_consumed)
        } else {
            None
        }
    }
}
