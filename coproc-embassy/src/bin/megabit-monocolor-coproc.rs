#![no_std]
#![no_main]

use defmt_rtt as _;
use embassy_nrf::{
    bind_interrupts,
    gpio::{Level, Output, OutputDrive},
    peripherals, spim,
    usb::{self, vbus_detect::HardwareVbusDetect},
};
use embassy_sync::{blocking_mutex::raw::NoopRawMutex, channel::Channel};
use megabit_coproc_embassy::{
    cobs_buffer::CobsBuffer,
    dot_matrix::DotMatrix,
    msg_router::msg_handler_task,
    usb::{init_usb_device, usb_driver_task},
};
use panic_probe as _;
use static_cell::StaticCell;

bind_interrupts!(struct Irqs {
    POWER_CLOCK => usb::vbus_detect::InterruptHandler;
    SPIM3 => spim::InterruptHandler<peripherals::SPI3>;
    USBD => usb::InterruptHandler<peripherals::USBD>;
});

static SET_LED_CHANNEL: StaticCell<Channel<NoopRawMutex, (u8, u8, bool), 1>> = StaticCell::new();
static ROW_UPDATE_CHANNEL: StaticCell<Channel<NoopRawMutex, (u8, [u8; 4]), 4>> = StaticCell::new();

#[embassy_executor::main]
async fn main(spawner: embassy_executor::Spawner) {
    let nrf_peripherals = embassy_nrf::init(Default::default());
    let mut led = Output::new(nrf_peripherals.P0_06, Level::Low, OutputDrive::Standard);
    let usb_driver = usb::Driver::new(nrf_peripherals.USBD, Irqs, HardwareVbusDetect::new(Irqs));
    let (usb, cdc_acm) = init_usb_device(usb_driver);

    static COBS_DECODE_BUFFER: StaticCell<[u8; 1024]> = StaticCell::new();
    static COBS_ENCODE_BUFFER: StaticCell<[u8; 256]> = StaticCell::new();
    let cobs_decoder = CobsBuffer::new(COBS_DECODE_BUFFER.init([0; 1024]));

    let channel = SET_LED_CHANNEL.init(Channel::new());
    let led_sender = channel.sender();
    let row_update_channel = ROW_UPDATE_CHANNEL.init(Channel::new());

    spawner.spawn(usb_driver_task(usb)).unwrap();
    spawner
        .spawn(msg_handler_task(
            cdc_acm,
            cobs_decoder,
            COBS_ENCODE_BUFFER.init([0; 256]),
            led_sender,
            row_update_channel.sender(),
        ))
        .unwrap();

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

    let mut dot_matrix = DotMatrix::new(spim, ncs_0, ncs_1).await.unwrap();

    let rx = row_update_channel.receiver();
    loop {
        let (row, row_data) = rx.receive().await;
        dot_matrix.update_row(row as usize, row_data).await.unwrap();
        if led.is_set_high() {
            led.set_low();
        } else {
            led.set_high();
        }
    }
}
