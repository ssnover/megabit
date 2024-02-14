use super::DisplayConfiguration;
use crate::messages::{SetDebugLed, SetMatrixRow, SetMatrixRowRgb, SetRgbLed, SimMessage};
use async_channel::{Receiver, Sender};
use megabit_serial_protocol::*;
use std::convert::AsRef;

pub async fn run(
    from_ws: Receiver<String>,
    from_serial: Receiver<Vec<u8>>,
    to_ws: Sender<String>,
    to_serial: Sender<Vec<u8>>,
    display_cfg: DisplayConfiguration,
) {
    tokio::select! {
        _ = handle_serial_packets(from_serial, to_ws.clone(), to_serial.clone(), display_cfg) => {
            tracing::info!("Serial handler exited");
        },
        _ = handle_ws_message(from_ws, to_serial, to_ws, &display_cfg) => {
            tracing::info!("Websocket message handler exited");
        }
    }
}

async fn handle_serial_packets(
    from_serial: Receiver<Vec<u8>>,
    to_ws: Sender<String>,
    to_serial: Sender<Vec<u8>>,
    display_cfg: DisplayConfiguration,
) {
    while let Ok(msg) = from_serial.recv().await {
        if let Ok(msg) = SerialMessage::try_from_bytes(&msg[..]) {
            if let Err(err) = handle_serial_message(&to_serial, &to_ws, &display_cfg, msg).await {
                tracing::error!("Error on handling serial message: {err}");
            }
        }
    }
}

async fn handle_serial_message(
    to_serial: &Sender<Vec<u8>>,
    to_ws: &Sender<String>,
    display_cfg: &DisplayConfiguration,
    msg: SerialMessage,
) -> anyhow::Result<()> {
    tracing::debug!("Handling serial message: {}", msg.as_ref());
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
            let status =
                handle_update_row(to_ws, display_cfg, row_number, row_data_len, row_data).await;
            to_serial
                .send(SerialMessage::UpdateRowResponse(UpdateRowResponse { status }).to_bytes())
                .await?;
        }
        SerialMessage::UpdateRowRgb(UpdateRowRgb {
            row_number,
            row_data_len: _,
            row_data,
        }) => {
            to_ws
                .send(
                    serde_json::to_string(&SimMessage::SetMatrixRowRgb(SetMatrixRowRgb {
                        row: row_number as usize,
                        data: row_data,
                    }))
                    .unwrap(),
                )
                .await?;
            to_serial
                .send(
                    SerialMessage::UpdateRowRgbResponse(UpdateRowRgbResponse {
                        status: Status::Success,
                    })
                    .to_bytes(),
                )
                .await?;
        }
        SerialMessage::GetDisplayInfo(GetDisplayInfo) => {
            to_serial
                .send(SerialMessage::GetDisplayInfoResponse(display_cfg.clone().into()).to_bytes())
                .await?
        }
        SerialMessage::SetLedState(SetLedState { new_state }) => {
            if let Ok(msg) =
                serde_json::to_string(&SimMessage::SetDebugLed(SetDebugLed { new_state }))
            {
                let _ = to_ws.send(msg).await;
            }
            to_serial
                .send(
                    SerialMessage::SetLedStateResponse(SetLedStateResponse {
                        status: Status::Success,
                    })
                    .to_bytes(),
                )
                .await?;
        }
        SerialMessage::SetRgbState(SetRgbState { r, g, b }) => {
            if let Ok(msg) = serde_json::to_string(&SimMessage::SetRgbLed(SetRgbLed { r, g, b })) {
                let _ = to_ws.send(msg).await;
            }
            to_serial
                .send(
                    SerialMessage::SetRgbStateResponse(SetRgbStateResponse {
                        status: Status::Success,
                    })
                    .to_bytes(),
                )
                .await?;
        }
        _ => tracing::debug!("Unhandled message received"),
    }

    Ok(())
}

async fn handle_update_row(
    to_ws: &Sender<String>,
    display_cfg: &DisplayConfiguration,
    row_number: u8,
    row_data_len: u8,
    row_data: Vec<u8>,
) -> Status {
    if usize::from((row_data_len / 8) + if row_data_len % 8 == 0 { 0 } else { 1 }) == row_data.len()
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
        if display_cfg.is_rgb {
            Status::Failure
        } else {
            if let Ok(msg) = serde_json::to_string(&SimMessage::SetMatrixRow(SetMatrixRow {
                row: usize::from(row_number),
                data: pixel_states,
            })) {
                let _ = to_ws.send(msg).await;
                Status::Success
            } else {
                Status::Failure
            }
        }
    } else {
        tracing::warn!("Got a request to write a matrix row of invalid length");
        Status::Failure
    }
}

async fn handle_ws_message(
    from_ws: Receiver<String>,
    to_serial: Sender<Vec<u8>>,
    to_ws: Sender<String>,
    display_cfg: &DisplayConfiguration,
) {
    while let Ok(msg_str) = from_ws.recv().await {
        if let Ok(msg) = serde_json::from_str::<SimMessage>(&msg_str) {
            match msg {
                SimMessage::ReportButtonPress => {
                    tracing::debug!("Sending button press notification");
                    to_serial
                        .send(SerialMessage::ReportButtonPress.to_bytes())
                        .await
                        .unwrap();
                }
                SimMessage::FrontendStarted => {
                    tracing::debug!("Got message indicating that the frontend is started");
                    if display_cfg.is_rgb {
                        to_ws
                            .send(serde_json::to_string(&SimMessage::RequestRgb).unwrap())
                            .await
                            .unwrap();
                    }
                }
                _ => {
                    tracing::warn!("Got unexpected message from the frontend: {msg_str}");
                }
            }
        }
    }
}
