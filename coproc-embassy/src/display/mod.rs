use crate::msg_router::display_cmd_router::DisplayCommand;
use embassy_sync::{
    blocking_mutex::raw::NoopRawMutex,
    channel::{Receiver, Sender},
};

#[cfg(feature = "dot_matrix")]
pub mod dot_matrix;
#[cfg(feature = "rgb_matrix")]
pub mod rgb_matrix;

#[cfg(feature = "dot_matrix")]
pub use dot_matrix::{DotMatrix, COLUMNS, DISPLAY_CMD_QUEUE_SIZE, ROWS};

#[cfg(feature = "rgb_matrix")]
pub use rgb_matrix::{DriverPins, WaveshareDriver, COLUMNS, DISPLAY_CMD_QUEUE_SIZE, ROWS};

pub type DisplayCmdSender = Sender<'static, NoopRawMutex, DisplayCommand, DISPLAY_CMD_QUEUE_SIZE>;
pub type DisplayCmdReceiver =
    Receiver<'static, NoopRawMutex, DisplayCommand, DISPLAY_CMD_QUEUE_SIZE>;
