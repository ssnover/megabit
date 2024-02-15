use std::io;
use strum::AsRefStr;

#[derive(Clone, Debug, AsRefStr)]
pub enum SerialMessage {
    SetLedState(SetLedState),
    SetLedStateResponse(SetLedStateResponse),
    SetRgbState(SetRgbState),
    SetRgbStateResponse(SetRgbStateResponse),
    GetDisplayInfo(GetDisplayInfo),
    GetDisplayInfoResponse(GetDisplayInfoResponse),
    ReportButtonPress,
    UpdateRow(UpdateRow),
    UpdateRowResponse(UpdateRowResponse),
    UpdateRowRgb(UpdateRowRgb),
    UpdateRowRgbResponse(UpdateRowRgbResponse),
    Ping,
    PingResponse,
}

impl SerialMessage {
    pub fn to_bytes(self) -> Vec<u8> {
        let mut out = vec![];
        match self {
            SerialMessage::UpdateRow(inner) => {
                out.push(0xa0);
                out.push(0x00);
                out.append(&mut inner.to_bytes())
            }
            SerialMessage::UpdateRowResponse(inner) => {
                out.push(0xa0);
                out.push(0x01);
                out.append(&mut inner.to_bytes())
            }
            SerialMessage::UpdateRowRgb(inner) => {
                out.push(0xa0);
                out.push(0x02);
                out.append(&mut inner.to_bytes())
            }
            SerialMessage::UpdateRowRgbResponse(inner) => {
                out.push(0xa0);
                out.push(0x03);
                out.append(&mut inner.to_bytes())
            }
            SerialMessage::GetDisplayInfo(inner) => {
                out.push(0xa0);
                out.push(0x04);
                out.append(&mut inner.to_bytes())
            }
            SerialMessage::GetDisplayInfoResponse(inner) => {
                out.push(0xa0);
                out.push(0x05);
                out.append(&mut inner.to_bytes())
            }
            SerialMessage::SetLedState(inner) => {
                out.push(0xde);
                out.push(0x00);
                out.append(&mut inner.to_bytes())
            }
            SerialMessage::SetLedStateResponse(inner) => {
                out.push(0xde);
                out.push(0x01);
                out.append(&mut inner.to_bytes())
            }
            SerialMessage::SetRgbState(inner) => {
                out.push(0xde);
                out.push(0x02);
                out.append(&mut inner.to_bytes())
            }
            SerialMessage::SetRgbStateResponse(inner) => {
                out.push(0xde);
                out.push(0x03);
                out.append(&mut inner.to_bytes())
            }
            SerialMessage::ReportButtonPress => {
                out.push(0xde);
                out.push(0x04);
            }
            SerialMessage::Ping => {
                out.push(0xde);
                out.push(0xfe);
            }
            SerialMessage::PingResponse => {
                out.push(0xde);
                out.push(0xff);
            }
        }

        out
    }

    pub fn try_from_bytes(data: &[u8]) -> io::Result<Self> {
        if data.len() >= 2 {
            match (data[0], data[1]) {
                (0xa0, 0x00) => Ok(SerialMessage::UpdateRow(UpdateRow::try_from_bytes(
                    &data[2..],
                )?)),
                (0xa0, 0x01) => Ok(SerialMessage::UpdateRowResponse(
                    UpdateRowResponse::try_from_bytes(&data[2..])?,
                )),
                (0xa0, 0x02) => Ok(SerialMessage::UpdateRowRgb(UpdateRowRgb::try_from_bytes(
                    &data[2..],
                )?)),
                (0xa0, 0x03) => Ok(SerialMessage::UpdateRowRgbResponse(
                    UpdateRowRgbResponse::try_from_bytes(&data[2..])?,
                )),
                (0xa0, 0x04) => Ok(SerialMessage::GetDisplayInfo(
                    GetDisplayInfo::try_from_bytes(&data[2..])?,
                )),
                (0xa0, 0x05) => Ok(SerialMessage::GetDisplayInfoResponse(
                    GetDisplayInfoResponse::try_from_bytes(&data[2..])?,
                )),
                (0xde, 0x00) => Ok(SerialMessage::SetLedState(SetLedState::try_from_bytes(
                    &data[2..],
                )?)),
                (0xde, 0x01) => Ok(SerialMessage::SetLedStateResponse(
                    SetLedStateResponse::try_from_bytes(&data[2..])?,
                )),
                (0xde, 0x02) => Ok(SerialMessage::SetRgbState(SetRgbState::try_from_bytes(
                    &data[2..],
                )?)),
                (0xde, 0x03) => Ok(SerialMessage::SetRgbStateResponse(
                    SetRgbStateResponse::try_from_bytes(&data[2..])?,
                )),
                (0xde, 0x04) => Ok(SerialMessage::ReportButtonPress),
                (0xde, 0xfe) => Ok(SerialMessage::Ping),
                (0xde, 0xff) => Ok(SerialMessage::PingResponse),
                _ => {
                    tracing::error!(
                        "Unexpected serial message kind 0x{:02x}{:02x}",
                        data[0],
                        data[1]
                    );
                    Err(io::ErrorKind::InvalidData.into())
                }
            }
        } else {
            Err(io::ErrorKind::InvalidData.into())
        }
    }
}

