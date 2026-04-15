use async_channel::{Receiver, Sender};
use std::{future::Future, sync::Arc};

mod cmd;
pub use cmd::Command;

pub trait CommandClientIntf {
    fn push_blocking(&self, cmd: Box<Command>) -> anyhow::Result<()>;

    fn push(&self, cmd: Box<Command>) -> impl Future<Output = anyhow::Result<()>> + Sync;
}

#[derive(Clone)]
pub struct CommandClient {
    tx: Sender<Box<Command>>,
    _handle: Arc<tokio::task::JoinHandle<()>>,
}

impl CommandClientIntf for CommandClient {
    fn push_blocking(&self, cmd: Box<Command>) -> anyhow::Result<()> {
        self.tx.send_blocking(cmd)?;
        Ok(())
    }

    async fn push(&self, cmd: Box<Command>) -> anyhow::Result<()> {
        self.tx.send(cmd).await?;
        Ok(())
    }
}

pub struct CommandQueue {
    rx: Receiver<Box<Command>>,
}

impl CommandQueue {
    pub fn start(rt: tokio::runtime::Handle) -> Box<CommandClient> {
        let (tx, rx) = async_channel::bounded(100);
        let queue = CommandQueue { rx };
        let queue_context = async move {
            let mut queue = queue;
            queue.handle_commands().await
        };
        let handle = rt.spawn(queue_context);
        let client = CommandClient {
            tx,
            _handle: Arc::new(handle),
        };
        Box::new(client)
    }

    async fn handle_commands(&mut self) {
        loop {
            match self.rx.recv().await {
                Ok(cmd) => {
                    todo!();
                }
                Err(_) => {
                    tracing::debug!("Stopping command queue because there are no clients");
                    break;
                }
            }
        }
    }
}
