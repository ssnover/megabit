use async_channel::{Receiver, Sender};
use std::{io, time::Duration};
use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
    net::{
        tcp::{ReadHalf, WriteHalf},
        TcpListener,
    },
};

pub struct SerialListener {
    listener: TcpListener,
}

impl SerialListener {
    pub async fn new(port: u16) -> io::Result<Self> {
        let socket_addr = format!("0.0.0.0:{port}");
        let listener = TcpListener::bind(socket_addr).await?;
        Ok(Self { listener })
    }

    pub async fn listen(
        self,
        to_simulator: Sender<Vec<u8>>,
        from_simulator: Receiver<Vec<u8>>,
    ) -> io::Result<()> {
        loop {
            let to_sim = to_simulator.clone();
            let from_sim = from_simulator.clone();

            let (mut stream, _peer) = self.listener.accept().await?;
            tokio::spawn(async move {
                if let Err(err) = stream.set_nodelay(true) {
                    tracing::warn!("Failed to sett tcp nodelay: {err:?}");
                }
                let (reader, writer) = stream.split();

                tokio::select! {
                    _ = Self::handle_incoming_serial_bytes(reader, to_sim) => {
                        tracing::info!("Exiting transport reader");
                    },
                    res = Self::handle_simulator_packet(writer, from_sim) => {
                        match res {
                            Ok(()) => tracing::info!("Exiting transport writer"),
                            Err(err) => tracing::error!("Transport writer closed due to error: {err:?}"),
                        }
                    }
                }
            });
        }
    }

    async fn handle_incoming_serial_bytes(mut reader: ReadHalf<'_>, to_simulator: Sender<Vec<u8>>) {
        let mut incoming_buffer = Vec::with_capacity(4096);
        loop {
            if incoming_buffer.len() >= 3 {
                if incoming_buffer.len() < 10 {
                    tracing::debug!("Got data: {:?}", &incoming_buffer[..]);
                }
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
            } else {
                if let Err(err) = reader.read_buf(&mut incoming_buffer).await {
                    tracing::error!("Error reading from the serial port: {err}");
                    break;
                }
            }
        }

        tracing::info!("Exiting handler for incoming serial data");
    }

    async fn handle_simulator_packet(
        mut writer: WriteHalf<'_>,
        from_simulator: Receiver<Vec<u8>>,
    ) -> io::Result<()> {
        while let Ok(msg) = from_simulator.recv().await {
            let mut encoded_data = cobs::encode_vec(&msg[..]);
            encoded_data.push(0x00);
            tracing::debug!("Sending serial data: {encoded_data:02x?}");
            send_data_stubbornly(&mut writer, &encoded_data[..]).await?
        }
        Ok(())
    }
}

async fn send_data_stubbornly(writer: &mut WriteHalf<'_>, data: &[u8]) -> io::Result<()> {
    let mut attempts = 0u32;
    loop {
        attempts += 1;
        match tokio::time::timeout(Duration::from_secs(5), async {
            if let Err(err) = writer.write_all(data).await {
                tracing::error!("Failed to write encoded serial packet: {err}");
                return Err(err);
            }
            writer.flush().await?;
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
            Ok(Err(err)) => {
                tracing::debug!("Try again!");
                return Err(err);
            }
        }
    }

    Ok(())
}

pub async fn run(
    port: u16,
    to_simulator: Sender<Vec<u8>>,
    from_simulator: Receiver<Vec<u8>>,
) -> io::Result<()> {
    let virtual_device = SerialListener::new(port).await.map_err(|err| {
        tracing::error!("Unable to create serial listener at port {port}: {err}");
        err
    })?;

    virtual_device.listen(to_simulator, from_simulator).await
}