#[derive(Clone, Debug)]
#[repr(u8)]
pub enum Status {
    Success = 0,
    Failure = 1,
    InProgress = 2,
}

impl TryFrom<u8> for Status {
    type Error = io::Error;
    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value {
            0 => Ok(Status::Success),
            1 => Ok(Status::Failure),
            2 => Ok(Status::InProgress),
            _ => Err(io::ErrorKind::InvalidData.into()),
        }
    }
}

impl From<Status> for u8 {
    fn from(value: Status) -> Self {
        match value {
            Status::Success => 0,
            Status::Failure => 1,
            Status::InProgress => 2,
        }
    }
}

#[derive(Clone, Debug)]
pub struct SetLedState {
    pub new_state: bool,
}

impl SetLedState {
    pub fn to_bytes(self) -> Vec<u8> {
        vec![if self.new_state { 0x01 } else { 0x00 }]
    }

    pub fn try_from_bytes(data: &[u8]) -> io::Result<Self> {
        if data.len() == 1 {
            Ok(Self {
                new_state: data[0] != 0x00,
            })
        } else {
            Err(io::ErrorKind::InvalidData.into())
        }
    }
}

#[derive(Clone, Debug)]
pub struct SetLedStateResponse {
    pub status: Status,
}

impl SetLedStateResponse {
    pub fn to_bytes(self) -> Vec<u8> {
        vec![self.status.into()]
    }

    pub fn try_from_bytes(data: &[u8]) -> io::Result<Self> {
        if data.len() == 1 {
            Ok(Self {
                status: Status::try_from(data[0])?,
            })
        } else {
            Err(io::ErrorKind::InvalidData.into())
        }
    }
}

#[derive(Clone, Debug)]
pub struct SetRgbState {
    pub r: u8,
    pub g: u8,
    pub b: u8,
}

impl SetRgbState {
    pub fn to_bytes(self) -> Vec<u8> {
        vec![self.r, self.g, self.b]
    }

    pub fn try_from_bytes(data: &[u8]) -> io::Result<Self> {
        if data.len() == 3 {
            Ok(Self {
                r: data[0],
                g: data[1],
                b: data[2],
            })
        } else {
            Err(io::ErrorKind::InvalidData.into())
        }
    }
}

#[derive(Clone, Debug)]
pub struct SetRgbStateResponse {
    pub status: Status,
}

impl SetRgbStateResponse {
    pub fn to_bytes(self) -> Vec<u8> {
        vec![self.status.into()]
    }

    pub fn try_from_bytes(data: &[u8]) -> io::Result<Self> {
        if data.len() == 1 {
            Ok(Self {
                status: Status::try_from(data[0])?,
            })
        } else {
            Err(io::ErrorKind::InvalidData.into())
        }
    }
}

#[derive(Debug, Clone)]
pub struct UpdateRow {
    pub row_number: u8,
    pub row_data_len: u8,
    pub row_data: Vec<u8>,
}

impl UpdateRow {
    pub fn to_bytes(mut self) -> Vec<u8> {
        let mut out = vec![self.row_number, self.row_data_len];
        out.append(&mut self.row_data);
        out
    }

    pub fn try_from_bytes(data: &[u8]) -> io::Result<Self> {
        if data.len() >= 3 {
            let row_number = data[0];
            let row_data_len = data[1];
            let row_data = Vec::from(&data[2..]);
            Ok(UpdateRow {
                row_number,
                row_data_len,
                row_data,
            })
        } else {
            Err(io::ErrorKind::InvalidData.into())
        }
    }
}

