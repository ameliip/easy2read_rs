pub const HEADER_LEN:     u16 = 10;
const AVP_HEADER_LEN: u16 = 6;
const VENDOR_ID:      u32 = 0x00005358; // CAEN Vendor ID From Spec

#[repr(u16)]
#[derive(Debug)]
pub enum MessageDir {
    Tx = 0x8001,
    Rx = 0x0001,
}

impl TryFrom<u16> for MessageDir {
    type Error = HeaderError;

    fn try_from(value: u16) -> Result<Self, Self::Error> {
        match value {
            0x8001 => Ok(Self::Tx),
            0x0001 => Ok(Self::Rx),
            _      => Err(HeaderError::WrongDirection),
        }
    }
}

#[derive(Debug)]
pub enum HeaderError {
    TooShort,
    WrongDirection, // Expected RX, found TX
    WrongVendorId,
}

#[derive(Debug)]
pub struct Header {
    pub direction: MessageDir,
    pub message_id: u16,
    pub vendor_id: u32,
    pub length: u16,
}

impl Header {
    pub fn new(mess_id: u16, len: u16) -> Self {
        Header {
            direction:  MessageDir::Tx,
            message_id: mess_id,
            vendor_id:  VENDOR_ID,
            length:     len,
        }
    }
}

impl TryFrom<&[u8]> for Header {
    type Error = HeaderError;

    fn try_from(data: &[u8]) -> Result<Self, Self::Error> {
        if data.len() < HEADER_LEN as usize {
            return Err(Self::Error::TooShort);
        }
        
        let direction = MessageDir::try_from(u16::from_be_bytes(data[0..2]
            .try_into()
            .unwrap()))?;
        
        let message_id = u16::from_be_bytes(data[2..4]
            .try_into()
            .unwrap()); // Unwrap is safe as len is checked before
        
        let vendor_id = u32::from_be_bytes(data[4..8].
            try_into()
            .unwrap()); // Unwrap is safe as len is checked before
        
        if vendor_id != VENDOR_ID {
            return Err(Self::Error::WrongVendorId); 
        }

        let length = u16::from_be_bytes(data[8..10]
            .try_into()
            .unwrap());// Unwrap is safe as len is checked before

        Ok(Header {
            direction,
            message_id,
            vendor_id,
            length,
        })
    }
    
}

#[derive(Debug)]
pub enum AvpError {
    TooShort,
    TooLong,
    InvalidLength,
}

pub struct Avp {
    pub reserved: u16,
    pub length: u16,
    pub attr_type: u16,
    pub value: Vec<u8>,
}

impl Avp {
    pub fn new(attr_type: u16, value: Vec<u8>) -> Result<Self, AvpError> {
        let length: u16 = (AVP_HEADER_LEN as usize + value.len())
            .try_into()
            .map_err(|_| AvpError::TooLong)?;

        Ok(Avp { reserved: 0, length, attr_type, value})
    }
}

impl TryFrom<&[u8]> for Avp {
    type Error = AvpError;

    fn try_from(data: &[u8]) -> Result<Self, Self::Error> {

        if data.len() < AVP_HEADER_LEN as usize{
            return Err(Self::Error::TooShort);
        }
        
        let reserved = u16::from_be_bytes(data[0..2].try_into().unwrap());
        let length = u16::from_be_bytes(data[2..4].try_into().unwrap());
        let attr_type = u16::from_be_bytes(data[4..6].try_into().unwrap());
        let value_len = length.checked_sub(AVP_HEADER_LEN)
                                  .ok_or(Self::Error::InvalidLength)? as usize;
        
        if data.len() < (AVP_HEADER_LEN as usize + value_len) {
            return Err(Self::Error::TooShort);
        }

        let value = data[6..(6 + value_len)].to_vec();

        Ok(Avp{reserved, length, attr_type, value})        
    }

}

impl From<&Avp> for Vec<u8> {
    fn from (avp: &Avp) -> Vec<u8> {
        let mut bytes: Vec<u8> = Vec::new();

        bytes.extend_from_slice(&(avp.reserved as u16).to_be_bytes());
        bytes.extend_from_slice(&(avp.length as u16).to_be_bytes());
        bytes.extend_from_slice(&(avp.attr_type as u16).to_be_bytes());
        bytes.extend(&avp.value);

        return bytes;
    } 
}

pub struct Message {
    pub header: Header,
    pub avp_list: Vec<Avp>,
}

#[derive(Debug)]
pub enum MessageError {
    TooLong,
    TooShort,
    Header(HeaderError),
    Avp(AvpError),
    InvalidValue,
}

impl From<HeaderError> for MessageError {
    fn from(e: HeaderError) -> Self { MessageError::Header(e) }
}

impl From<AvpError> for MessageError {
    fn from(e: AvpError) -> Self { MessageError::Avp(e) }
}

impl Message {
    pub fn new(message_id: u16, avp_list: Vec<Avp>) -> Result<Self, MessageError> {
        let len: u16 = (avp_list.iter()
            .map(|avp| avp.length as u32)
            .sum::<u32>() + HEADER_LEN as u32)
            .try_into()
            .map_err(|_| MessageError::TooLong)?;

        Ok(Message {
            header: Header::new(message_id, len),
            avp_list,
        })
    }
}

impl TryFrom<&[u8]> for Message {
    type Error = MessageError;

    fn try_from(data: &[u8]) -> Result<Self, Self::Error> {
        let header = Header::try_from(data).map_err(|e| MessageError::Header(e))?;
        let mut avp_list: Vec<Avp> = Vec::new();
        let mut offset: usize = HEADER_LEN as usize;

        if header.length > 0xfff {
            return Err(Self::Error::TooLong);
        }

        while offset < header.length as usize {
            let avp: Avp = Avp::try_from(&data[offset..]).map_err(|e| MessageError::Avp(e))?;
            offset += avp.length as usize;
            avp_list.push(avp);            
        }
        
        if avp_list.len() > 0 {
                Ok(Message {header, avp_list})
        } else {
            Err(Self::Error::TooShort)
        }
    }
}

impl From<Message> for Vec<u8> {
    fn from(message: Message) -> Vec<u8> {
        let mut bytes: Vec<u8> = Vec::new();

        bytes.extend_from_slice(&(message.header.direction as u16).to_be_bytes());
        bytes.extend_from_slice(&(message.header.message_id as u16).to_be_bytes());
        bytes.extend_from_slice(&(message.header.vendor_id as u32).to_be_bytes());
        bytes.extend_from_slice(&(message.header.length as u16).to_be_bytes());
        
        for avp in &message.avp_list {
            bytes.extend(Vec::<u8>::from(avp));
        }

        return bytes;
    }
}