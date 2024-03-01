use crate::{msg_router::cmds, usb::UsbResponder};
use embassy_time::Timer;
use embassy_usb_driver::EndpointError;
use embedded_hal::digital::StatefulOutputPin;

mod button;
pub use button::Button;
mod rgb_led;
pub use rgb_led::RgbLed;

pub struct SystemStateManager<
    R: UsbResponder + 'static,
    RGB: RgbLed,
    DEBUG: StatefulOutputPin,
    BTN: Button,
> {
    responder: &'static R,
    rgb_led: RGB,
    debug_led: DEBUG,
    button: BTN,
}

impl<R: UsbResponder + 'static, RGB: RgbLed, DEBUG: StatefulOutputPin, BTN: Button>
    SystemStateManager<R, RGB, DEBUG, BTN>
{
    pub fn new(responder: &'static R, rgb_led: RGB, debug_led: DEBUG, button: BTN) -> Self {
        Self {
            responder,
            rgb_led,
            debug_led,
            button,
        }
    }

    pub async fn run(mut self) {
        let mut usb_error = false;
        loop {
            self.button.wait_for_release().await;
            Timer::after_millis(50).await;
            if self.report_button_press().await.is_err() {
                usb_error = true;
            }
            if self.debug_led.is_set_high().unwrap() {
                self.debug_led.set_low().unwrap();
            } else {
                self.debug_led.set_high().unwrap();
            }
            if usb_error {
                self.rgb_led.set_state(0xff, 0, 0);
            }
        }
    }

    async fn report_button_press(&mut self) -> Result<(), EndpointError> {
        self.responder
            .send(&[
                cmds::report_button_press::MAJOR,
                cmds::report_button_press::MINOR,
            ])
            .await
    }
}
