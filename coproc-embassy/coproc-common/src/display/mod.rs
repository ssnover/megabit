use core::sync::atomic::{AtomicBool, Ordering};

use crate::msg_router::display_cmd_router::DisplayCommand;
use embassy_sync::{
    blocking_mutex::raw::{NoopRawMutex, RawMutex},
    channel::{Receiver, Sender},
    mutex::Mutex,
};

#[cfg(feature = "dot_matrix")]
pub mod dot_matrix;
#[cfg(feature = "rgb_matrix")]
pub mod rgb_matrix;

#[cfg(feature = "dot_matrix")]
pub use dot_matrix::{COLUMNS, DISPLAY_CMD_QUEUE_SIZE, DotMatrix, ROWS};

#[cfg(feature = "rgb_matrix")]
pub use rgb_matrix::{COLUMNS, DISPLAY_CMD_QUEUE_SIZE, DriverPins, ROWS, WaveshareDriver};

pub type DisplayCmdSender = Sender<'static, NoopRawMutex, DisplayCommand, DISPLAY_CMD_QUEUE_SIZE>;
pub type DisplayCmdReceiver =
    Receiver<'static, NoopRawMutex, DisplayCommand, DISPLAY_CMD_QUEUE_SIZE>;

pub struct PixelBuffer<M: RawMutex + 'static, const N: usize> {
    buffer_a: Mutex<M, [u16; N]>,
    buffer_b: Mutex<M, [u16; N]>,
    write_a: AtomicBool,
}

impl<M: RawMutex + 'static, const N: usize> PixelBuffer<M, N> {
    pub fn new(buffer_a: Mutex<M, [u16; N]>, buffer_b: Mutex<M, [u16; N]>) -> Self {
        Self {
            buffer_a,
            buffer_b,
            write_a: AtomicBool::new(true),
        }
    }

    pub fn write_buffer(&self) -> &Mutex<M, [u16; N]> {
        if self.write_a.load(Ordering::SeqCst) {
            &self.buffer_a
        } else {
            &self.buffer_b
        }
    }

    pub fn read_buffer(&self) -> &Mutex<M, [u16; N]> {
        if self.write_a.load(Ordering::SeqCst) {
            &self.buffer_b
        } else {
            &self.buffer_a
        }
    }

    pub fn flip(&self) {
        self.write_a
            .store(!self.write_a.load(Ordering::SeqCst), Ordering::SeqCst);
    }
}
