use std::{sync::Arc, time::Duration};

use crate::transport::Connection;
use async_channel::{Receiver, Sender, TryRecvError};
use megabit_serial_protocol::SerialMessage;

#[derive(Clone, Debug)]
pub enum Event {
    NextAppRequest,
    PreviousAppRequest,
    ResumePauseRequest,
    ReloadAppsRequest,
    Shutdown,
}

#[derive(Clone)]
pub struct EventListener {
    pending_event: Option<Event>,
    event_rx: Receiver<Event>,
    _handle: Arc<tokio::task::JoinHandle<()>>,
}

impl EventListener {
    pub fn new(conn: Connection, rt_handle: tokio::runtime::Handle) -> Self {
        let (tx, rx) = async_channel::bounded(10);
        let handle = rt_handle.spawn(event_listener_task(tx, conn));
        Self {
            pending_event: None,
            event_rx: rx,
            _handle: Arc::new(handle),
        }
    }

    pub fn has_pending_events(&mut self) -> bool {
        match &self.pending_event {
            Some(_) => true,
            None => match self.event_rx.try_recv() {
                Ok(event) => {
                    self.pending_event = Some(event);
                    true
                }
                Err(TryRecvError::Empty) => false,
                Err(TryRecvError::Closed) => {
                    tracing::error!("Event listener sender dropped");
                    self.pending_event = Some(Event::Shutdown);
                    true
                }
            },
        }
    }

    pub fn next(&mut self) -> Option<Event> {
        self.pending_event.take().map(|event| {
            tracing::info!("Consuming event: {event:?}");
            event
        })
    }
}

async fn event_listener_task(tx: Sender<Event>, conn: Connection) {
    let button_press_matcher =
        Box::new(|msg: &SerialMessage| matches!(msg, &SerialMessage::ReportButtonPress));

    loop {
        if conn
            .wait_for_message(button_press_matcher.clone(), Some(Duration::from_secs(1)))
            .await
            .is_some()
        {
            tracing::info!("Received button press");
            if let Err(err) = tx.send(Event::NextAppRequest).await {
                tracing::error!("Failed to send event from event listener task: {err:?}");
                return;
            }
        } else {
            if tx.receiver_count() == 0 {
                tracing::warn!("All event listener receivers have hung up, exiting");
                return;
            }
        }
    }
}
