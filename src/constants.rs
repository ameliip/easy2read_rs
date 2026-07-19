#[derive(Debug)]
pub enum ConversionError {
    UnknownValue(u32),
}

// ============================================================
// AVP Attribute Type codes
// ============================================================
pub const AVP_COMMAND_NAME:      u16 = 0x0001;
pub const AVP_RESULT_CODE:       u16 = 0x0002;
pub const AVP_EVENT_TYPE:        u16 = 0x000E;
pub const AVP_TAG_ID_LEN:        u16 = 0x000F;
pub const AVP_TIMESTAMP:         u16 = 0x0010;
pub const AVP_TAG_ID:            u16 = 0x0011;
pub const AVP_TAG_TYPE:          u16 = 0x0012;
pub const AVP_CHANNEL_NAME:      u16 = 0x001E;
pub const AVP_CHANNEL_ADDRESS:   u16 = 0x001F;
pub const AVP_TRIGGER_NAME:      u16 = 0x0020;
pub const AVP_TRIGGER_TYPE:      u16 = 0x0021;
pub const AVP_READ_POINT_NAME:   u16 = 0x0022;
pub const AVP_TAG_VALUE:         u16 = 0x004D;
pub const AVP_TAG_ADDRESS:       u16 = 0x004E;
pub const AVP_LENGTH:            u16 = 0x0050;
pub const AVP_BIT_RATE:          u16 = 0x0051;
pub const AVP_POWER_GET:         u16 = 0x0052;
pub const AVP_PROTOCOL:          u16 = 0x0054;
pub const AVP_READ_POINT_STATUS: u16 = 0x0056;
pub const AVP_BOOLEAN:           u16 = 0x0057;
pub const AVP_IP_ADDRESS:        u16 = 0x0058;
pub const AVP_IP_NET_MASK:       u16 = 0x0059;
pub const AVP_IP_GATEWAY:        u16 = 0x005A;
pub const AVP_DESB_ENABLE:       u16 = 0x005B;
pub const AVP_FW_RELEASE:        u16 = 0x005C;
pub const AVP_DESB_STATUS:       u16 = 0x005D;
pub const AVP_EPC_PWD:           u16 = 0x005E;
pub const AVP_RF_ON_OFF:         u16 = 0x005F;
pub const AVP_BAUD_RATE:         u16 = 0x0060;
pub const AVP_DATA_BITS:         u16 = 0x0061;
pub const AVP_STOP_BITS:         u16 = 0x0062;
pub const AVP_PARITY:            u16 = 0x0063;
pub const AVP_FLOW_CTRL:         u16 = 0x0064;
pub const AVP_DATE_TIME:         u16 = 0x0065;
pub const AVP_SEL_UNSEL_OP:      u16 = 0x0066;
pub const AVP_BITMASK:           u16 = 0x0067;
pub const AVP_IO_REGISTER:       u16 = 0x0069;
pub const AVP_CONFIG_PARAMETER:  u16 = 0x006A;
pub const AVP_CONFIG_VALUE:      u16 = 0x006B;
pub const AVP_NO_OF_TRIGGERS:    u16 = 0x006C;
pub const AVP_NO_OF_CHANNELS:    u16 = 0x006D;
pub const AVP_EVENT_MODE:        u16 = 0x006E;
pub const AVP_UPGRADE_TYPE:      u16 = 0x006F;
pub const AVP_UPGRADE_ARGUMENT:  u16 = 0x0070;
pub const AVP_MEMORY_BANK:       u16 = 0x0071;
pub const AVP_PAYLOAD:           u16 = 0x0072;
pub const AVP_G2_PASSWORD:       u16 = 0x0073;
pub const AVP_G2_NSI:            u16 = 0x0074;
pub const AVP_Q_PARAMETER:       u16 = 0x0075;
pub const AVP_READER_INFO:       u16 = 0x0076;
pub const AVP_RF_REGULATION:     u16 = 0x0077;
pub const AVP_RF_CHANNEL:        u16 = 0x0078;
pub const AVP_RSSI:              u16 = 0x007A;
pub const AVP_OPTION:            u16 = 0x007B;
pub const AVP_XPC:               u16 = 0x007C;
pub const AVP_PC:                u16 = 0x007D;
pub const AVP_POWER_SET:         u16 = 0x0096;
pub const AVP_SOURCE_NAME:       u16 = 0x00FB;

