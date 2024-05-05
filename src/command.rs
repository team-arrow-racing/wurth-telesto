/// Start byte.
pub const START: u8 = 0x02;

/// Maximum length.
pub const MAX_PAYLOAD_LEN: usize = 224;

/// Command.
#[derive(Debug)]
pub enum Command {
    Request(Request),
    Response(Response),
    Event(Event),
}

/// Request command.
#[derive(Debug)]
pub enum Request {
    /// Send data to configured address.
    SendData = 0x00,
    /// Send data to specific address.
    SendDataEx = 0x01,
    /// Switch operating mode.
    SetMode = 0x04,
    /// Reset module.
    Reset = 0x05,
    /// Change the RF channel.
    SetChannel = 0x06,
    /// Set the destination network id.
    SetDestinationNetworkId = 0x07,
    /// Set the destination address.
    SetDestinationAddress = 0x08,
    /// Change a user setting.
    SetUserSetting = 0x09,
    /// Read a user setting.
    GetUserSetting = 0x0A,
    /// Request RSSI of last `packet.
    Rssi = 0x0D,
    /// Go to shutdown mode.
    Shutdown = 0x0E,
    /// Go to standby mode.
    Standby = 0x0F,
    /// Change the radio transmit power.
    TransmitPower = 0x11,
    /// Perform a factory reset.
    FactoryReset = 0x12,
}

pub(crate) fn command(buf: &mut [u8], kind: Request, data: &[u8]) -> usize {
    assert!(data.len() <= MAX_PAYLOAD_LEN);

    let len = data.len() + 4;
    assert!(buf.len() >= len);

    buf[0] = START;
    buf[1] = kind as u8;
    buf[2..(data.len() + 2)].copy_from_slice(data);
    buf[len - 1] = checksum(&buf[0..len - 2]);

    len
}

pub(crate) fn checksum(bytes: &[u8]) -> u8 {
    let mut result = bytes[0];
    for byte in bytes[1..].iter() {
        result ^= byte;
    }

    result
}

#[allow(clippy::from_over_into)]
impl Into<u8> for Request {
    fn into(self) -> u8 {
        self as u8
    }
}

/// Send data error kind.
#[derive(Debug, Clone, Copy)]
pub enum SendDataError {
    AckTimeout = 0x01,
    InvalidChannel = 0x02,
    ChannelBusy = 0x03,
    ModuleBusy = 0x04,
    PayloadInvalid = 0xFF,
}

/// Command response.
#[derive(Debug)]
pub enum Response {
    /// Data has been sent.
    SendData = 0x40,
    /// Mode has been updated.
    SetMode = 0x44,
    /// Reset request received.
    Reset = 0x45,
    /// Channel has been updated.
    SetChannel = 0x46,
    /// Destination network id has been updated.
    SetDestinationNetworkId = 0x47,
    /// Distination address has been updated.
    SetDestinationAddress = 0x48,
    /// User setting has been updated.
    SetUserSetting = 0x49,
    /// Requested user setting value.
    GetUserSetting = 0x4A,
    /// Receive signal strength response value.
    Rssi = 0x4D,
    /// Shtudown request received.
    Shutdown = 0x4E,
    /// Standby request received.
    Standby = 0x4F,
    /// Radio transmit power has been updated.
    TransmitPower = 0x51,
    /// Factory reset request received.
    FactoryReset = 0x52,
}

impl Response {
    pub fn try_from_raw(raw: u8) -> Option<Self> {
        match raw {
            x if x == Self::SendData as u8 => Some(Self::SendData),
            x if x == Self::SetMode as u8 => Some(Self::SetMode),
            x if x == Self::Reset as u8 => Some(Self::Reset),
            x if x == Self::SetChannel as u8 => Some(Self::SetChannel),
            x if x == Self::SendData as u8 => Some(Self::SendData),
            x if x == Self::SetDestinationNetworkId as u8 => Some(Self::SetDestinationNetworkId),
            x if x == Self::SetDestinationAddress as u8 => Some(Self::SetDestinationAddress),
            x if x == Self::SetUserSetting as u8 => Some(Self::SetUserSetting),
            x if x == Self::GetUserSetting as u8 => Some(Self::GetUserSetting),
            x if x == Self::Rssi as u8 => Some(Self::Rssi),
            x if x == Self::Standby as u8 => Some(Self::Standby),
            x if x == Self::TransmitPower as u8 => Some(Self::TransmitPower),
            x if x == Self::FactoryReset as u8 => Some(Self::FactoryReset),
            _ => None,
        }
    }
}

#[derive(Debug)]
pub enum Event {
    /// Data has been repeated.
    DataRepeat = 0x80,
    /// Data has been received.
    DataReceived = 0x81,
    /// Reset has been applied.
    Reset = 0x85,
    /// Woke up from standby mode.
    Wakeup = 0x8F,
    /// Radio packet has been transmitted.
    PacketTransmit = 0x90,
}

impl Event {
    pub fn try_from_raw(raw: u8) -> Option<Self> {
        // bit of a hack, but it ensures a single source of truth for the integer -> enum mapping.
        match raw {
            x if x == Self::DataRepeat as u8 => Some(Self::DataRepeat),
            x if x == Self::DataReceived as u8 => Some(Self::DataReceived),
            x if x == Self::Reset as u8 => Some(Self::Reset),
            x if x == Self::Wakeup as u8 => Some(Self::Wakeup),
            x if x == Self::PacketTransmit as u8 => Some(Self::PacketTransmit),
            _ => None,
        }
    }
}
