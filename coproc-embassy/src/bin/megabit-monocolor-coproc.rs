#![no_std]
#![no_main]

use defmt_rtt as _;
use embassy_nrf::{
    bind_interrupts,
    gpio::{Input, Level, Output, OutputDrive, Pull},
    peripherals,
    pwm::{Prescaler, SimplePwm},
    spim,
    usb::{self, vbus_detect::HardwareVbusDetect},
};
use embassy_sync::{blocking_mutex::raw::NoopRawMutex, channel::Channel};
use embassy_time::Timer;
use megabit_coproc_embassy::{
    cobs_buffer::CobsBuffer,
    display::{dot_matrix::DisplayCommandHandler, DotMatrix, DISPLAY_CMD_QUEUE_SIZE},
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

const COBS_DECODE_BUFFER_SIZE: usize = 1024;
const COBS_ENCODE_BUFFER_SIZE: usize = 256;
static DISPLAY_CMD_CHANNEL: StaticCell<
    Channel<NoopRawMutex, DisplayCommand, DISPLAY_CMD_QUEUE_SIZE>,
> = StaticCell::new();
static COBS_DECODE_BUFFER: StaticCell<[u8; COBS_DECODE_BUFFER_SIZE]> = StaticCell::new();
static COBS_ENCODE_BUFFER: StaticCell<[u8; COBS_ENCODE_BUFFER_SIZE]> = StaticCell::new();
static USB_RESPONDER: StaticCell<Responder<UsbDriver, COBS_ENCODE_BUFFER_SIZE>> = StaticCell::new();

#[embassy_executor::main]
async fn main(spawner: embassy_executor::Spawner) {
    let nrf_peripherals = embassy_nrf::init(Default::default());
    let usb_driver = usb::Driver::new(nrf_peripherals.USBD, Irqs, HardwareVbusDetect::new(Irqs));
    let (usb, cdc_acm) = init_usb_device(usb_driver);
    let (responder, receiver) = split(
        cdc_acm,
        COBS_ENCODE_BUFFER.init_with(|| [0; COBS_ENCODE_BUFFER_SIZE]),
    );
    let responder = USB_RESPONDER.init(responder);

    let display_cmd_channel = DISPLAY_CMD_CHANNEL.init(Channel::new());
    let display_cmd_router = DisplayCmdRouter::new(display_cmd_channel.sender(), false);

    let router = MessageRouter::new(
        receiver,
        CobsBuffer::new(COBS_DECODE_BUFFER.init_with(|| [0; COBS_DECODE_BUFFER_SIZE])),
        responder,
        display_cmd_router,
    );

    let mut led_pin = Output::new(nrf_peripherals.P1_11, Level::Low, OutputDrive::Standard);
    let mut button_pin = Input::new(nrf_peripherals.P1_12, Pull::Down);
    let mut pwm = SimplePwm::new_3ch(
        nrf_peripherals.PWM0,
        nrf_peripherals.P0_24,
        nrf_peripherals.P0_06,
        nrf_peripherals.P0_16,
    );
    pwm.set_prescaler(Prescaler::Div1);
    pwm.set_max_duty(0x7fff);
    pwm.set_duty(0, 0);
    pwm.set_duty(1, 0);
    pwm.set_duty(2, 0);

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

    spawner.spawn(usb_driver_task(usb)).unwrap();
    spawner.spawn(msg_handler_task(router)).unwrap();
    let mut count = 0u8;

    embassy_futures::join::join(display_cmd_handler.run(), async {
        loop {
            button_pin.wait_for_falling_edge().await;
            Timer::after_millis(50).await;
            count = (count + 1) & 0b11;
            if led_pin.is_set_high() {
                led_pin.set_low();
            } else {
                led_pin.set_high();
            }
            if count == 0 {
                pwm.set_duty(0, 0);
                pwm.set_duty(1, 0);
                pwm.set_duty(2, 0);
            } else if count == 1 {
                pwm.set_duty(0, 0xff << 7);
                pwm.set_duty(1, 0);
                pwm.set_duty(2, 0);
            } else if count == 2 {
                pwm.set_duty(0, 0);
                pwm.set_duty(1, 0xff << 7);
                pwm.set_duty(2, 0);
            } else if count == 3 {
                pwm.set_duty(0, 0);
                pwm.set_duty(1, 0);
                pwm.set_duty(2, 0xff << 7);
            }
        }
    })
    .await;
}

#[embassy_executor::task]
pub async fn msg_handler_task(
    router: MessageRouter<
        UsbDriver,
        Responder<UsbDriver, COBS_ENCODE_BUFFER_SIZE>,
        COBS_DECODE_BUFFER_SIZE,
    >,
) {
    router.run().await
}

#[embassy_executor::task]
pub async fn usb_driver_task(mut device: embassy_usb::UsbDevice<'static, UsbDriver>) {
    device.run().await;
}
