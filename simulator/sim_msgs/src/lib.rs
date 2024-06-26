use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum SimMessage {
    CommitRender,
    FrontendStarted,
    SetDebugLed(SetDebugLed),
    SetRgbLed(SetRgbLed),
    ReportButtonPress,
    SetMatrixRow(SetMatrixRow),
    SetMatrixRowRgb(SetMatrixRowRgb),
    RequestRgb,
    StartRecording,
    StopRecording,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct SetDebugLed {
    pub new_state: bool,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct SetRgbLed {
    pub r: u8,
    pub g: u8,
    pub b: u8,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct SetMatrixRow {
    pub row: usize,
    pub data: Vec<bool>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct SetMatrixRowRgb {
    pub row: usize,
    pub data: Vec<u16>,
}
