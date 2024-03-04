// Driver for a 64x32 RGB HUB-75 display by Waveshare.
// Based on:
// * https://github.com/david-sawatzke/hub75-rs/blob/30f2aa62279669f9cf97149f450b2b0f7bdbab8b/src/lib.rs#L261
// * https://learn.adafruit.com/adafruit-protomatter-rgb-matrix-library/arduino-library

use super::{COLUMNS, ROWS};
use core::{cell::RefCell, convert::Infallible};
use embassy_sync::blocking_mutex::raw::ThreadModeRawMutex;
use embassy_sync::mutex::Mutex;
use embassy_time::Timer;
use embedded_hal::digital::OutputPin;

pub trait Hub75Display {}

pub trait DriverPins {
    // Color data pins
    type R1: OutputPin<Error = Infallible>;
    type G1: OutputPin<Error = Infallible>;
    type B1: OutputPin<Error = Infallible>;
    type R2: OutputPin<Error = Infallible>;
    type G2: OutputPin<Error = Infallible>;
    type B2: OutputPin<Error = Infallible>;
    // Row address selection
    type A: OutputPin<Error = Infallible>;
    type B: OutputPin<Error = Infallible>;
    type C: OutputPin<Error = Infallible>;
    type D: OutputPin<Error = Infallible>;
    // Control pins
    type CLK: OutputPin<Error = Infallible>;
    type LAT: OutputPin<Error = Infallible>;
    type OE: OutputPin<Error = Infallible>;

    fn r1(&mut self) -> &mut Self::R1;
    fn g1(&mut self) -> &mut Self::G1;
    fn b1(&mut self) -> &mut Self::B1;
    fn r2(&mut self) -> &mut Self::R2;
    fn g2(&mut self) -> &mut Self::G2;
    fn b2(&mut self) -> &mut Self::B2;

    fn a(&mut self) -> &mut Self::A;
    fn b(&mut self) -> &mut Self::B;
    fn c(&mut self) -> &mut Self::C;
    fn d(&mut self) -> &mut Self::D;

    fn clk(&mut self) -> &mut Self::CLK;
    fn lat(&mut self) -> &mut Self::LAT;
    fn oe(&mut self) -> &mut Self::OE;
}

impl<
        R1: OutputPin<Error = Infallible>,
        G1: OutputPin<Error = Infallible>,
        B1: OutputPin<Error = Infallible>,
        R2: OutputPin<Error = Infallible>,
        G2: OutputPin<Error = Infallible>,
        B2: OutputPin<Error = Infallible>,
        A: OutputPin<Error = Infallible>,
        B: OutputPin<Error = Infallible>,
        C: OutputPin<Error = Infallible>,
        D: OutputPin<Error = Infallible>,
        CLK: OutputPin<Error = Infallible>,
        LAT: OutputPin<Error = Infallible>,
        OE: OutputPin<Error = Infallible>,
    > DriverPins for (R1, G1, B1, R2, G2, B2, A, B, C, D, CLK, LAT, OE)
{
    type R1 = R1;
    type G1 = G1;
    type B1 = B1;
    type R2 = R2;
    type G2 = G2;
    type B2 = B2;
    type A = A;
    type B = B;
    type C = C;
    type D = D;
    type CLK = CLK;
    type LAT = LAT;
    type OE = OE;

    fn r1(&mut self) -> &mut R1 {
        &mut self.0
    }

    fn g1(&mut self) -> &mut G1 {
        &mut self.1
    }

    fn b1(&mut self) -> &mut B1 {
        &mut self.2
    }

    fn r2(&mut self) -> &mut R2 {
        &mut self.3
    }

    fn g2(&mut self) -> &mut G2 {
        &mut self.4
    }

    fn b2(&mut self) -> &mut B2 {
        &mut self.5
    }

    fn a(&mut self) -> &mut A {
        &mut self.6
    }

    fn b(&mut self) -> &mut Self::B {
        &mut self.7
    }

    fn c(&mut self) -> &mut Self::C {
        &mut self.8
    }

    fn d(&mut self) -> &mut Self::D {
        &mut self.9
    }

    fn clk(&mut self) -> &mut Self::CLK {
        &mut self.10
    }

    fn lat(&mut self) -> &mut Self::LAT {
        &mut self.11
    }

    fn oe(&mut self) -> &mut Self::OE {
        &mut self.12
    }
}

