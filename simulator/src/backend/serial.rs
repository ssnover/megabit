use async_channel::{Receiver, Sender};
use nix::pty::{self, OpenptyResult};
use std::{
    io,
    os::fd::{AsRawFd, FromRawFd, IntoRawFd},
    path::{Path, PathBuf},
    time::Duration,
};
use tokio::{
    fs::File,
    io::{AsyncReadExt, AsyncWriteExt, ReadHalf, WriteHalf},
};

pub struct VirtualSerial {
    host_file: File,
    _host_path: PathBuf,
    _device_file: File,
    device_path: PathBuf,
}

impl VirtualSerial {
    pub fn new(symlink_path: impl AsRef<Path>) -> io::Result<Self> {
        let OpenptyResult { master, slave } = pty::openpty(None, None).expect("Failed to open pty");
        let serial = Self {
            _host_path: nix::unistd::ttyname(master.as_raw_fd()).expect("Valid fd for pty"),
            host_file: unsafe { File::from_raw_fd(master.into_raw_fd()) },
            device_path: nix::unistd::ttyname(slave.as_raw_fd()).expect("Valid fd for pty"),
            _device_file: unsafe { File::from_raw_fd(slave.into_raw_fd()) },
        };
        tracing::debug!("Host path: {}", serial._host_path.display());
        if symlink_path.as_ref().exists() {
            std::fs::remove_file(symlink_path.as_ref())?;
        }
        tracing::info!(
            "Creating symlink to {} at {}",
            serial.device_path.display(),
            symlink_path.as_ref().display()
        );
        std::os::unix::fs::symlink(&serial.device_path, symlink_path)?;
        Ok(serial)
    }

    pub async fn listen(self, to_simulator: Sender<Vec<u8>>, from_simulator: Receiver<Vec<u8>>) {
        let (reader, writer) = tokio::io::split(self.host_file);

        tokio::join!(
            Self::handle_incoming_serial_bytes(reader, to_simulator),
            Self::handle_simulator_packet(writer, from_simulator)
        );

        tracing::info!("Exiting serial device listening context");
    }

    async fn handle_incoming_serial_bytes(
        mut reader: ReadHalf<File>,
        to_simulator: Sender<Vec<u8>>,
    ) {
        // Alright, this function is a bit of a doozy. The chief problem is that tokio Files lock
        // in order to read or write because Linux has no means of polling them for new data.
        // This means we must time out the read periodically or else it will grab the lock and
        // prevent the other task from writing data. Additionally, I've added a small delay at the
        // end of the loop to give the write context a little bit of extra time to grab and write.
        // When the write task gets starved, weird things happen. In particular, despite it being
        // a pseudoterminal, I've observed written data being read out (as if it was echoed from
        // the other side). My goal here is principally to prevent this task from starving the
        // other one.
        let mut incoming_buffer = Vec::with_capacity(2048);
        loop {
            if let Ok(Err(err)) = tokio::time::timeout(
                Duration::from_millis(5),
                reader.read_buf(&mut incoming_buffer),
            )
            .await
            {
                tracing::error!("Error reading from the pty: {err}");
            }

            if incoming_buffer.len() >= 3 {
                tracing::trace!("Got data");
                if let Ok(decoded_data) = cobs::decode_vec(&incoming_buffer[..]) {
                    tracing::debug!(
                        "Decoded a payload of {} bytes from buffer of {} bytes",
                        decoded_data.len(),
                        incoming_buffer.len()
                    );
                    if let Err(err) = to_simulator.send(decoded_data).await {
                        tracing::error!("Failed to send serial payload: {err}");
                        break;
                    }
                    let (encoded_len, _) = incoming_buffer
                        .iter()
                        .enumerate()
                        .find(|(_idx, elem)| **elem == 0x00)
                        .expect("Need a terminator to have a valid COBS encoded payload");
                    incoming_buffer =
                        Vec::from_iter(incoming_buffer.into_iter().skip(encoded_len + 1));
                }
            }

            tokio::time::sleep(Duration::from_millis(1)).await;
        }

        tracing::info!("Exiting handler for incoming serial data");
    }

    async fn handle_simulator_packet(
        mut writer: WriteHalf<File>,
        from_simulator: Receiver<Vec<u8>>,
    ) {
        while let Ok(msg) = from_simulator.recv().await {
            let mut encoded_data = cobs::encode_vec(&msg[..]);
            encoded_data.push(0x00);
            tracing::debug!("Sending serial data: {encoded_data:02x?}");
            send_data_stubbornly(&mut writer, &encoded_data[..]).await
        }
    }
}

async fn send_data_stubbornly(writer: &mut WriteHalf<File>, data: &[u8]) {
    let mut attempts = 0u32;
    loop {
        attempts += 1;
        match tokio::time::timeout(Duration::from_secs(5), async {
            if let Err(err) = writer.write_all(data).await {
                tracing::error!("Failed to write encoded serial packet: {err}");
                return Err(err);
            }
            Ok(())
        })
        .await
        {
            Err(_) => tracing::debug!("Writing timed out"),
            Ok(Ok(())) => {
                if attempts > 1 {
                    tracing::debug!("Serial send complete after {attempts} attempts");
                } else {
                    tracing::debug!("Serial send complete");
                }
                break;
            }
            Ok(Err(_)) => {
                tracing::debug!("Try again!");
            }
        }
    }
}

pub async fn run(
    device_path: impl AsRef<Path>,
    to_simulator: Sender<Vec<u8>>,
    from_simulator: Receiver<Vec<u8>>,
) {
    let virtual_device = match VirtualSerial::new(device_path.as_ref()) {
        Ok(device) => device,
        Err(err) => {
            tracing::error!(
                "Unable to create virtual serial device {}: {err}",
                device_path.as_ref().display()
            );
            let _ = std::fs::remove_file(device_path.as_ref());
            return;
        }
    };

    virtual_device.listen(to_simulator, from_simulator).await
}
