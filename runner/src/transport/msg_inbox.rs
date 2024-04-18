use megabit_serial_protocol::SerialMessage;
use std::{
    collections::VecDeque,
    sync::{Arc, Mutex, Weak},
    time::{Duration, Instant},
};
use tokio::sync::watch::{channel, Receiver, Sender};

type MessageQueue = VecDeque<(Instant, Box<SerialMessage>)>;

#[derive(Clone, Copy)]
pub enum HandleNotification {
    NewMessages(Instant),
    ClosedConnection,
}

pub struct MessageInbox {
    msg_rx: async_channel::Receiver<SerialMessage>,
    msg_queue: Arc<Mutex<MessageQueue>>,
    notification_tx: Sender<HandleNotification>,
    notification_rx: Receiver<HandleNotification>,
    msg_expiration_duration: Option<Duration>,
}

#[derive(Clone)]
pub struct InboxHandle {
    msg_queue: Weak<Mutex<MessageQueue>>,
    notification_rx: Receiver<HandleNotification>,
}

impl MessageInbox {
    pub fn new(
        msg_rx: async_channel::Receiver<SerialMessage>,
        msg_expiration_age: Option<Duration>,
    ) -> Self {
        let (tx, rx) = channel(HandleNotification::NewMessages(Instant::now()));
        Self {
            msg_rx,
            msg_queue: Arc::new(Mutex::new(VecDeque::new())),
            notification_tx: tx,
            notification_rx: rx,
            msg_expiration_duration: msg_expiration_age,
        }
    }

    pub fn get_handle(&self) -> InboxHandle {
        InboxHandle {
            msg_queue: Arc::downgrade(&self.msg_queue),
            notification_rx: self.notification_rx.clone(),
        }
    }

    pub async fn run(self) {
        while let Ok(msg) = self.msg_rx.recv().await {
            if matches!(msg, SerialMessage::PingResponse) {
                continue;
            }
            {
                let mut msg_queue = self.msg_queue.lock().unwrap();
                tracing::debug!("Inbox got {:?}", msg.as_ref());
                msg_queue.push_back((std::time::Instant::now(), Box::new(msg)));

                if let Some(expiration_age) = self.msg_expiration_duration {
                    while let Some((receive_time, _msg)) = msg_queue.front() {
                        if receive_time.elapsed() > expiration_age {
                            let _ = msg_queue.pop_front();
                        } else {
                            break;
                        }
                    }
                }
            }
            tracing::trace!("Alerting handles of new messages");
            if self
                .notification_tx
                .send(HandleNotification::NewMessages(Instant::now()))
                .is_err()
            {
                break;
            }
        }

        let _ = self
            .notification_tx
            .send(HandleNotification::ClosedConnection);
        tracing::debug!("Stopping message inbox");
    }
}

impl InboxHandle {
    pub async fn wait_for_message(
        &mut self,
        matcher: Box<dyn Fn(&SerialMessage) -> bool + Send + Sync>,
        timeout: Option<Duration>,
    ) -> Option<SerialMessage> {
        let start_time = Instant::now();
        let timeout_instant = timeout.map(|duration| std::time::Instant::now() + duration);
        loop {
            let msg = if let Some(msg_queue) = self.msg_queue.upgrade() {
                let queue = msg_queue.lock().expect("Mutex locks");
                let msg = queue.iter().find_map(|(received_time, msg)| {
                    if matcher(msg) && *received_time > start_time {
                        Some(msg.clone())
                    } else {
                        None
                    }
                });
                msg
            } else {
                tracing::debug!("Message inbox has been deleted, no messages to search");
                break None;
            };

            if let Some(msg) = msg {
                break Some(*msg);
            } else {
                tracing::trace!("No match, waiting for new messages");
                let notification = if let Some(timeout_instant) = timeout_instant {
                    let time_left = timeout_instant - std::time::Instant::now();
                    match tokio::time::timeout(time_left, self.notification_rx.changed()).await {
                        Ok(Ok(())) => *self.notification_rx.borrow_and_update(),
                        Ok(Err(_)) | Err(_) => break None,
                    }
                } else {
                    *self.notification_rx.borrow_and_update()
                };

                match notification {
                    HandleNotification::NewMessages(_) => continue,
                    HandleNotification::ClosedConnection => {
                        tracing::debug!(
                            "Notification channel from inbox closed, no messages to search"
                        );
                        break None;
                    }
                }
            }
        }
    }

    pub fn check_for_message_since(
        &self,
        matcher: Box<dyn Fn(&SerialMessage) -> bool + Send + Sync>,
        start_time: Instant,
    ) -> Option<SerialMessage> {
        if let Some(msg_queue) = self.msg_queue.upgrade() {
            let msg_queue = msg_queue.lock().expect("Mutex locks");
            msg_queue.iter().find_map(|(receive_time, msg)| {
                if *receive_time >= start_time && matcher(msg.as_ref()) {
                    Some(*msg.clone())
                } else {
                    None
                }
            })
        } else {
            None
        }
    }
}
