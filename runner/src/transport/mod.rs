use std::{net::SocketAddr, path::PathBuf, str::FromStr};
use tokio::io::{AsyncRead, AsyncWrite};
use tokio_serial::{SerialPortBuilderExt, SerialStream};

mod connection;
mod msg_inbox;
mod tasks;

pub use connection::{Connection, SyncConnection};
pub use tasks::start_transport_task;

trait AsyncIo: AsyncRead + AsyncWrite + Unpin + Send + Sync {}

impl AsyncIo for SerialStream {}
impl AsyncIo for tokio::net::TcpStream {}

#[derive(Debug, Clone)]
pub enum DeviceTransport {
    Serial(PathBuf),
    Tcp(SocketAddr),
}

impl From<String> for DeviceTransport {
    fn from(value: String) -> Self {
        if let Ok(addr) = SocketAddr::from_str(&value) {
            Self::Tcp(addr)
        } else {
            if let Ok(path) = PathBuf::from_str(&value) {
                Self::Serial(path)
            } else {
                panic!("Failed to parse DeviceTransport from {value}");
            }
        }
    }
}

async fn connect(info: DeviceTransport) -> Box<dyn AsyncIo> {
    match info {
        DeviceTransport::Serial(device_path) => {
            match tokio_serial::new(device_path.to_str().unwrap(), 230400).open_native_async() {
                Ok(serial) => {
                    tracing::info!("Opened serial port: {}", device_path.display());
                    Box::new(serial)
                }
                Err(err) => {
                    tracing::error!(
                        "Failed to open serial port {}: {err}",
                        device_path.display()
                    );
                    panic!("");
                }
            }
        }
        DeviceTransport::Tcp(addr) => match tokio::net::TcpStream::connect(addr).await {
            Ok(stream) => {
                tracing::info!("Opened tcp connection: {addr}");
                Box::new(stream)
            }
            Err(err) => {
                tracing::error!("Failed to open tcp stream to {addr}: {err}");
                panic!("");
            }
        },
    }
}