// ============================================================
// Command codes
// ============================================================
pub const CMD_INVENTORY_TAG:                 u16 = 0x0013;
pub const CMD_ADD_READ_POINT_TO_SOURCE:      u16 = 0x005F;
pub const CMD_REMOVE_READ_POINT_FROM_SOURCE: u16 = 0x0060;
pub const CMD_SET_POWER:                     u16 = 0x0064;
pub const CMD_GET_POWER:                     u16 = 0x0073;
pub const CMD_SET_PROTOCOL:                  u16 = 0x0074;
pub const CMD_CHECK_READ_POINT_STATUS:       u16 = 0x0076;
pub const CMD_CHECK_READ_POINT_IN_SOURCE:    u16 = 0x0078;
pub const CMD_GET_PROTOCOL:                  u16 = 0x0079;
pub const CMD_GET_FIRMWARE_RELEASE:          u16 = 0x007C;
pub const CMD_RF_ON_OFF:                     u16 = 0x0080;
pub const CMD_GET_BIT_RATE:                  u16 = 0x0081;
pub const CMD_SET_RS232:                     u16 = 0x0083;
pub const CMD_SET_DATE_TIME:                 u16 = 0x0084;
pub const CMD_GET_IO:                        u16 = 0x0086;
pub const CMD_SET_IO:                        u16 = 0x0087;
pub const CMD_SET_IO_DIRECTION:              u16 = 0x0088;
pub const CMD_GET_IO_DIRECTION:              u16 = 0x0089;
pub const CMD_SET_SOURCE_CONFIG:             u16 = 0x008A;
pub const CMD_GET_SOURCE_CONFIG:             u16 = 0x008B;
pub const CMD_PROGRAM_ID_EPC_C1G2:           u16 = 0x0095;
pub const CMD_READ_TAG_DATA_EPC_C1G2:        u16 = 0x0096;
pub const CMD_WRITE_TAG_DATA_EPC_C1G2:       u16 = 0x0097;
pub const CMD_LOCK_TAG_EPC_C1G2:             u16 = 0x0098;
pub const CMD_KILL_TAG_EPC_C1G2:             u16 = 0x0099;
pub const CMD_QUERY_EPC_C1G2:                u16 = 0x009A;
pub const CMD_SET_Q_EPC_C1G2:                u16 = 0x009B;
pub const CMD_GET_Q_EPC_C1G2:                u16 = 0x009C;
pub const CMD_GET_READER_INFO:               u16 = 0x009E;
pub const CMD_SET_LBT_MODE:                  u16 = 0x009F;
pub const CMD_GET_LBT_MODE:                  u16 = 0x00A0;
pub const CMD_GET_RF_REGULATION:             u16 = 0x00A2;
pub const CMD_SET_RF_CHANNEL:                u16 = 0x00A3;
pub const CMD_GET_RF_CHANNEL:                u16 = 0x00A4;
pub const CMD_LOCK_BLOCK_PERMALOCK_EPC_C1G2: u16 = 0x00B1;
pub const CMD_READ_BLOCK_PERMALOCK_EPC_C1G2: u16 = 0x00B2;

// ============================================================
// ResultCode values (AVP 0x0002)
// ============================================================
#[repr(u16)]
#[derive(Debug, PartialEq)]
pub enum ResultCode {
    Success            = 0,
    Unknown            = 102,
    InvalidCmd         = 127,
    PwrOutRange        = 183,
    InvalidPar         = 200,
    TagNotPresent      = 202,
    TagWrite           = 203,
    TagBadAddress      = 205,
    InvalidFunction    = 206,
    Locked             = 209,
    Failed             = 210,
}

// ============================================================
// Protocol values (AVP 0x0054) — 4 bytes
// ============================================================
#[repr(u32)]
#[derive(Debug, PartialEq)]
pub enum Protocol {
    Iso18000_6B  = 0x00000000,
    EpcC1G1      = 0x00000001,
    Iso18000_6A  = 0x00000002,
    EpcC1G2      = 0x00000003,
}

impl TryFrom<u32> for Protocol {
    type Error = ConversionError;
    fn try_from(value: u32) -> Result<Self, Self::Error> {
        match value {
            0x00000000 => Ok(Self::Iso18000_6B),
            0x00000001 => Ok(Self::EpcC1G1),
            0x00000002 => Ok(Self::Iso18000_6A),
            0x00000003 => Ok(Self::EpcC1G2),
            _ => Err(ConversionError::UnknownValue(value))
        }
    }
}

// ============================================================
// TagType values (AVP 0x0012)
// ============================================================
#[repr(u16)]
#[derive(Debug, PartialEq)]
pub enum TagType {
    Iso18KB  = 0x0000,
    EpcC1G1  = 0x0001,
    Iso18KA  = 0x0002,
    EpcC1G2  = 0x0003,
    Epc119   = 0x0005,
}

// ============================================================
// EventType values (AVP 0x000E) — 4 bytes
// ============================================================
#[repr(u32)]
#[derive(Debug, PartialEq)]
pub enum EventType {
    Unknown  = 0x00000000,
    Glimpsed = 0x00000001,
    New      = 0x00000002,
    Observed = 0x00000003,
    Lost     = 0x00000004,
    Purged   = 0x00000005,
}

