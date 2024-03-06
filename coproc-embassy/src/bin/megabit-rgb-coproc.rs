#![no_std]
#![no_main]

use core::cell::RefCell;
use defmt_rtt as _;
use embassy_nrf::{
    bind_interrupts,
    gpio::{Input, Level, Output, OutputDrive, Pin, Pull},
    peripherals,
    pwm::{Instance, Prescaler, SimplePwm},
    usb::{self, vbus_detect::HardwareVbusDetect},
    Peripheral,
};
use embassy_sync::{
    blocking_mutex::raw::{NoopRawMutex, ThreadModeRawMutex},
    channel::Channel,
    mutex::Mutex,
};
use megabit_coproc_embassy::{
    cobs_buffer::CobsBuffer,
    display::{
        rgb_matrix::DisplayCommandHandler, WaveshareDriver, COLUMNS, DISPLAY_CMD_QUEUE_SIZE, ROWS,
    },
    msg_router::{
        display_cmd_router::{DisplayCmdRouter, DisplayCommand},
        system_cmd_router::{SystemCmdRouter, SystemCommand},
        MessageRouter,
    },
    system_state::{Button, RgbLed, SystemStateManager, SYSTEM_CMD_QUEUE_SIZE},
    usb::{init_usb_device, split, Responder},
};
use panic_probe as _;
use static_cell::StaticCell;

bind_interrupts!(struct Irqs {
    POWER_CLOCK => usb::vbus_detect::InterruptHandler;
    USBD => usb::InterruptHandler<peripherals::USBD>;
});

type UsbDriver = usb::Driver<'static, peripherals::USBD, HardwareVbusDetect>;

const DEFAULT_MONO_COLOR: (u8, u8, u8) = (0xff, 00, 00);

const COBS_DECODE_BUFFER_SIZE: usize = 1024;
const COBS_ENCODE_BUFFER_SIZE: usize = 256;
static DISPLAY_CMD_CHANNEL: StaticCell<
    Channel<NoopRawMutex, DisplayCommand, DISPLAY_CMD_QUEUE_SIZE>,
> = StaticCell::new();
static SYSTEM_CMD_CHANNEL: StaticCell<Channel<NoopRawMutex, SystemCommand, SYSTEM_CMD_QUEUE_SIZE>> =
    StaticCell::new();