type PixelBuffer = [u16; ROWS * COLUMNS];

pub struct WaveshareDriver<PINS: DriverPins> {
    pins: PINS,
    pixel_data: &'static Mutex<ThreadModeRawMutex, RefCell<PixelBuffer>>,
}

impl<PINS: DriverPins> WaveshareDriver<PINS> {
    pub fn new(
        pins: PINS,
        pixel_data: &'static Mutex<ThreadModeRawMutex, RefCell<PixelBuffer>>,
    ) -> Self {
        Self { pins, pixel_data }
    }

    pub fn handle(&self) -> DriverHandle {
        DriverHandle::new(self.pixel_data)
    }

    pub async fn run(&mut self, mut timing_pin: impl OutputPin) {
        loop {
            Timer::after_millis(5).await;
            self.render(&mut timing_pin).await;
        }
    }

    pub async fn render(&mut self, timing_pin: &mut impl OutputPin) {
        let pixel_data = self.pixel_data.lock().await;
        let pixel_data = pixel_data.borrow();
        for pwm_step in 0..(1u8 << 5) {
            for row in 0..(ROWS / 2) {
                for idx in (row * COLUMNS)..((row + 1) * COLUMNS) {
                    timing_pin.set_low().unwrap();
                    let idx2 = idx + pixel_data.len() / 2;
                    let (r1, g1, b1) = channels(pixel_data[idx]);
                    let (r2, g2, b2) = channels(pixel_data[idx2]);
                    self.pins.r1().set_state((r1 > pwm_step).into()).unwrap();
                    self.pins.g1().set_state((g1 > pwm_step).into()).unwrap();
                    self.pins.b1().set_state((b1 > pwm_step).into()).unwrap();
                    self.pins.r2().set_state((r2 > pwm_step).into()).unwrap();
                    self.pins.g2().set_state((g2 > pwm_step).into()).unwrap();
                    self.pins.b2().set_state((b2 > pwm_step).into()).unwrap();

                    self.pins.clk().set_high().unwrap();
                    self.pins.clk().set_low().unwrap();
                    timing_pin.set_high().unwrap();
                }

                self.pins.oe().set_high().unwrap();
                Timer::after_micros(2).await;
                self.pins.lat().set_low().unwrap();
                Timer::after_micros(2).await;
                self.pins.lat().set_high().unwrap();

                // set the address
                let addr = row as u8;
                self.pins
                    .a()
                    .set_state(((addr & (1 << 0)) != 0).into())
                    .unwrap();
                self.pins
                    .b()
                    .set_state(((addr & (1 << 1)) != 0).into())
                    .unwrap();
                self.pins
                    .c()
                    .set_state(((addr & (1 << 2)) != 0).into())
                    .unwrap();
                self.pins
                    .d()
                    .set_state(((addr & (1 << 3)) != 0).into())
                    .unwrap();
                Timer::after_micros(2).await;
                self.pins.oe().set_low().unwrap();
            }
        }

        self.pins.oe().set_high().unwrap();
    }
}

fn channels(pixel_color: u16) -> (u8, u8, u8) {
    let r = (pixel_color & 0b11111_00000_00000) >> 10;
    let g = (pixel_color & 0b00000_11111_00000) >> 5;
    let b = pixel_color & 0b00000_00000_11111;
    (r as u8, g as u8, b as u8)
}

pub struct DriverHandle {
    pixel_data: &'static Mutex<ThreadModeRawMutex, RefCell<PixelBuffer>>,
}

impl DriverHandle {
    pub fn new(pixel_data: &'static Mutex<ThreadModeRawMutex, RefCell<PixelBuffer>>) -> Self {
        Self { pixel_data }
    }

    pub async fn set_cell(&mut self, row: u8, col: u8, value: u16) {
        let idx = col as usize + (row as usize * COLUMNS);
        let pixel_data = self.pixel_data.lock().await;
        let mut pixel_data = pixel_data.borrow_mut();
        pixel_data[idx] = value;
    }
}
