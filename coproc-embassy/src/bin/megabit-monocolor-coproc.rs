#![no_std]
#![no_main]

use defmt_rtt as _;
use embassy_nrf::{
    bind_interrupts,
    gpio::{Level, Output, OutputDrive},
    peripherals::{self, P0_21, P0_27, SPI3},
    spim::{self, Spim},
    usb::{self, vbus_detect::HardwareVbusDetect},
};
use embassy_sync::{blocking_mutex::raw::NoopRawMutex, channel::Channel};
use embassy_time::Timer;
use megabit_coproc_embassy::{
    cobs_buffer::CobsBuffer,
    display::{self, dot_matrix::DisplayCommandHandler},
    msg_router::{
        display_cmd_router::{DisplayCmdRouter, DisplayCommand},
        MessageRouter,
    },
    usb::{init_usb_device, split, Responder},
};
use panic_probe as _;
use static_cell::StaticCell;

bind_interrupts!(struct Irqs {
    POWER_CLOCK => usb::vbus_detect::InterruptHandler;
    SPIM3 => spim::InterruptHandler<peripherals::SPI3>;
    USBD => usb::InterruptHandler<peripherals::USBD>;
});

type UsbDriver = usb::Driver<'static, peripherals::USBD, HardwareVbusDetect>;
type DotMatrix =
    display::DotMatrix<Spim<'static, SPI3>, Output<'static, P0_27>, Output<'static, P0_21>>;

static DISPLAY_CMD_CHANNEL: StaticCell<Channel<NoopRawMutex, DisplayCommand, 1>> =
    StaticCell::new();
static COBS_DECODE_BUFFER: StaticCell<[u8; 1024]> = StaticCell::new();
static COBS_ENCODE_BUFFER: StaticCell<[u8; 256]> = StaticCell::new();
static USB_RESPONDER: StaticCell<Responder<UsbDriver, 256>> = StaticCell::new();

#[embassy_executor::main]
async fn main(spawner: embassy_executor::Spawner) {
    let nrf_peripherals = embassy_nrf::init(Default::default());
    let mut led = Output::new(nrf_peripherals.P0_06, Level::Low, OutputDrive::Standard);
    let usb_driver = usb::Driver::new(nrf_peripherals.USBD, Irqs, HardwareVbusDetect::new(Irqs));
    let (usb, cdc_acm) = init_usb_device(usb_driver);
    let (responder, receiver) = split(cdc_acm, COBS_ENCODE_BUFFER.init_with(|| [0; 256]));
    let responder = USB_RESPONDER.init(responder);

    let display_cmd_channel = DISPLAY_CMD_CHANNEL.init(Channel::new());
    let display_cmd_router = DisplayCmdRouter::new(display_cmd_channel.sender());

    let router = MessageRouter::new(
        receiver,
        CobsBuffer::new(COBS_DECODE_BUFFER.init_with(|| [0; 1024])),
        responder,
        display_cmd_router,
    );

    spawner.spawn(usb_driver_task(usb)).unwrap();
    spawner.spawn(msg_handler_task(router)).unwrap();

    let mut config = spim::Config::default();
    config.frequency = spim::Frequency::M4;
    config.mode = spim::MODE_0;

    let spim = spim::Spim::new_txonly(
        nrf_peripherals.SPI3,
        Irqs,
        nrf_peripherals.P0_13,
        nrf_peripherals.P1_01,
        config,
    );
    let ncs_0 = Output::new(nrf_peripherals.P0_27, Level::High, OutputDrive::Standard);
    let ncs_1 = Output::new(nrf_peripherals.P0_21, Level::High, OutputDrive::Standard);

    let dot_matrix = DotMatrix::new(spim, ncs_0, ncs_1).await.unwrap();

    let mut display_cmd_handler =
        DisplayCommandHandler::new(dot_matrix, responder, display_cmd_channel.receiver());

    loop {
        display_cmd_handler.try_handle_cmd().await;
        Timer::after_millis(400).await;
        if led.is_set_high() {
            led.set_low();
        } else {
            led.set_high();
        }
    }
}

#[embassy_executor::task]
pub async fn msg_handler_task(router: MessageRouter<UsbDriver, Responder<UsbDriver, 256>, 1024>) {
    router.run().await
}

#[embassy_executor::task]
pub async fn usb_driver_task(mut device: embassy_usb::UsbDevice<'static, UsbDriver>) {
    device.run().await;
}
