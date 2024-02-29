#[cfg(feature = "dot_matrix")]
pub mod dot_matrix;
#[cfg(feature = "rgb_matrix")]
pub mod rgb_matrix;

#[cfg(feature = "dot_matrix")]
pub use dot_matrix::{
    DisplayCmdReceiver, DisplayCmdSender, DotMatrix, COLUMNS, DISPLAY_CMD_QUEUE_SIZE, ROWS,
};

#[cfg(feature = "rgb_matrix")]
pub use rgb_matrix::{COLUMNS, ROWS};
