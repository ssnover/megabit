use super::{display_buffer::DisplayBuffer, Color};
use async_channel::{Receiver, Sender};
use std::{
    path::PathBuf,
    sync::{Arc, Mutex},
    time::Duration,
};
use tokio::sync::oneshot;

enum RecorderRequest {
    Start(Duration, oneshot::Sender<bool>),
    Stop(oneshot::Sender<anyhow::Result<PathBuf>>),
    CaptureFrame(oneshot::Sender<()>),
    Timeout,
    Close,
}

#[derive(Clone)]
pub struct RecorderClient {
    _handle: Arc<tokio::task::JoinHandle<()>>,
    recorder_tx: Sender<RecorderRequest>,
}

impl RecorderClient {
    pub async fn start(&self, duration: Duration) -> anyhow::Result<()> {
        let (tx, rx) = oneshot::channel();
        self.recorder_tx
            .send(RecorderRequest::Start(duration, tx))
            .await?;
        if rx.await? {
            Ok(())
        } else {
            Err(anyhow::anyhow!("Failed to start recording"))
        }
    }

    pub async fn stop(&self) -> anyhow::Result<PathBuf> {
        let (tx, rx) = oneshot::channel();
        self.recorder_tx.send(RecorderRequest::Stop(tx)).await?;
        rx.await?
    }

    pub async fn capture_frame(&self) -> anyhow::Result<()> {
        let (tx, rx) = oneshot::channel();
        self.recorder_tx
            .send(RecorderRequest::CaptureFrame(tx))
            .await?;
        rx.await?;
        Ok(())
    }
}

impl Drop for RecorderClient {
    fn drop(&mut self) {
        let tx = self.recorder_tx.clone();
        tokio::spawn(async move {
            let _ = tx.send(RecorderRequest::Close).await;
        });
    }
}

pub fn start_recorder(display_buffer: Arc<Mutex<DisplayBuffer>>) -> RecorderClient {
    let (tx, rx) = async_channel::unbounded();
    let recorder = Recorder::new(tx.clone(), rx, display_buffer);
    let handle = tokio::spawn(async move { recorder.run().await });

    RecorderClient {
        _handle: Arc::new(handle),
        recorder_tx: tx,
    }
}

pub struct Recorder {
    request_tx: Sender<RecorderRequest>,
    request_rx: Receiver<RecorderRequest>,
    display_buffer: Arc<Mutex<DisplayBuffer>>,
    frames: Vec<Vec<u16>>,
    dimensions: (usize, usize),
    timeout_handle: Option<tokio::task::JoinHandle<()>>,
}

impl Recorder {
    fn new(
        request_tx: Sender<RecorderRequest>,
        request_rx: Receiver<RecorderRequest>,
        display_buffer: Arc<Mutex<DisplayBuffer>>,
    ) -> Self {
        let dimensions = {
            let display_buffer = display_buffer.lock().unwrap();
            display_buffer.dims()
        };
        Self {
            request_tx,
            request_rx,
            display_buffer,
            frames: vec![],
            dimensions,
            timeout_handle: None,
        }
    }

    async fn run(mut self) {
        while let Ok(request) = self.request_rx.recv().await {
            match request {
                RecorderRequest::Start(duration, reply) => {
                    let res = self.start_recording(duration);
                    let _ = reply.send(res);
                }
                RecorderRequest::Stop(reply) => {
                    let res = self.stop_recording();
                    let _ = reply.send(res);
                }
                RecorderRequest::CaptureFrame(reply) => {
                    let _res = self.capture_frame();
                    let _ = reply.send(());
                }
                RecorderRequest::Timeout => match self.stop_recording() {
                    Ok(path) => tracing::info!("Saving recording to {}", path.display()),
                    Err(err) => tracing::error!("Failed to save recording: {err}"),
                },
                RecorderRequest::Close => {
                    break;
                }
            }
        }

        tracing::info!("Exiting recorder task");
    }

    fn in_progress(&self) -> bool {
        self.timeout_handle.is_some()
    }

    fn start_recording(&mut self, duration: Duration) -> bool {
        if self.in_progress() {
            false
        } else {
            let tx = self.request_tx.clone();
            self.timeout_handle = Some(tokio::spawn(async move {
                tokio::time::sleep(duration).await;
                tracing::warn!("Timed out on recording, stopping early");
                let _ = tx.send(RecorderRequest::Timeout).await;
            }));
            true
        }
    }

    fn stop_recording(&mut self) -> anyhow::Result<PathBuf> {
        if self.in_progress() {
            self.timeout_handle.as_ref().unwrap().abort();
            self.timeout_handle = None;
            self.save_recording()
        } else {
            Err(anyhow::anyhow!(
                "No recording in progress, but got request to stop recording"
            ))
        }
    }

    fn capture_frame(&mut self) -> bool {
        if self.in_progress() {
            let display_buffer = self.display_buffer.lock().unwrap();
            let frame = Vec::from(display_buffer.get_frame());
            self.frames.push(frame);
            tracing::info!("Adding frame");
            true
        } else {
            false
        }
    }

    fn save_recording(&mut self) -> anyhow::Result<PathBuf> {
        if self.frames.is_empty() {
            return Err(anyhow::anyhow!(
                "No frames in the recording, nothing to save!"
            ));
        }
        let uuid = uuid::Uuid::new_v4().to_string();
        let path = PathBuf::from(format!("/tmp/{uuid}.gif"));
        let mut image_file = std::fs::File::create(&path)?;
        let mut encoder = gif::Encoder::new(
            &mut image_file,
            self.dimensions.0 as u16,
            self.dimensions.1 as u16,
            &[],
        )?;
        encoder.set_repeat(gif::Repeat::Infinite).unwrap();

        let n_frames = self.frames.len();
        for captured_frame in &self.frames {
            let mut frame = gif::Frame::from_rgb(
                self.dimensions.0 as u16,
                self.dimensions.1 as u16,
                captured_frame
                    .iter()
                    .map(|pixel| Color(*pixel).to_rgb())
                    .flatten()
                    .collect::<Vec<_>>()
                    .as_slice(),
            );
            frame.delay = 30;
            encoder.write_frame(&frame)?;
        }
        self.frames.clear();
        tracing::info!("Saving {n_frames} frames to {}", path.display());

        Ok(path)
    }
}
