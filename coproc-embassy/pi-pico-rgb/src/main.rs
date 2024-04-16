#![no_std]
#![no_main]

use core::cell::RefCell;
use defmt::unwrap;
use defmt_rtt as _;
use embassy_executor::Executor;
use embassy_rp::{
    bind_interrupts,
    gpio::{Input, Level, Output, Pin, Pull},
    multicore::{spawn_core1, Stack},
    peripherals::{
        self, PIN_0, PIN_1, PIN_10, PIN_11, PIN_12, PIN_17, PIN_19, PIN_2, PIN_21, PIN_25, PIN_3,
        PIN_4, PIN_5, PIN_6, PIN_7, PIN_8, PIN_9, PWM_CH0, PWM_CH1, PWM_CH2, USB,
    },
    pwm::{self, Pwm},
    usb::{self, InterruptHandler},
    Peripheral,
};
use embassy_sync::{
    blocking_mutex::raw::{CriticalSectionRawMutex, NoopRawMutex},
    channel::Channel,
    mutex::Mutex,
};
use megabit_coproc_common::{
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

type UsbDriver = usb::Driver<'static, peripherals::USB>;

const DEFAULT_MONO_COLOR: (u8, u8, u8) = (0xff, 00, 00);

const COBS_DECODE_BUFFER_SIZE: usize = 1024;
const COBS_ENCODE_BUFFER_SIZE: usize = 256;

static mut CORE1_STACK: Stack<4096> = Stack::new();
static EXECUTOR0: StaticCell<Executor> = StaticCell::new();
static EXECUTOR1: StaticCell<Executor> = StaticCell::new();

static COBS_DECODE_BUFFER: StaticCell<[u8; COBS_DECODE_BUFFER_SIZE]> = StaticCell::new();
static COBS_ENCODE_BUFFER: StaticCell<[u8; COBS_ENCODE_BUFFER_SIZE]> = StaticCell::new();
static MESSAGE_BUFFER: StaticCell<[u8; COBS_DECODE_BUFFER_SIZE]> = StaticCell::new();
static USB_RESPONDER: StaticCell<Responder<UsbDriver, COBS_ENCODE_BUFFER_SIZE>> = StaticCell::new();
static ROW_DATA_BUFFER: StaticCell<Mutex<NoopRawMutex, [u16; COLUMNS]>> = StaticCell::new();
static DISPLAY_CMD_CHANNEL: StaticCell<
    Channel<NoopRawMutex, DisplayCommand, DISPLAY_CMD_QUEUE_SIZE>,
> = StaticCell::new();
static SYSTEM_CMD_CHANNEL: StaticCell<Channel<NoopRawMutex, SystemCommand, SYSTEM_CMD_QUEUE_SIZE>> =
    StaticCell::new();
static PIXEL_BUFFER_HANDLE: StaticCell<
    Mutex<CriticalSectionRawMutex, RefCell<[u16; ROWS * COLUMNS]>>,
> = StaticCell::new();

bind_interrupts!(struct Irqs {
    USBCTRL_IRQ => InterruptHandler<USB>;
});

#[cortex_m_rt::entry]
fn main() -> ! {
    let peripherals = embassy_rp::init(Default::default());

    // Initialize the USB driver and handles
    let usb_driver = usb::Driver::new(peripherals.USB, Irqs);
    let (usb, cdc_acm) = init_usb_device(usb_driver);
    let (responder, receiver) = split(
        cdc_acm,
        COBS_ENCODE_BUFFER.init_with(|| [0; COBS_ENCODE_BUFFER_SIZE]),
    );
    let responder = USB_RESPONDER.init(responder);

    let row_data_buffer = ROW_DATA_BUFFER.init_with(|| Mutex::new([0u16; COLUMNS]));

    let display_cmd_channel = DISPLAY_CMD_CHANNEL.init(Channel::new());
    let display_cmd_router =
        DisplayCmdRouter::new(display_cmd_channel.sender(), row_data_buffer, true);
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

    let pixel_buffer =
        PIXEL_BUFFER_HANDLE.init_with(|| Mutex::new(RefCell::new([0u16; ROWS * COLUMNS])));

    let r1 = Output::new(peripherals.PIN_0, Level::Low);
    let g1 = Output::new(peripherals.PIN_1, Level::Low);
    let r2 = Output::new(peripherals.PIN_2, Level::Low);
    let g2 = Output::new(peripherals.PIN_3, Level::Low);
    let a = Output::new(peripherals.PIN_4, Level::Low);
    let c = Output::new(peripherals.PIN_5, Level::Low);
    let clk = Output::new(peripherals.PIN_6, Level::Low);
    let oe = Output::new(peripherals.PIN_7, Level::High);
    let b1 = Output::new(peripherals.PIN_8, Level::Low);
    let b2 = Output::new(peripherals.PIN_9, Level::Low);
    let b = Output::new(peripherals.PIN_10, Level::Low);
    let d = Output::new(peripherals.PIN_11, Level::Low);
    let lat = Output::new(peripherals.PIN_12, Level::Low);

    let driver_pins = (r1, g1, b1, r2, g2, b2, a, b, c, d, clk, lat, oe);
    let waveshare = WaveshareDriver::new(driver_pins, pixel_buffer);
    let driver_handle = waveshare.handle();

    let mut config = pwm::Config::default();
    config.top = PWM_MAX_COUNT;
    let pwm_r = pwm::Pwm::new_output_a(peripherals.PWM_CH0, peripherals.PIN_16, config.clone());
    let pwm_g = pwm::Pwm::new_output_a(peripherals.PWM_CH1, peripherals.PIN_18, config.clone());
    let pwm_b = pwm::Pwm::new_output_a(peripherals.PWM_CH2, peripherals.PIN_20, config);
    let rgb_led = NanoRgbLed::new(pwm_r, pwm_g, pwm_b);
    let led_pin = Output::new(peripherals.PIN_25, Level::Low);
    let button_pin = UserButton::new(peripherals.PIN_17);

    let debug_pin = Output::new(peripherals.PIN_19, Level::Low);
    let debug_2 = Output::new(peripherals.PIN_21, Level::Low);

    let display_cmd_handler = DisplayCommandHandler::new(
        responder,
        display_cmd_channel.receiver(),
        DEFAULT_MONO_COLOR,
        driver_handle,
        row_data_buffer,
        debug_pin,
    );
    let system_state_mgr = SystemStateManager::new(
        system_cmd_channel.receiver(),
        responder,
        rgb_led,
        led_pin,
        button_pin,
    );

    spawn_core1(
        peripherals.CORE1,
        unsafe { &mut *core::ptr::addr_of_mut!(CORE1_STACK) },
        move || {
            let executor1 = EXECUTOR1.init(Executor::new());
            executor1.run(|spawner| unwrap!(spawner.spawn(core1_task(waveshare, debug_2))));
        },
    );

    let executor0 = EXECUTOR0.init(Executor::new());
    executor0.run(|spawner| {
        unwrap!(spawner.spawn(core0_task(
            display_cmd_handler,
            system_state_mgr,
            usb,
            router
        )))
    });
}

#[embassy_executor::task]
async fn core0_task(
    mut display_cmd_handler: DisplayCommandHandler<
        Responder<UsbDriver, COBS_ENCODE_BUFFER_SIZE>,
        Output<'static, PIN_19>,
        CriticalSectionRawMutex,
    >,
    system_state_mgr: SystemStateManager<
        Responder<UsbDriver, COBS_ENCODE_BUFFER_SIZE>,
        NanoRgbLed<PWM_CH0, PWM_CH1, PWM_CH2>,
        Output<'static, PIN_25>,
        UserButton<PIN_17>,
    >,
    mut usb: embassy_usb::UsbDevice<'static, UsbDriver>,
    router: MessageRouter<
        UsbDriver,
        Responder<UsbDriver, COBS_ENCODE_BUFFER_SIZE>,
        COBS_DECODE_BUFFER_SIZE,
    >,
) {
    embassy_futures::join::join4(
        display_cmd_handler.run(),
        system_state_mgr.run(),
        usb.run(),
        router.run(),
    )
    .await;
}

// todo: That generic parameter on Output for the PIN is going away at some point
#[embassy_executor::task]
async fn core1_task(
    mut waveshare: WaveshareDriver<
        (
            Output<'static, PIN_0>,
            Output<'static, PIN_1>,
            Output<'static, PIN_8>,
            Output<'static, PIN_2>,
            Output<'static, PIN_3>,
            Output<'static, PIN_9>,
            Output<'static, PIN_4>,
            Output<'static, PIN_10>,
            Output<'static, PIN_5>,
            Output<'static, PIN_11>,
            Output<'static, PIN_6>,
            Output<'static, PIN_12>,
            Output<'static, PIN_7>,
        ),
        CriticalSectionRawMutex,
    >,
    mut debug_pin: Output<'static, PIN_21>,
) {
    loop {
        embassy_time::Timer::after_micros(500).await;
        waveshare.render(&mut debug_pin).await;
    }
}

const PWM_MAX_COUNT: u16 = 0x7fff;

struct NanoRgbLed<C1: pwm::Channel, C2: pwm::Channel, C3: pwm::Channel> {
    pwm_r: Pwm<'static, C1>,
    pwm_g: Pwm<'static, C2>,
    pwm_b: Pwm<'static, C3>,
}

impl<C1: pwm::Channel, C2: pwm::Channel, C3: pwm::Channel> NanoRgbLed<C1, C2, C3> {
    pub fn new(pwm_r: Pwm<'static, C1>, pwm_g: Pwm<'static, C2>, pwm_b: Pwm<'static, C3>) -> Self {
        Self {
            pwm_r,
            pwm_g,
            pwm_b,
        }
    }
}

impl<C1: pwm::Channel, C2: pwm::Channel, C3: pwm::Channel> RgbLed for NanoRgbLed<C1, C2, C3> {
    fn set_state(&mut self, r: u8, g: u8, b: u8) {
        let mut config = pwm::Config::default();
        config.top = PWM_MAX_COUNT;
        config.compare_a = (r as u16) << 7;
        self.pwm_r.set_config(&config);
        config.compare_a = (g as u16) << 7;
        self.pwm_g.set_config(&config);
        config.compare_a = (b as u16) << 7;
        self.pwm_b.set_config(&config);
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
