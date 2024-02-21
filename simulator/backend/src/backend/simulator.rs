use crate::backend::recorder::start_recorder;

use super::DisplayConfiguration;
use super::{display_buffer::DisplayBuffer, recorder::RecorderClient};
use async_channel::{Receiver, Sender};
use megabit_serial_protocol::*;
use megabit_sim_msgs::{SetDebugLed, SetMatrixRow, SetMatrixRowRgb, SetRgbLed, SimMessage};
use std::time::Duration;
use std::{
    convert::AsRef,
    sync::{Arc, Mutex},
};

pub async fn run(
    from_ws: Receiver<Vec<u8>>,
    from_serial: Receiver<Vec<u8>>,
    to_ws: Sender<Vec<u8>>,
    to_serial: Sender<Vec<u8>>,
    display_cfg: DisplayConfiguration,
) {
    let display_buffer = Arc::new(Mutex::new(DisplayBuffer::new(
        display_cfg.width as usize,
        display_cfg.height as usize,
    )));
    let recorder = start_recorder(display_buffer.clone());

    tokio::select! {
        _ = handle_serial_packets(from_serial, to_ws.clone(), to_serial.clone(), display_cfg, display_buffer.clone(), recorder.clone()) => {
            tracing::info!("Serial handler exited");
        },
        _ = handle_ws_message(from_ws, to_serial, to_ws, &display_cfg, recorder) => {
            tracing::info!("Websocket message handler exited");
        }
    }
}

async fn handle_serial_packets(
    from_serial: Receiver<Vec<u8>>,
    to_ws: Sender<Vec<u8>>,
    to_serial: Sender<Vec<u8>>,
    display_cfg: DisplayConfiguration,
    display_buffer: Arc<Mutex<DisplayBuffer>>,
    recorder: RecorderClient,
) {
    while let Ok(msg) = from_serial.recv().await {
        if let Ok(msg) = SerialMessage::try_from_bytes(&msg[..]) {
            if let Err(err) = handle_serial_message(
                &to_serial,
                &to_ws,
                &display_cfg,
                &display_buffer,
                &recorder,
                msg,
            )
            .await
            {
                tracing::error!("Error on handling serial message: {err}");
            }
        } else {
            if msg.len() >= 2 {
                tracing::warn!(
                    "Failed to parse serial message with message type: 0x{:02x}{:02x}",
                    msg[0],
                    msg[1]
                )
            } else {
                tracing::error!(
                    "Got message with length {}, not even long enough to have a message type",
                    msg.len()
                );
            }
        }
    }
}

async fn handle_serial_message(
    to_serial: &Sender<Vec<u8>>,
    to_ws: &Sender<Vec<u8>>,
    display_cfg: &DisplayConfiguration,
    display_buffer: &Arc<Mutex<DisplayBuffer>>,
    recorder: &RecorderClient,
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
            let status = handle_update_row(
                to_ws,
                display_cfg,
                &display_buffer,
                row_number,
                row_data_len,
                row_data,
            )
            .await;
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
                    rmp_serde::to_vec(&SimMessage::SetMatrixRowRgb(SetMatrixRowRgb {
                        row: row_number as usize,
                        data: row_data.clone(),
                    }))
                    .unwrap(),
                )
                .await?;
            {
                let mut display_buffer = display_buffer.lock().unwrap();
                display_buffer.update_row_rgb(row_number, row_data);
            }
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
            if let Ok(msg) = rmp_serde::to_vec(&SimMessage::SetDebugLed(SetDebugLed { new_state }))
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
            if let Ok(msg) = rmp_serde::to_vec(&SimMessage::SetRgbLed(SetRgbLed { r, g, b })) {
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
        SerialMessage::RequestCommitRender(RequestCommitRender {}) => {
            if display_cfg.is_rgb {
                let _ = to_ws
                    .send(rmp_serde::to_vec(&SimMessage::RequestRgb).unwrap())
                    .await;
            }
            if let Ok(msg) = rmp_serde::to_vec(&SimMessage::CommitRender) {
                let _ = to_ws.send(msg).await;
            }
            if let Err(err) = recorder.capture_frame().await {
                tracing::error!("Tried to capture frame on render: {err}");
            }
            to_serial
                .send(SerialMessage::CommitRenderResponse(CommitRenderResponse {}).to_bytes())
                .await?;
        }
        m => tracing::debug!("Unhandled message received: {}", m.as_ref()),
    }

    Ok(())
}

async fn handle_update_row(
    to_ws: &Sender<Vec<u8>>,
    display_cfg: &DisplayConfiguration,
    display_buffer: &Arc<Mutex<DisplayBuffer>>,
    row_number: u8,
    row_data_len: u8,
    row_data: Vec<u8>,
) -> Status {
    if usize::from((row_data_len / 8) + if row_data_len % 8 == 0 { 0 } else { 1 }) == row_data.len()
    {
        if display_cfg.is_rgb {
            Status::Failure
        } else {
            let pixel_states = row_data
                .into_iter()
                .map(|byte| {
                    (0..8)
                        .into_iter()
                        .map(move |bit| (byte & (1 << bit)) != 0x00)
                })
                .flatten()
                .collect::<Vec<bool>>();
            {
                let mut display_buffer = display_buffer.lock().unwrap();
                display_buffer.update_row(row_number, pixel_states.clone());
            }
            if let Ok(msg) = rmp_serde::to_vec(&SimMessage::SetMatrixRow(SetMatrixRow {
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
    from_ws: Receiver<Vec<u8>>,
    to_serial: Sender<Vec<u8>>,
    to_ws: Sender<Vec<u8>>,
    display_cfg: &DisplayConfiguration,
    recorder: RecorderClient,
) {
    while let Ok(msg) = from_ws.recv().await {
        if let Ok(msg) = rmp_serde::from_slice::<SimMessage>(&msg) {
            match msg {
                SimMessage::ReportButtonPress => {
                    tracing::info!("Sending button press notification");
                    to_serial
                        .send(SerialMessage::ReportButtonPress.to_bytes())
                        .await
                        .unwrap();
                }
                SimMessage::FrontendStarted => {
                    tracing::debug!("Got message indicating that the frontend is started");
                    if display_cfg.is_rgb {
                        to_ws
                            .send(rmp_serde::to_vec(&SimMessage::RequestRgb).unwrap())
                            .await
                            .unwrap();
                    }
                }
                SimMessage::StartRecording => {
                    if let Err(err) = recorder.start(Duration::from_secs(30)).await {
                        tracing::error!("Failed to start recording: {err}");
                    }
                }
                SimMessage::StopRecording => {
                    if let Err(err) = recorder.stop().await {
                        tracing::error!("Failed to stop recording: {err}");
                    }
                }
                _ => {
                    tracing::warn!("Got unexpected message from the frontend: {msg:?}");
                }
            }
        }
    }
}
