use crate::protocol;
use crate::transport;
use crate::constants;

#[derive(Debug)]
pub enum CommandError {
    Message(protocol::MessageError),      // Error at the protocol level (malformed message, invalid length, etc.)
    Transport(transport::TransportError), // Error at the transport level (timeout, disconnection, etc.)
    ReaderError(u16),                     // Reader returned a non-success result code
    UnknownValue(u32),                    // Unknown value in message
}

impl From<protocol::MessageError> for CommandError {
    fn from(e: protocol::MessageError) -> Self {CommandError::Message(e)}
}

impl From<transport::TransportError> for CommandError {
    fn from(e: transport::TransportError) -> Self {CommandError::Transport(e)}
}

impl From<protocol::HeaderError> for CommandError {
    fn from(e: protocol::HeaderError) -> Self {
        CommandError::Message(protocol::MessageError::Header(e))
    }
}

impl From<protocol::AvpError> for CommandError {
    fn from(e: protocol::AvpError) -> Self {
        CommandError::Message(protocol::MessageError::Avp(e))
    }
}

impl From<constants::ConversionError> for CommandError {
    fn from(e: constants::ConversionError) -> Self {
        match e {
            constants::ConversionError::UnknownValue(v) => CommandError::UnknownValue(v)
        }
    }
}

/// Client for communicating with a CAEN UHF RFID reader over serial.
///
/// Manages the serial connection and provides methods for sending commands
/// and receiving responses according to the CAEN easy2read protocol.
///
/// # Example
/// ```no_run
/// let mut client = Easy2ReadClient::new("COM3", 115200)?;
/// client.cmd_set_protocol(Protocol::EpcC1G2)?;
/// ```
pub struct Easy2ReadClient {
    transport: Box<dyn transport::Transport>,
    message_id: u16,
}

impl Easy2ReadClient {
    
    /// Creates a new `Easy2ReadClient` connected to the specified serial port.
    ///
    /// # Arguments
    /// * `port_name` - Serial port name (e.g. `"COM3"` on Windows, `"/dev/ttyUSB0"` on Linux)
    /// * `baud_rate` - Baud rate for the serial connection (typically `115200`)
    ///
    /// # Errors
    /// * `CommandError::Transport` - If the port cannot be opened (wrong name, permission denied, etc.)
    pub fn new(port_name: &str, baud_rate: u32) -> Result<Self, CommandError> {
        let transport = transport::SerialTransport::new(port_name, baud_rate)
            .map_err(|e| CommandError::Transport(e))?;
        let message_id: u16 = 0;

        Ok(Easy2ReadClient { transport: Box::new(transport), message_id })
    }

    /// Reads a complete message from the transport layer.
    /// Reads the header first, then the payload based on the length field in the header.
    ///
    /// # Errors
    /// * `CommandError::Transport` - If a communication error occurs on the serial port
    /// * `CommandError::Message` - If the received bytes cannot be parsed as a valid message
    fn receive_message(&mut self) -> Result<protocol::Message, CommandError> {
        // Reads Header
        let mut header_bytes = [0u8; protocol::HEADER_LEN as usize];
        self.transport.read_exact(&mut header_bytes)?;
        let header: protocol::Header = protocol::Header::try_from(&header_bytes[..])?;

        // Reads payload
        let payload_len: usize = (header.length - protocol::HEADER_LEN) as usize;
        let mut payload_bytes = vec![0u8; payload_len];
        self.transport.read_exact(&mut payload_bytes)?;

        let mut full = Vec::from(header_bytes.as_slice());
        full.extend(payload_bytes);
        
        Ok(protocol::Message::try_from(&full[..])?)
    }

    /// Checks the `ResultCode` AVP in a response message.
    ///
    /// # Errors
    /// * `CommandError::Message` - If the response does not contain a `ResultCode` AVP
    /// * `CommandError::ReaderError` - If the result code is not `ERR_SUCCESS`
    fn check_result_code(&self, response: &protocol::Message) -> Result<(), CommandError> {
       let result_avp = response.avp_list.iter()
            .find(|avp| avp.attr_type == constants::AVP_RESULT_CODE)
            .ok_or(CommandError::Message(protocol::MessageError::TooShort))?;

        let result_code = u16::from_be_bytes(result_avp.value[0..2].try_into().unwrap());

        if result_code == constants::ResultCode::Success as u16 {
            Ok(())
        } else {
            Err(CommandError::ReaderError(result_code))
        }
    }