static COBS_DECODE_BUFFER: StaticCell<[u8; COBS_DECODE_BUFFER_SIZE]> = StaticCell::new();
static COBS_ENCODE_BUFFER: StaticCell<[u8; COBS_ENCODE_BUFFER_SIZE]> = StaticCell::new();
static MESSAGE_BUFFER: StaticCell<[u8; COBS_DECODE_BUFFER_SIZE]> = StaticCell::new();
static USB_RESPONDER: StaticCell<Responder<UsbDriver, COBS_ENCODE_BUFFER_SIZE>> = StaticCell::new();
static PIXEL_BUFFER_HANDLE: StaticCell<Mutex<ThreadModeRawMutex, RefCell<[u16; ROWS * COLUMNS]>>> =
    StaticCell::new();

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
    let display_cmd_router = DisplayCmdRouter::new(display_cmd_channel.sender(), true);
    let system_cmd_channel = SYSTEM_CMD_CHANNEL.init(Channel::new());
    let system_cmd_router = SystemCmdRouter::new(system_cmd_channel.sender());

    let router = MessageRouter::new(
        receiver,
        MESSAGE_BUFFER.init_with(|| [0; COBS_DECODE_BUFFER_SIZE]),
        CobsBuffer::new(COBS_DECODE_BUFFER.init_with(|| [0; COBS_DECODE_BUFFER_SIZE])),
        responder,
        display_cmd_router,
        system_cmd_router,
    );

    let mut debug_pin = Output::new(nrf_peripherals.P0_27, Level::Low, OutputDrive::Standard);

    let pixel_buffer =
        PIXEL_BUFFER_HANDLE.init_with(|| Mutex::new(RefCell::new([0u16; ROWS * COLUMNS])));

    let r1 = Output::new(nrf_peripherals.P0_04, Level::Low, OutputDrive::Standard);
    let g1 = Output::new(nrf_peripherals.P0_05, Level::Low, OutputDrive::Standard);
    let b1 = Output::new(nrf_peripherals.P0_30, Level::Low, OutputDrive::Standard);
    let r2 = Output::new(nrf_peripherals.P0_29, Level::Low, OutputDrive::Standard);
    let g2 = Output::new(nrf_peripherals.P0_31, Level::Low, OutputDrive::Standard);
    let b2 = Output::new(nrf_peripherals.P0_02, Level::Low, OutputDrive::Standard);
    let a = Output::new(nrf_peripherals.P0_13, Level::Low, OutputDrive::Standard);
    let b = Output::new(nrf_peripherals.P1_00, Level::Low, OutputDrive::Standard);
    let c = Output::new(nrf_peripherals.P1_01, Level::Low, OutputDrive::Standard);
    let d = Output::new(nrf_peripherals.P1_02, Level::Low, OutputDrive::Standard);
    let clk = Output::new(nrf_peripherals.P0_21, Level::Low, OutputDrive::Standard);
    let lat = Output::new(nrf_peripherals.P0_23, Level::Low, OutputDrive::Standard);
    let oe = Output::new(nrf_peripherals.P1_14, Level::High, OutputDrive::Standard);
    let driver_pins = (r1, g1, b1, r2, g2, b2, a, b, c, d, clk, lat, oe);
    let mut waveshare = WaveshareDriver::new(driver_pins, pixel_buffer);
    let driver_handle = waveshare.handle();

    let led_pin = Output::new(nrf_peripherals.P1_11, Level::Low, OutputDrive::Standard);
    let button_pin = UserButton::new(nrf_peripherals.P1_12);
    let pwm = NanoRgbLed::new(
        nrf_peripherals.PWM0,
        nrf_peripherals.P0_24,
        nrf_peripherals.P0_06,
        nrf_peripherals.P0_16,
    );

    let mut display_cmd_handler = DisplayCommandHandler::new(
        responder,
        display_cmd_channel.receiver(),
        DEFAULT_MONO_COLOR,
        driver_handle,
    );
    let system_state_mgr = SystemStateManager::new(
        system_cmd_channel.receiver(),
        responder,
        pwm,
        led_pin,
        button_pin,
    );

    // todo: See multiprio example once driver is introduced
    spawner.spawn(usb_driver_task(usb)).unwrap();
    spawner.spawn(msg_handler_task(router)).unwrap();

    //embassy_futures::join::join(display_cmd_handler.run(), system_state_mgr.run()).await;
    embassy_futures::join::join3(display_cmd_handler.run(), system_state_mgr.run(), async {
        loop {
            embassy_time::Timer::after_millis(5).await;
            waveshare.render(&mut debug_pin).await;
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

struct NanoRgbLed<T: Instance> {
    pwm: SimplePwm<'static, T>,
}

impl<T: Instance> NanoRgbLed<T> {
    const MAX_DUTY_CYCLE: u16 = 0x7fff;
    const RED_CHANNEL: usize = 0;
    const GREEN_CHANNEL: usize = 1;
    const BLUE_CHANNEL: usize = 2;

    pub fn new(
        pwm: impl Peripheral<P = T> + 'static,
        red_pin: impl Peripheral<P = impl Pin> + 'static,
        green_pin: impl Peripheral<P = impl Pin> + 'static,
        blue_pin: impl Peripheral<P = impl Pin> + 'static,
    ) -> Self {
        let mut pwm = SimplePwm::new_3ch(pwm, red_pin, green_pin, blue_pin);
        pwm.set_prescaler(Prescaler::Div1);
        pwm.set_max_duty(Self::MAX_DUTY_CYCLE);
        pwm.set_duty(Self::RED_CHANNEL, 0);
        pwm.set_duty(Self::GREEN_CHANNEL, 0);
        pwm.set_duty(Self::BLUE_CHANNEL, 0);
        Self { pwm }
    }
}

impl<T: Instance> RgbLed for NanoRgbLed<T> {
    fn set_state(&mut self, r: u8, g: u8, b: u8) {
        self.pwm.set_duty(Self::RED_CHANNEL, (r as u16) << 7);
        self.pwm.set_duty(Self::GREEN_CHANNEL, (g as u16) << 7);
        self.pwm.set_duty(Self::BLUE_CHANNEL, (b as u16) << 7);
    }

    fn off(&mut self) {
        self.set_state(0, 0, 0)
    }
}

struct UserButton<T: Pin> {
    button_input: Input<'static, T>,
}

impl<T: Pin> UserButton<T> {
    pub fn new(pin: impl Peripheral<P = T> + 'static) -> Self {
        Self {
            button_input: Input::new(pin, Pull::Down),
        }
    }
}

impl<T: Pin> Button for UserButton<T> {
    fn wait_for_press(&mut self) -> impl core::future::Future<Output = ()> {
        self.button_input.wait_for_rising_edge()
    }

    fn wait_for_release(&mut self) -> impl core::future::Future<Output = ()> {
        self.button_input.wait_for_falling_edge()
    }
}
