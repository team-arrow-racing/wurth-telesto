/// User setting index.
#[derive(Debug)]
#[repr(u8)]
enum Setting {
    UartBaudRate = 0x00,
    DefaultRfProfile = 0x01,
    DefaultRfTxPower = 0x02,
    DefaultRfChannel = 0x03,
    DefaultAddressMode = 0x04,
    RetryNumbers = 0x06,
    DefaultDestinationNetId = 0x07,
    DefaultDestinationAddr = 0x08,
    SourceNetId = 0x0A,
    SourceAddr = 0x0B,
    ConfigFlags = 0x0F,
    RpFlags = 0x10,
    RpNumSlots = 0x11,
    FactorySettings = 0x20,
    FirmwareVersion = 0x21,
    RuntimeSettings = 0x22,
}