    /// Gets the air protocol currently in use by the reader.
    ///
    /// # Returns
    /// The current [`constants::Protocol`] in use
    ///
    /// # Errors
    /// * `CommandError::Transport` - If a communication error occurs on the serial port
    /// * `CommandError::ReaderError` - If the reader returns a non-success result code
    /// * `CommandError::Message` - If the response does not contain the expected AVP
    pub fn cmd_get_protocol(&mut self) -> Result<constants::Protocol, CommandError> {
        // Build Command
        let avp_cmd_name = protocol::Avp::new(
            constants::AVP_COMMAND_NAME, 
            constants::CMD_GET_PROTOCOL.to_be_bytes().to_vec(),
        )?;
        let message: protocol::Message = protocol::Message::new(
            self.message_id,
            vec![avp_cmd_name]
        )?;

        let bytes = Vec::<u8>::from(message);
        self.transport.write_all(&bytes)?;
        let response = self.receive_message()?;

        self.check_result_code(&response)?;

        // Find protocol avp in response
        let result_avp = response.avp_list.iter()
            .find(|avp| avp.attr_type == constants::AVP_PROTOCOL)
            .ok_or(CommandError::Message(protocol::MessageError::TooShort))?;

        // Extract protocol value
        let protocol_val = u32::from_be_bytes(result_avp.value[0..4].try_into().unwrap());
        let protocol = constants::Protocol::try_from(protocol_val)?;
        Ok(protocol)
    }

    /// Sets the air protocol to use for tag communication.
    ///
    /// # Arguments
    /// * `protocol` - The protocol to set (see [`constants::Protocol`])
    ///
    /// # Errors
    /// * `CommandError::Transport` - If a communication error occurs on the serial port
    /// * `CommandError::ReaderError` - If the reader rejects the command (check [`constants::ResultCode`])
    pub fn cmd_set_protocol(&mut self, protocol: constants::Protocol) -> Result<(), CommandError> {
        // Build Command
        let avp_cmd_name = protocol::Avp::new(
            constants::AVP_COMMAND_NAME, 
            constants::CMD_SET_PROTOCOL.to_be_bytes().to_vec(),
        )?;
        let avp_prot_type = protocol::Avp::new(
            constants::AVP_PROTOCOL,
            (protocol as u32).to_be_bytes().to_vec(),
        )?;
        let message: protocol::Message = protocol::Message::new(
            self.message_id,
            vec![avp_cmd_name, avp_prot_type]
        )?;

        let bytes = Vec::<u8>::from(message);
        self.transport.write_all(&bytes)?;
        let response = self.receive_message()?;

        self.check_result_code(&response)?;
        Ok(())
    }

    /// Gets the current RF power level of the reader.
    ///
    /// # Returns
    /// The current power level in milliwatts (u32)
    ///
    /// # Errors
    /// * `CommandError::Transport` - If a communication error occurs on the serial port
    /// * `CommandError::ReaderError` - If the reader returns a non-success result code
    /// * `CommandError::Message` - If the response does not contain the expected AVP
    pub fn cmd_get_power(&mut self) -> Result<u32, CommandError> {
        // Build Command
        let avp_cmd_name = protocol::Avp::new(
            constants::AVP_COMMAND_NAME, 
            constants::CMD_GET_POWER.to_be_bytes().to_vec(),
        )?;
        let message: protocol::Message = protocol::Message::new(
            self.message_id,
            vec![avp_cmd_name]
        )?;

        let bytes = Vec::<u8>::from(message);
        self.transport.write_all(&bytes)?;
        let response = self.receive_message()?;

        self.check_result_code(&response)?;

        // Find protocol avp in response
        let result_avp = response.avp_list.iter()
            .find(|avp| avp.attr_type == constants::AVP_POWER_GET)
            .ok_or(CommandError::Message(protocol::MessageError::TooShort))?;

        Ok(u32::from_be_bytes(result_avp.value[0..4].try_into().unwrap()))
    }    

    /// Sets the RF power level for the reader.
    ///
    /// # Arguments
    /// * `power_mw` - Power level in milliwatts
    ///
    /// # Errors
    /// Returns `CommandError::ReaderError` if the reader rejects the value
    pub fn cmd_set_power(&mut self, power_mw: u32) -> Result<(), CommandError> {
        // Build Command
        let avp_cmd_name = protocol::Avp::new(
            constants::AVP_COMMAND_NAME, 
            constants::CMD_SET_POWER.to_be_bytes().to_vec(),
        )?;
        let avp_pow_lvl = protocol::Avp::new(
            constants::AVP_POWER_SET,
            (power_mw as u32).to_be_bytes().to_vec(),
        )?;
        let message: protocol::Message = protocol::Message::new(
            self.message_id,
            vec![avp_cmd_name, avp_pow_lvl]
        )?;

        let bytes = Vec::<u8>::from(message);
        self.transport.write_all(&bytes)?;
        let response = self.receive_message()?;

        self.check_result_code(&response)?;
        Ok(())
    }

}