// ============================================================
// ReadPointStatus values (AVP 0x0056) — 4 bytes
// ============================================================
#[repr(u32)]
#[derive(Debug, PartialEq)]
pub enum ReadPointStatus {
    Good = 0x00000000,
    Poor = 0x00000001,
    Bad  = 0x00000002,
}

impl TryFrom<u32> for ReadPointStatus {
    type Error = ConversionError;
    fn try_from(value: u32) -> Result<Self, Self::Error> {
        match value {
            0x00000000 => Ok(Self::Good),
            0x00000001 => Ok(Self::Poor),
            0x00000002 => Ok(Self::Bad),
            _ => Err(ConversionError::UnknownValue(value))
        }
    }
}

// ============================================================
// MemoryBank values (AVP 0x0071)
// ============================================================
#[repr(u16)]
#[derive(Debug, PartialEq)]
pub enum MemoryBank {
    Reserved = 0x0000,
    Epc      = 0x0001,
    Tid      = 0x0002,
    User     = 0x0003,
}

// ============================================================
// Parity values (AVP 0x0063) — 4 bytes
// ============================================================
#[repr(u32)]
#[derive(Debug, PartialEq)]
pub enum Parity {
    None = 0x00000000,
    Odd  = 0x00000001,
    Even = 0x00000002,
}

// ============================================================
// FlowCtrl values (AVP 0x0064) — 4 bytes
// ============================================================
#[repr(u32)]
#[derive(Debug, PartialEq)]
pub enum FlowCtrl {
    None     = 0x00000000,
    Hardware = 0x00000001,
    Software = 0x00000002,
}

// ============================================================
// EventMode values (AVP 0x006E)
// ============================================================
#[repr(u16)]
#[derive(Debug, PartialEq)]
pub enum EventMode {
    ReadCycle   = 0x0000,
    Time        = 0x0001,
    NoEvent     = 0x0002,
}

// ============================================================
// ConfigParameter values (AVP 0x006A) — 4 bytes
// ============================================================
#[repr(u32)]
#[derive(Debug, PartialEq)]
pub enum ConfigParameter {
    ReadCycle          = 0x00000000,
    ObservedThreshold  = 0x00000001,
    LostThreshold      = 0x00000002,
    StartingQ          = 0x00000003, // EPC C1GEN2 only
    Session            = 0x00000004, // EPC C1GEN2 only
    Target             = 0x00000005, // EPC C1GEN2 only
    Selected           = 0x00000006, // EPC C1GEN2 only
    DataExchangeStatus = 0x00000007, // ISO 18000-6B only
    AntennaDwellTime   = 0x00000008, // A528 only
    InventoryType      = 0x00000009, // A528 only
}

// ============================================================
// RFRegulation values (AVP 0x0077)
// ============================================================
#[repr(u16)]
#[derive(Debug, PartialEq)]
pub enum RFRegulation {
    EtsiEn302208  = 0x0000,
    EtsiEn300220  = 0x0001,
    Fcc           = 0x0002,
    Malaysia      = 0x0003,
    Japan         = 0x0004,
    Korea         = 0x0005,
    Australia     = 0x0006,
    China         = 0x0007,
    Taiwan        = 0x0008,
    Singapore     = 0x0009,
    Brazil        = 0x000A,
    JapanStdT106  = 0x000B,
    JapanStdT107  = 0x000C,
}

impl TryFrom<u16> for RFRegulation {
    type Error = ConversionError;
    fn try_from(value: u16) -> Result<Self, Self::Error> {
        match value {
            0x0000 => Ok(Self::EtsiEn302208),
            0x0001 => Ok(Self::EtsiEn300220),
            0x0002 => Ok(Self::Fcc),
            0x0003 => Ok(Self::Malaysia), 
            0x0004 => Ok(Self::Japan),
            0x0005 => Ok(Self::Korea),
            0x0006 => Ok(Self::Australia),
            0x0007 => Ok(Self::China),
            0x0008 => Ok(Self::Taiwan),
            0x0009 => Ok(Self::Singapore),
            0x000A => Ok(Self::Brazil),
            0x000B => Ok(Self::JapanStdT106),
            0x000C => Ok(Self::JapanStdT107),
            _ => Err(ConversionError::UnknownValue(value as u32))
        }
    }
}

// ============================================================
// SelUnselOp values (AVP 0x0066)
// ============================================================
#[repr(u16)]
#[derive(Debug, PartialEq)]
pub enum SelUnselOp {
    SelectEqual       = 0x0000,
    SelectNotEqual    = 0x0001,
    SelectGreater     = 0x0002,
    SelectLower       = 0x0003,
    UnselectEqual     = 0x0004,
    UnselectNotEqual  = 0x0005,
    UnselectGreater   = 0x0006,
    UnselectLower     = 0x0007,
}