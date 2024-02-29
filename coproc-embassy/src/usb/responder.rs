use core::future::Future;
use embassy_sync::{blocking_mutex::raw::NoopRawMutex, mutex::Mutex};
use embassy_usb::{class::cdc_acm::Sender, driver::EndpointError};

pub trait UsbResponder {
    fn send(&self, unencoded_buf: &[u8]) -> impl Future<Output = Result<(), EndpointError>>;
}

pub struct Responder<T: embassy_usb_driver::Driver<'static>, const N: usize> {
    inner: Mutex<NoopRawMutex, ResponderInner<T, N>>,
}

impl<T: embassy_usb_driver::Driver<'static>, const N: usize> Responder<T, N> {
    pub fn new(tx: Sender<'static, T>, encoded_buffer: &'static mut [u8; N]) -> Self {
        Self {
            inner: Mutex::new(ResponderInner::new(tx, encoded_buffer)),
        }
    }
}

impl<T: embassy_usb_driver::Driver<'static>, const N: usize> UsbResponder for Responder<T, N> {
    fn send(&self, unencoded_buf: &[u8]) -> impl Future<Output = Result<(), EndpointError>> {
        async { self.inner.lock().await.send(unencoded_buf).await }
    }
}

struct ResponderInner<T: embassy_usb_driver::Driver<'static>, const N: usize> {
    tx: Sender<'static, T>,
    encoded_buffer: &'static mut [u8; N],
}

impl<T: embassy_usb_driver::Driver<'static>, const N: usize> ResponderInner<T, N> {
    fn new(tx: Sender<'static, T>, encoded_buffer: &'static mut [u8; N]) -> Self {
        Self { tx, encoded_buffer }
    }

    /// Takes a buffer of bytes, encodes them with COBS, adds a sentinel byte, and writes it out to the USB host.
    async fn send(&mut self, unencoded_buf: &[u8]) -> Result<(), EndpointError> {
        let encoded_bytes = cobs::encode(unencoded_buf, self.encoded_buffer);
        self.encoded_buffer[encoded_bytes] = 0x00;
        self.tx
            .write_packet(&self.encoded_buffer[..encoded_bytes + 1])
            .await
    }
}
