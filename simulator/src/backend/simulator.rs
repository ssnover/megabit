use crate::messages::{SetDebugLed, SetMatrixRow, SetRgbLed, SimMessage};
use async_channel::{Receiver, Sender};
use megabit_serial_protocol::*;

pub async fn run(
    from_ws: Receiver<String>,
    from_serial: Receiver<Vec<u8>>,
    to_ws: Sender<String>,
    to_serial: Sender<Vec<u8>>,
) {
    tokio::select! {
        _ = handle_serial_packets(from_serial, to_ws, to_serial.clone()) => {
            tracing::info!("Serial handler exited");
        },
        _ = handle_ws_message(from_ws, to_serial) => {
            tracing::info!("Websocket message handler exited");
        }
    }
}

async fn handle_serial_packets(
    from_serial: Receiver<Vec<u8>>,
    to_ws: Sender<String>,
    to_serial: Sender<Vec<u8>>,
) {
    while let Ok(msg) = from_serial.recv().await {
        if let Ok(msg) = SerialMessage::try_from_bytes(&msg[..]) {
            if let Err(err) = handle_serial_message(&to_serial, &to_ws, msg).await {
                tracing::error!("Error on handling serial message: {err}");
            }
        }
    }
}

async fn handle_serial_message(
    to_serial: &Sender<Vec<u8>>,
    to_ws: &Sender<String>,
    msg: SerialMessage,
) -> anyhow::Result<()> {
    match msg {
        SerialMessage::Ping => {
            to_serial
                .send(SerialMessage::PingResponse.to_bytes())
                .await?;
        }
        SerialMessage::UpdateRow(UpdateRow {
            row_number,
            row_data_len,
            row_data,
        }) => {
            if usize::from((row_data_len / 8) + if row_data_len % 8 == 0 { 0 } else { 1 })
                == row_data.len()
            {
                let pixel_states = row_data
                    .into_iter()
                    .map(|byte| {
                        (0..8)
                            .into_iter()
                            .map(move |bit| (byte & (1 << bit)) != 0x00)
                    })
                    .flatten()
                    .collect::<Vec<bool>>();
                if let Ok(msg) = serde_json::to_string(&SimMessage::SetMatrixRow(SetMatrixRow {
                    row: usize::from(row_number),
                    data: pixel_states,
                })) {
                    let _ = to_ws.send(msg).await;
                    to_serial.send(vec![0xa0, 0x01, 0x00]).await?;
                } else {
                    to_serial.send(vec![0xa0, 0x01, 0x01]).await?;
                }
            } else {
                tracing::warn!("Got a request to write a matrix row of invalid length");
                to_serial.send(vec![0xa0, 0x01, 0x01]).await?;
            }
        }
        SerialMessage::SetLedState(SetLedState { new_state }) => {
            if let Ok(msg) =
                serde_json::to_string(&SimMessage::SetDebugLed(SetDebugLed { new_state }))
            {
                let _ = to_ws.send(msg).await;
            }
            if !new_state {
                tracing::info!("Got request to disable debug LED");
            } else {
                tracing::info!("Got request to enable debug LED");
            }
            to_serial.send(vec![0xde, 0x01, 0x00]).await?;
        }
        SerialMessage::SetRgbState(SetRgbState { r, g, b }) => {
            if let Ok(msg) = serde_json::to_string(&SimMessage::SetRgbLed(SetRgbLed { r, g, b })) {
                let _ = to_ws.send(msg).await;
            }
            to_serial.send(vec![0xde, 0x03, 0x00]).await?;
        }
        _ => tracing::debug!("Unhandled message received"),
    }

    Ok(())
}

async fn handle_ws_message(from_ws: Receiver<String>, to_serial: Sender<Vec<u8>>) {
    while let Ok(msg_str) = from_ws.recv().await {
        if let Ok(msg) = serde_json::from_str::<SimMessage>(&msg_str) {
            match msg {
                SimMessage::ReportButtonPress => {
                    tracing::debug!("Sending button press notification");
                    to_serial.send(vec![0xde, 0x04]).await.unwrap();
                }
                SimMessage::FrontendStarted => {
                    tracing::debug!("Got message indicating that the frontend is started");
                }
                _ => {
                    tracing::warn!("Got unexpected message from the frontend: {msg_str}");
                }
            }
        }
    }
}
