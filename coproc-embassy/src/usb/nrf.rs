use embassy_nrf::{
    peripherals,
    usb::{self, vbus_detect::HardwareVbusDetect},
};

pub type UsbDriver = usb::Driver<'static, peripherals::USBD, HardwareVbusDetect>;

#[embassy_executor::task]
pub async fn usb_driver_task(mut device: embassy_usb::UsbDevice<'static, UsbDriver>) {
    device.run().await;
}
