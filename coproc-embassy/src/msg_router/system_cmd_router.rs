use crate::system_state::SystemCmdSender;

pub enum SystemCommand {
    SetRgbState(SetRgbState),
    SetDebugLedState(SetDebugLedState),
}

pub struct SetRgbState {
    pub r: u8,
    pub g: u8,
    pub b: u8,
}

pub struct SetDebugLedState {
    pub state: bool,
}

pub struct SystemCmdRouter {
    request_sender: SystemCmdSender,
}

impl SystemCmdRouter {
    pub fn new(request_sender: SystemCmdSender) -> Self {
        Self { request_sender }
    }

    pub async fn handle_set_led_state(&self, payload: &[u8]) {
        let state = payload[0] != 0;
        self.request_sender
            .send(SystemCommand::SetDebugLedState(SetDebugLedState { state }))
            .await;
    }

    pub async fn handle_set_rgb_state(&self, payload: &[u8]) {
        let r = payload[0];
        let g = payload[1];
        let b = payload[2];
        self.request_sender
            .send(SystemCommand::SetRgbState(SetRgbState { r, g, b }))
            .await;
    }
}
