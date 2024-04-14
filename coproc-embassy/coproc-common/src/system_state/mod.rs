use core::sync::atomic::{AtomicBool, Ordering};

use crate::{
    msg_router::{
        cmds,
        system_cmd_router::{SetDebugLedState, SetRgbState, SystemCommand},
    },
    usb::UsbResponder,
};
use embassy_sync::{
    blocking_mutex::raw::NoopRawMutex,
    channel::{Receiver, Sender},
};
use embassy_time::Timer;
use embedded_hal::digital::StatefulOutputPin;

mod button;
pub use button::Button;
mod rgb_led;
pub use rgb_led::RgbLed;

pub const SYSTEM_CMD_QUEUE_SIZE: usize = 2;
pub type SystemCmdSender = Sender<'static, NoopRawMutex, SystemCommand, SYSTEM_CMD_QUEUE_SIZE>;
pub type SystemCmdReceiver = Receiver<'static, NoopRawMutex, SystemCommand, SYSTEM_CMD_QUEUE_SIZE>;

pub struct SystemStateManager<
    R: UsbResponder + 'static,
    RGB: RgbLed,
    DEBUG: StatefulOutputPin,
    BTN: Button,
> {
    cmd_rx: SystemCmdReceiver,
    responder: &'static R,
    rgb_led: RGB,
    debug_led: DEBUG,
    button: BTN,
}

impl<R: UsbResponder + 'static, RGB: RgbLed, DEBUG: StatefulOutputPin, BTN: Button>
    SystemStateManager<R, RGB, DEBUG, BTN>
{
    pub fn new(
        cmd_rx: SystemCmdReceiver,
        responder: &'static R,
        rgb_led: RGB,
        debug_led: DEBUG,
        button: BTN,
    ) -> Self {
        Self {
            cmd_rx,
            responder,
            rgb_led,
            debug_led,
            button,
        }
    }

    pub async fn run(self) {
        let debug_led_overridden = AtomicBool::new(false);
        let debug_led_state_override = AtomicBool::new(false);
        let error_state = AtomicBool::new(false);

        embassy_futures::join::join3(
            Self::report_button_presses(self.button, self.responder, &error_state),
            Self::blink_debug_led(
                self.debug_led,
                &debug_led_overridden,
                &debug_led_state_override,
            ),
            Self::handle_commands(
                self.rgb_led,
                self.cmd_rx,
                self.responder,
                &error_state,
                &debug_led_overridden,
                &debug_led_state_override,
            ),
        )
        .await;
    }

    async fn report_button_presses(
        mut button: BTN,
        responder: &'static R,
        error_state: &AtomicBool,
    ) {
        loop {
            button.wait_for_release().await;
            Timer::after_millis(50).await;
            if responder
                .send(&[
                    cmds::report_button_press::MAJOR,
                    cmds::report_button_press::MINOR,
                ])
                .await
                .is_err()
            {
                error_state.store(true, Ordering::Relaxed);
            } else {
                error_state.store(false, Ordering::Relaxed);
            }
        }
    }

    async fn blink_debug_led(
        mut debug_led: DEBUG,
        debug_led_overridden: &AtomicBool,
        debug_led_override_state: &AtomicBool,
    ) {
        loop {
            Timer::after_millis(1000).await;
            if debug_led_overridden.load(Ordering::Relaxed) {
                debug_led
                    .set_state(debug_led_override_state.load(Ordering::Relaxed).into())
                    .unwrap();
            } else {
                debug_led.toggle().unwrap();
            }
        }
    }

    async fn handle_commands(
        mut rgb_led: RGB,
        cmd_rx: SystemCmdReceiver,
        responder: &'static R,
        error_state: &AtomicBool,
        debug_led_overridden: &AtomicBool,
        debug_led_override_state: &AtomicBool,
    ) {
        loop {
            match cmd_rx.receive().await {
                SystemCommand::SetRgbState(SetRgbState { r, g, b }) => {
                    rgb_led.set_state(r, g, b);
                    responder
                        .send(&[
                            cmds::set_rgb_state_response::MAJOR,
                            cmds::set_rgb_state_response::MINOR,
                            0x00,
                        ])
                        .await
                        .map_err(|_| {
                            error_state.store(true, Ordering::Relaxed);
                        })
                        .unwrap();
                }
                SystemCommand::SetDebugLedState(SetDebugLedState { state }) => {
                    debug_led_override_state.store(state, Ordering::Relaxed);
                    debug_led_overridden.store(true, Ordering::Relaxed);
                    responder
                        .send(&[
                            cmds::set_led_state_response::MAJOR,
                            cmds::set_led_state_response::MINOR,
                            0x00,
                        ])
                        .await
                        .map_err(|_| error_state.store(true, Ordering::Relaxed))
                        .unwrap();
                }
            }
        }
    }
}
