use async_channel::{Receiver, Sender};
use megabit_serial_protocol::SerialMessage;
use std::{
    collections::VecDeque,
    sync::{Arc, Mutex, Weak},
    time::{Duration, Instant},
};

pub enum HandleNotification {
    NewMessages,
    ClosedConnection,
}

pub struct MessageInbox {
    msg_rx: Receiver<SerialMessage>,
    msg_queue: Arc<Mutex<VecDeque<(Instant, SerialMessage)>>>,
    notification_tx: Sender<HandleNotification>,
    notification_rx: Receiver<HandleNotification>,
    msg_expiration_duration: Option<Duration>,
}

#[derive(Clone, Debug)]
pub struct InboxHandle {
    msg_queue: Weak<Mutex<VecDeque<(Instant, SerialMessage)>>>,
    notification_rx: Receiver<HandleNotification>,
}

impl MessageInbox {
    pub fn new(msg_rx: Receiver<SerialMessage>, msg_expiration_age: Option<Duration>) -> Self {
        let (tx, rx) = async_channel::bounded(1);
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
                tracing::debug!("Inbox got {msg:?}");
                msg_queue.push_back((std::time::Instant::now(), msg));

                if let Some(expiration_age) = self.msg_expiration_duration {
                    while let Some((receive_time, _msg)) = msg_queue.get(0) {
                        if receive_time.elapsed() > expiration_age {
                            let _ = msg_queue.pop_front();
                        } else {
                            break;
                        }
                    }
                }
            }
            tracing::trace!("Alerting handles of new messages");
            let _ = self
                .notification_tx
                .send(HandleNotification::NewMessages)
                .await;
        }

        let _ = self
            .notification_tx
            .send(HandleNotification::ClosedConnection)
            .await;
        tracing::debug!("Stopping message inbox");
    }
}

impl InboxHandle {
    pub async fn wait_for_message(
        &self,
        matcher: Box<dyn Fn(&SerialMessage) -> bool>,
        timeout: Option<Duration>,
    ) -> Option<SerialMessage> {
        let timeout_instant = timeout.map(|duration| std::time::Instant::now() + duration);
        loop {
            let msg = if let Some(msg_queue) = self.msg_queue.upgrade() {
                let queue = msg_queue.lock().expect("Mutex locks");
                let msg = queue.iter().find_map(|(_received_time, msg)| {
                    if matcher(msg) {
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
                break Some(msg);
            } else {
                tracing::trace!("No match, waiting for new messages");
                let notification = if let Some(timeout_instant) = timeout_instant {
                    let time_left = timeout_instant - std::time::Instant::now();
                    match tokio::time::timeout(time_left, self.notification_rx.recv()).await {
                        Ok(res) => res,
                        Err(_) => break None,
                    }
                } else {
                    self.notification_rx.recv_blocking()
                };

                match notification {
                    Ok(HandleNotification::NewMessages) => continue,
                    Ok(HandleNotification::ClosedConnection) | Err(_) => {
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
        matcher: Box<dyn Fn(&SerialMessage) -> bool>,
        start_time: Instant,
    ) -> Option<SerialMessage> {
        if let Some(msg_queue) = self.msg_queue.upgrade() {
            let msg_queue = msg_queue.lock().expect("Mutex locks");
            msg_queue.iter().find_map(|(receive_time, msg)| {
                if *receive_time >= start_time && matcher(msg) {
                    Some(msg.clone())
                } else {
                    None
                }
            })
        } else {
            None
        }
    }
}
