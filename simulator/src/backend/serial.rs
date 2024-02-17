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
        let (mut stream, _peer) = self.listener.accept().await?;
        stream.set_nodelay(true)?;
        let (reader, writer) = stream.split();

        tokio::join!(
            Self::handle_incoming_serial_bytes(reader, to_simulator),
            Self::handle_simulator_packet(writer, from_simulator)
        );

        tracing::info!("Exiting serial device listening context");
        Ok(())
    }

    async fn handle_incoming_serial_bytes(mut reader: ReadHalf<'_>, to_simulator: Sender<Vec<u8>>) {
        // Alright, this function is a bit of a doozy. The chief problem is that tokio Files lock
        // in order to read or write because Linux has no means of polling them for new data.
        // This means we must time out the read periodically or else it will grab the lock and
        // prevent the other task from writing data. Additionally, I've added a small delay at the
        // end of the loop to give the write context a little bit of extra time to grab and write.
        // When the write task gets starved, weird things happen. In particular, despite it being
        // a pseudoterminal, I've observed written data being read out (as if it was echoed from
        // the other side). My goal here is principally to prevent this task from starving the
        // other one.
        let mut incoming_buffer = Vec::with_capacity(4096);
        loop {
            if let Err(err) = reader.read_buf(&mut incoming_buffer).await {
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
        }

        tracing::info!("Exiting handler for incoming serial data");
    }

    async fn handle_simulator_packet(mut writer: WriteHalf<'_>, from_simulator: Receiver<Vec<u8>>) {
        while let Ok(msg) = from_simulator.recv().await {
            let mut encoded_data = cobs::encode_vec(&msg[..]);
            encoded_data.push(0x00);
            tracing::debug!("Sending serial data: {encoded_data:02x?}");
            send_data_stubbornly(&mut writer, &encoded_data[..]).await
        }
    }
}

async fn send_data_stubbornly(writer: &mut WriteHalf<'_>, data: &[u8]) {
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
            Ok(Err(_)) => {
                tracing::debug!("Try again!");
            }
        }
    }
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
