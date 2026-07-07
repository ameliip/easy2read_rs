const HEADER_LEN:     u32 = 10;
const AVP_HEADER_LEN: u16 = 6;
const VENDOR_ID:      u32 = 0x00005358; // CAEN Vendor ID From Spec

#[repr(u16)]
#[derive(Debug)]
enum MessageDir {
    Tx = 0x8001,
    Rx = 0x0001,
}

impl TryFrom<u16> for MessageDir {
    type Error = HeaderError;

    fn try_from(value: u16) -> Result<Self, Self::Error> {
        match value {
            0x8001 => Ok(MessageDir::Tx),
            0x0001 => Ok(MessageDir::Rx),
            _      => Err(HeaderError::WrongDirection),
        }
    }
}

#[derive(Debug)]
enum HeaderError {
    TooShort,
    WrongDirection, // Expected RX, found TX
    WrongVendorId,
}

#[derive(Debug)]
struct Header {
    direction: MessageDir,
    message_id: u16,
    vendor_id: u32,
    length: u16,
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
    reserved: u16,
    length: u16,
    attr_type: u16,
    value: Vec<u8>,
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

pub struct Message {
    header: Header,
    avp_list: Vec<Avp>,
}

#[derive(Debug)]
pub enum MessageError {
    TooLong,
}

impl Message {
    pub fn new(message_id: u16, avp_list: Vec<Avp>) -> Result<Self, MessageError> {
        let len: u16 = (avp_list.iter()
            .map(|avp| avp.length as u32)
            .sum::<u32>() + HEADER_LEN)
            .try_into()
            .map_err(|_| MessageError::TooLong)?;

        Ok(Message {
            header: Header::new(message_id, len),
            avp_list,
        })
    }
}