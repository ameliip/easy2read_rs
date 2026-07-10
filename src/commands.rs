use crate::protocol::{Header, Avp, Message, HeaderError, AvpError, HEADER_LEN};
use crate::transport;

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

#[derive(Debug)]
pub enum MessageError {
    TooLong,
    TooShort,
    Header(HeaderError),
    Avp(AvpError),
    Transport(transport::TransportError)
}

impl From<HeaderError> for MessageError {
    fn from(e: HeaderError) -> Self {
        MessageError::Header(e)
    }
}

impl From<AvpError> for MessageError {
    fn from(e: AvpError) -> Self {
        MessageError::Avp(e)
    }
}

impl From<transport::TransportError> for MessageError {
    fn from(e: transport::TransportError) -> Self {
        MessageError::Transport(e)
    }
}