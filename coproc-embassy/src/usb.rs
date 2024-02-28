use embassy_sync::{blocking_mutex::raw::NoopRawMutex, mutex::Mutex};
use embassy_usb::{
    class::cdc_acm::{self, CdcAcmClass, Receiver, Sender},
    driver::EndpointError,
};
use static_cell::StaticCell;

pub fn init_usb_device<T>(
    usb_driver: T,
) -> (embassy_usb::UsbDevice<'static, T>, CdcAcmClass<'static, T>)
where
    T: embassy_usb_driver::Driver<'static>,
{
    let mut config = embassy_usb::Config::new(0x16c0, 0x27de);
    config.manufacturer = Some("Snostorm Labs");
    config.product = Some("Megabit coproc");
    config.serial_number = Some("0123456789ABCDEF");
    config.max_power = 125;
    config.max_packet_size_0 = 64;
    config.device_class = 0xEF;
    config.device_sub_class = 0x02;
    config.device_protocol = 0x01;
    config.composite_with_iads = true;

    static STATE: StaticCell<cdc_acm::State> = StaticCell::new();
    let state = STATE.init(cdc_acm::State::new());

    static DEVICE_DESC: StaticCell<[u8; 256]> = StaticCell::new();
    static CONFIG_DESC: StaticCell<[u8; 256]> = StaticCell::new();
    static BOS_DESC: StaticCell<[u8; 256]> = StaticCell::new();
    static MSOS_DESC: StaticCell<[u8; 128]> = StaticCell::new();
    static CONTROL_BUF: StaticCell<[u8; 128]> = StaticCell::new();
    let mut builder = embassy_usb::Builder::new(
        usb_driver,
        config,
        &mut DEVICE_DESC.init_with(|| [0; 256])[..],
        &mut CONFIG_DESC.init_with(|| [0; 256])[..],
        &mut BOS_DESC.init_with(|| [0; 256])[..],
        &mut MSOS_DESC.init_with(|| [0; 128])[..],
        &mut CONTROL_BUF.init_with(|| [0; 128])[..],
    );
    let class = CdcAcmClass::new(&mut builder, state, 64);
    let usb = builder.build();
    (usb, class)
}

pub fn split<T, const N: usize>(
    class: CdcAcmClass<'static, T>,
    encoded_buffer: &'static mut [u8; N],
) -> (Responder<T, N>, Receiver<'static, T>)
where
    T: embassy_usb_driver::Driver<'static>,
{
    let (tx, rx) = class.split();
    (Responder::new(tx, encoded_buffer), rx)
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

    pub async fn send(&self, unencoded_buf: &[u8]) -> Result<(), EndpointError> {
        self.inner.lock().await.send(unencoded_buf).await
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

pub struct Disconnected {}

impl From<EndpointError> for Disconnected {
    fn from(val: EndpointError) -> Self {
        match val {
            EndpointError::BufferOverflow => panic!("Buffer overflow"),
            EndpointError::Disabled => Disconnected {},
        }
    }
}
