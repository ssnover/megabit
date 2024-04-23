use crate::{api_server::ApiServerHandle, transport::Connection};
use async_channel::{Receiver, Sender, TryRecvError};
use megabit_runner_msgs::ConsoleMessage;
use megabit_serial_protocol::SerialMessage;
use std::{sync::Arc, time::Duration};

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
    pub fn new(
        conn: Connection,
        api_server_handle: ApiServerHandle,
        rt_handle: tokio::runtime::Handle,
    ) -> Self {
        let (tx, rx) = async_channel::bounded(10);
        let handle = rt_handle.spawn(event_listener_task(tx, conn, api_server_handle));
        Self {
            pending_event: None,
            event_rx: rx,
            _handle: Arc::new(handle),
        }
    }

    fn try_get_next_event(&mut self) {
        match self.event_rx.try_recv() {
            Ok(event) => {
                self.pending_event = Some(event);
            }
            Err(TryRecvError::Empty) => {}
            Err(TryRecvError::Closed) => {
                tracing::error!("Event listener sender dropped");
                self.pending_event = Some(Event::Shutdown);
            }
        }
    }

    pub fn has_pending_events(&mut self) -> bool {
        self.try_get_next_event();
        self.pending_event.is_some()
    }

    pub fn next(&mut self) -> Option<Event> {
        self.try_get_next_event();
        self.pending_event.take().map(|event| {
            tracing::info!("Consuming event: {event:?}");
            event
        })
    }
}

async fn event_listener_task(
    tx: Sender<Event>,
    conn: Connection,
    api_server_handle: ApiServerHandle,
) {
    tokio::join!(
        api_listener_task(tx.clone(), api_server_handle),
        button_press_listener_task(tx.clone(), conn),
    );
}

async fn button_press_listener_task(tx: Sender<Event>, conn: Connection) {
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

async fn api_listener_task(tx: Sender<Event>, api_server_handle: ApiServerHandle) {
    while let Ok(msg) = api_server_handle.next().await {
        let event = match msg {
            ConsoleMessage::NextApp => Event::NextAppRequest,
            ConsoleMessage::PauseRendering => Event::ResumePauseRequest,
            ConsoleMessage::PreviousApp => Event::PreviousAppRequest,
            ConsoleMessage::ResumeRendering => Event::ResumePauseRequest,
            _ => {
                continue;
            }
        };
        tracing::info!("Received console message event: {event:?}");
        if let Err(err) = tx.send(event).await {
            tracing::error!("API listener task unable to send event, exiting {err:?}");
            break;
        }
    }
}