pub fn pack_bools_to_bytes(bits: &[bool]) -> Vec<u8> {
    bits.into_iter()
        .enumerate()
        .fold(Vec::new(), |mut acc, (idx, elem)| {
            let byte_idx = idx / 8;
            let bit_idx = idx % 8;
            if acc.len() <= byte_idx {
                acc.push(0x00);
            }
            if *elem {
                acc[byte_idx] |= 1 << bit_idx;
            }
            acc
        })
}

#[derive(Debug, Clone)]
pub struct UpdateRowResponse {
    pub status: Status,
}

impl UpdateRowResponse {
    pub fn to_bytes(self) -> Vec<u8> {
        vec![self.status.into()]
    }

    pub fn try_from_bytes(data: &[u8]) -> io::Result<Self> {
        if data.len() == 1 {
            Ok(Self {
                status: Status::try_from(data[0])?,
            })
        } else {
            Err(io::ErrorKind::InvalidData.into())
        }
    }
}

#[derive(Debug, Clone)]
pub struct UpdateRowRgb {
    pub row_number: u8,
    pub row_data_len: u8,
    pub row_data: Vec<u16>,
}

impl UpdateRowRgb {
    pub fn to_bytes(self) -> Vec<u8> {
        let mut out = vec![self.row_number, self.row_data_len];
        let mut row_data = self
            .row_data
            .into_iter()
            .map(|elem| elem.to_be_bytes())
            .flatten()
            .collect();
        out.append(&mut row_data);
        out
    }

    pub fn try_from_bytes(data: &[u8]) -> io::Result<Self> {
        if data.len() >= 3 {
            let row_number = data[0];
            let row_data_len = data[1];
            let row_data = Vec::from(&data[2..]);
            let row_data = data[2..]
                .iter()
                .enumerate()
                .step_by(2)
                .map(|(idx, _elem)| u16::from_be_bytes([row_data[idx], row_data[idx + 1]]))
                .collect();
            Ok(Self {
                row_number,
                row_data_len,
                row_data,
            })
        } else {
            Err(io::ErrorKind::InvalidData.into())
        }
    }
}

#[derive(Debug, Clone)]
pub struct UpdateRowRgbResponse {
    pub status: Status,
}

impl UpdateRowRgbResponse {
    pub fn to_bytes(self) -> Vec<u8> {
        vec![self.status.into()]
    }

    pub fn try_from_bytes(data: &[u8]) -> io::Result<Self> {
        if data.len() == 1 {
            Ok(Self {
                status: Status::try_from(data[0])?,
            })
        } else {
            Err(io::ErrorKind::InvalidData.into())
        }
    }
}

#[derive(Debug, Clone)]
pub struct GetDisplayInfo;

impl GetDisplayInfo {
    pub fn to_bytes(self) -> Vec<u8> {
        vec![]
    }

    pub fn try_from_bytes(data: &[u8]) -> io::Result<Self> {
        if data.len() == 0 {
            Ok(Self {})
        } else {
            Err(io::ErrorKind::InvalidData.into())
        }
    }
}

#[derive(Debug, Clone)]
#[repr(u8)]
pub enum PixelRepresentation {
    Monocolor = 0,
    RGB555 = 1,
}

impl PixelRepresentation {
    pub fn try_from_byte(byte: u8) -> io::Result<Self> {
        Ok(match byte {
            0 => PixelRepresentation::Monocolor,
            1 => PixelRepresentation::RGB555,
            _ => {
                return Err(io::ErrorKind::InvalidData.into());
            }
        })
    }
}

#[derive(Debug, Clone)]
pub struct GetDisplayInfoResponse {
    pub width: u32,
    pub height: u32,
    pub pixel_representation: PixelRepresentation,
}

impl GetDisplayInfoResponse {
    pub fn to_bytes(self) -> Vec<u8> {
        [
            &self.width.to_be_bytes()[..],
            &self.height.to_be_bytes()[..],
            &[self.pixel_representation as u8][..],
        ]
        .concat()
    }

    pub fn try_from_bytes(data: &[u8]) -> io::Result<Self> {
        if data.len() == 9 {
            Ok(Self {
                width: u32::from_be_bytes(data[0..4].try_into().unwrap()),
                height: u32::from_be_bytes(data[4..8].try_into().unwrap()),
                pixel_representation: PixelRepresentation::try_from_byte(data[8])?,
            })
        } else {
            Err(io::ErrorKind::InvalidData.into())
        }
    }
}
