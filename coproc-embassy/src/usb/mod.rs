use embassy_usb::{
    class::cdc_acm::{self, CdcAcmClass},
    driver::EndpointError,
};
use static_cell::StaticCell;

mod nrf;
pub use nrf::{usb_driver_task, UsbDriver};

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
        &mut DEVICE_DESC.init([0; 256])[..],
        &mut CONFIG_DESC.init([0; 256])[..],
        &mut BOS_DESC.init([0; 256])[..],
        &mut MSOS_DESC.init([0; 128])[..],
        &mut CONTROL_BUF.init([0; 128])[..],
    );
    let class = CdcAcmClass::new(&mut builder, state, 64);
    let usb = builder.build();
    (usb, class)
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
