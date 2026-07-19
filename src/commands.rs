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

    /// Converts a `&str` to a NULL-terminated `Vec<u8>` as required by the CAEN easy2read protocol.
    /// All string values in the protocol must end with a `0x00` byte.
    fn null_terminate(s: &str) -> Vec<u8> {
        let mut v = s.as_bytes().to_vec();
        v.push(0);
        v
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

        let avp_cmd_name = protocol::Avp::new(
            constants::AVP_COMMAND_NAME, 
            constants::CMD_GET_PROTOCOL.to_be_bytes().to_vec(),
        )?;
        let message = protocol::Message::new(
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

        let avp_cmd_name = protocol::Avp::new(
            constants::AVP_COMMAND_NAME, 
            constants::CMD_SET_PROTOCOL.to_be_bytes().to_vec(),
        )?;
        let avp_prot_type = protocol::Avp::new(
            constants::AVP_PROTOCOL,
            (protocol as u32).to_be_bytes().to_vec(),
        )?;
        let message = protocol::Message::new(
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

        let avp_cmd_name = protocol::Avp::new(
            constants::AVP_COMMAND_NAME, 
            constants::CMD_GET_POWER.to_be_bytes().to_vec(),
        )?;
        let message = protocol::Message::new(
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

        let avp_cmd_name = protocol::Avp::new(
            constants::AVP_COMMAND_NAME, 
            constants::CMD_SET_POWER.to_be_bytes().to_vec(),
        )?;
        let avp_pow_lvl = protocol::Avp::new(
            constants::AVP_POWER_SET,
            power_mw.to_be_bytes().to_vec(),
        )?;
        let message = protocol::Message::new(
            self.message_id,
            vec![avp_cmd_name, avp_pow_lvl]
        )?;

        let bytes = Vec::<u8>::from(message);
        self.transport.write_all(&bytes)?;
        let response = self.receive_message()?;

        self.check_result_code(&response)?;
        Ok(())
    }

    /// Gets the RF channel currently in use by the reader.
    ///
    /// # Returns
    /// The current RF channel number (0-9, referred to ETSI EN 302 208 regulation)
    ///
    /// # Errors
    /// * `CommandError::Transport` - If a communication error occurs on the serial port
    /// * `CommandError::ReaderError` - If the reader returns a non-success result code
    /// * `CommandError::Message` - If the response does not contain the expected AVP
    pub fn cmd_get_rf_channel(&mut self) -> Result<u16, CommandError> {

        let avp_cmd_name = protocol::Avp::new(
            constants::AVP_COMMAND_NAME, 
            constants::CMD_GET_RF_CHANNEL.to_be_bytes().to_vec(),
        )?;
        let message = protocol::Message::new(
            self.message_id,
            vec![avp_cmd_name]
        )?;

        let bytes = Vec::<u8>::from(message);
        self.transport.write_all(&bytes)?;
        let response = self.receive_message()?;
        
        self.check_result_code(&response)?;

        // Find protocol avp in response
        let result_avp = response.avp_list.iter()
            .find(|avp| avp.attr_type == constants::AVP_RF_CHANNEL)
            .ok_or(CommandError::Message(protocol::MessageError::TooShort))?;

        Ok(u16::from_be_bytes(result_avp.value[0..2].try_into().unwrap()))
    }

    /// Sets the RF channel for the reader.
    ///
    /// # Arguments
    /// * `channel` - RF channel number (0-9, referred to ETSI EN 302 208 regulation)
    ///
    /// # Errors
    /// * `CommandError::Transport` - If a communication error occurs on the serial port
    /// * `CommandError::ReaderError` - If the reader returns a non-success result code
    pub fn cmd_set_rf_channel(&mut self, channel: u16) -> Result<(), CommandError> {

        let avp_cmd_name = protocol::Avp::new(
            constants::AVP_COMMAND_NAME, 
            constants::CMD_SET_RF_CHANNEL.to_be_bytes().to_vec(),
        )?;
        let avp_channel = protocol::Avp::new(
            constants::AVP_RF_CHANNEL,
            channel.to_be_bytes().to_vec(),
        )?;
        let message = protocol::Message::new(
            self.message_id,
            vec![avp_cmd_name, avp_channel]
        )?;

        let bytes = Vec::<u8>::from(message);
        self.transport.write_all(&bytes)?;
        let response = self.receive_message()?;

        self.check_result_code(&response)?;
        Ok(())        
    }

    /// Gets the current Listen Before Talk (LBT) mode setting.
    /// Only supported on ETSI EN 302 208 compatible readers.
    ///
    /// # Returns
    /// `true` if LBT is enabled, `false` if disabled
    ///
    /// # Errors
    /// * `CommandError::Transport` - If a communication error occurs on the serial port
    /// * `CommandError::ReaderError` - If the reader returns a non-success result code
    /// * `CommandError::Message` - If the response does not contain the expected AVP
    pub fn cmd_get_lbt_mode(&mut self) -> Result<bool, CommandError> {

        let avp_cmd_name = protocol::Avp::new(
            constants::AVP_COMMAND_NAME, 
            constants::CMD_SET_LBT_MODE.to_be_bytes().to_vec(),
        )?;
        let message = protocol::Message::new(
            self.message_id,
            vec![avp_cmd_name]
        )?;

        let bytes = Vec::<u8>::from(message);
        self.transport.write_all(&bytes)?;
        let response = self.receive_message()?;

        self.check_result_code(&response)?;

        // Find protocol avp in response
        let result_avp = response.avp_list.iter()
            .find(|avp| avp.attr_type == constants::AVP_BOOLEAN)
            .ok_or(CommandError::Message(protocol::MessageError::TooShort))?;

        let status = u16::from_be_bytes(result_avp.value[0..2].try_into().unwrap());
        Ok(status != 0) // status conversion to bool
    }

    /// Enables or disables the Listen Before Talk (LBT) capability.
    /// Only supported on ETSI EN 302 208 compatible readers.
    ///
    /// # Arguments
    /// * `enabled` - `true` to enable LBT, `false` to disable
    ///
    /// # Errors
    /// * `CommandError::Transport` - If a communication error occurs on the serial port
    /// * `CommandError::ReaderError` - If the reader returns a non-success result code
    pub fn cmd_set_lbt_mode(&mut self, enabled: bool) -> Result<(), CommandError> {

        let avp_cmd_name = protocol::Avp::new(
            constants::AVP_COMMAND_NAME, 
            constants::CMD_SET_LBT_MODE.to_be_bytes().to_vec(),
        )?;
        let avp_lbt_mode = protocol::Avp::new(
            constants::AVP_BOOLEAN,
            (enabled as u16).to_be_bytes().to_vec(),
        )?;
        let message = protocol::Message::new(
            self.message_id,
            vec![avp_cmd_name, avp_lbt_mode]
        )?;

        let bytes = Vec::<u8>::from(message);
        self.transport.write_all(&bytes)?;
        let response = self.receive_message()?;

        self.check_result_code(&response)?;
        Ok(())
    }

    /// Gets the RF regulation currently in use by the reader.
    ///
    /// # Returns
    /// The current [`constants::RFRegulation`] in use
    ///
    /// # Errors
    /// * `CommandError::Transport` - If a communication error occurs on the serial port
    /// * `CommandError::ReaderError` - If the reader returns a non-success result code
    /// * `CommandError::Message` - If the response does not contain the expected AVP
    pub fn cmd_get_rf_regulation(&mut self) -> Result<constants::RFRegulation, CommandError>{

        let avp_cmd_name = protocol::Avp::new(
            constants::AVP_COMMAND_NAME, 
            constants::CMD_GET_RF_REGULATION.to_be_bytes().to_vec(),
        )?;
        let message = protocol::Message::new(
            self.message_id,
            vec![avp_cmd_name]
        )?;

        let bytes = Vec::<u8>::from(message);
        self.transport.write_all(&bytes)?;
        let response = self.receive_message()?;

        self.check_result_code(&response)?;

        // Find protocol avp in response
        let result_avp = response.avp_list.iter()
            .find(|avp| avp.attr_type == constants::AVP_RF_REGULATION)
            .ok_or(CommandError::Message(protocol::MessageError::TooShort))?;

        let regulation_raw = u16::from_be_bytes(result_avp.value[0..2].try_into().unwrap());
        let regulation = constants::RFRegulation::try_from(regulation_raw)?;

        Ok(regulation)
    }

    /// Starts or stops the generation of a continuous RF wave.
    /// Used only for test and measurement purposes.
    ///
    /// # Arguments
    /// * `enabled` - `true` to start the RF wave, `false` to stop it
    ///
    /// # Errors
    /// * `CommandError::Transport` - If a communication error occurs on the serial port
    /// * `CommandError::ReaderError` - If the reader returns a non-success result code
    pub fn cmd_rf_on_off(&mut self, enabled: bool) -> Result<(), CommandError> {

        let avp_cmd_name = protocol::Avp::new(
            constants::AVP_COMMAND_NAME, 
            constants::CMD_RF_ON_OFF.to_be_bytes().to_vec(),
        )?;
        let avp_rf_mode = protocol::Avp::new(
            constants::AVP_RF_ON_OFF,
            (enabled as u16).to_be_bytes().to_vec(),
        )?;
        let message = protocol::Message::new(
            self.message_id,
            vec![avp_cmd_name, avp_rf_mode]
        )?;

        let bytes = Vec::<u8>::from(message);
        self.transport.write_all(&bytes)?;
        let response = self.receive_message()?;

        self.check_result_code(&response)?;
        Ok(())
    }

    /// Gets the firmware revision of the reader.
    ///
    /// # Returns
    /// A `String` containing the firmware revision
    ///
    /// # Errors
    /// * `CommandError::Transport` - If a communication error occurs on the serial port
    /// * `CommandError::ReaderError` - If the reader returns a non-success result code
    /// * `CommandError::Message` - If the response does not contain the expected AVP\
    pub fn cmd_get_firmware_release(&mut self) -> Result<String, CommandError> {

        let avp_cmd_name = protocol::Avp::new(
            constants::AVP_COMMAND_NAME, 
            constants::CMD_GET_FIRMWARE_RELEASE.to_be_bytes().to_vec(),
        )?;
        let message = protocol::Message::new(
            self.message_id,
            vec![avp_cmd_name]
        )?;

        let bytes = Vec::<u8>::from(message);
        self.transport.write_all(&bytes)?;
        let response = self.receive_message()?;

        self.check_result_code(&response)?;

        // Find protocol avp in response
        let result_avp = response.avp_list.iter()
            .find(|avp| avp.attr_type == constants::AVP_FW_RELEASE)
            .ok_or(CommandError::Message(protocol::MessageError::TooShort))?;

        let mut fwver_raw = result_avp.value.clone();
        fwver_raw.retain(|&b| b != 0);
        let fw_release = String::from_utf8(fwver_raw)
            .map_err(|_| CommandError::Message(protocol::MessageError::InvalidValue))?;

        Ok(fw_release)
    }

    /// Gets information about the reader (model and serial number).
    ///
    /// # Returns
    /// A `String` in the format `<reader name> <serial number>`
    ///
    /// # Errors
    /// * `CommandError::Transport` - If a communication error occurs on the serial port
    /// * `CommandError::ReaderError` - If the reader returns a non-success result code
    /// * `CommandError::Message` - If the response does not contain the expected AVP
    pub fn cmd_get_reader_info(&mut self) -> Result<String, CommandError> {
  
        let avp_cmd_name = protocol::Avp::new(
            constants::AVP_COMMAND_NAME, 
            constants::CMD_GET_READER_INFO.to_be_bytes().to_vec(),
        )?;
        let message = protocol::Message::new(
            self.message_id,
            vec![avp_cmd_name]
        )?;

        let bytes = Vec::<u8>::from(message);
        self.transport.write_all(&bytes)?;
        let response = self.receive_message()?;

        self.check_result_code(&response)?;

        // Find protocol avp in response
        let result_avp = response.avp_list.iter()
            .find(|avp| avp.attr_type == constants::AVP_READER_INFO)
            .ok_or(CommandError::Message(protocol::MessageError::TooShort))?;

        let mut info_raw = result_avp.value.clone();
        info_raw.retain(|&b| b != 0);
        let fw_info = String::from_utf8(info_raw)
            .map_err(|_| CommandError::Message(protocol::MessageError::InvalidValue))?;

        Ok(fw_info)  
    }

    /// Modifies the serial port settings of the reader.
    ///
    /// # Arguments
    /// * `baud_rate` - Baud rate value
    /// * `data_bits` - Number of data bits
    /// * `stop_bits` - Number of stop bits
    /// * `parity` - Parity setting
    /// * `flow_ctrl` - Flow control setting
    ///
    /// # Errors
    /// * `CommandError::Transport` - If a communication error occurs on the serial port
    /// * `CommandError::ReaderError` - If the reader returns a non-success result code
    pub fn cmd_set_rs232(
        &mut self, 
        baud_rate: u32, 
        data_bits: u32, 
        stop_bits: u32, 
        parity: constants::Parity, 
        flow_ctlr: constants::FlowCtrl
    ) -> Result<(), CommandError> {

        let avp_cmd_name = protocol::Avp::new(
            constants::AVP_COMMAND_NAME, 
            constants::CMD_SET_RS232.to_be_bytes().to_vec(),
        )?;
        let avp_baud_rate = protocol::Avp::new(
            constants::AVP_BAUD_RATE, 
            baud_rate.to_be_bytes().to_vec(),
        )?;
        let avp_data_bits = protocol::Avp::new(
            constants::AVP_DATA_BITS, 
            data_bits.to_be_bytes().to_vec(),
        )?;
        let avp_stop_bits = protocol::Avp::new(
            constants::AVP_STOP_BITS, 
            stop_bits.to_be_bytes().to_vec(),
        )?;
        let avp_parity = protocol::Avp::new(
            constants::AVP_PARITY, 
            (parity as u32).to_be_bytes().to_vec(),
        )?;
        let avp_flow_ctrl = protocol::Avp::new(
            constants::AVP_FLOW_CTRL, 
            (flow_ctlr as u32).to_be_bytes().to_vec(),
        )?;
        let message = protocol::Message::new(
            self.message_id,
            vec![avp_cmd_name, avp_baud_rate, avp_data_bits, avp_stop_bits, avp_parity, avp_flow_ctrl]
        )?;

        let bytes = Vec::<u8>::from(message);
        self.transport.write_all(&bytes)?;
        let response = self.receive_message()?;

        self.check_result_code(&response)?;
        Ok(())
    }

    /// Reads the current status of the I/O lines of the reader.
    ///
    /// # Returns
    /// A `u32` representing the I/O register status.
    /// Input lines are mapped on the least significant bits,
    /// output lines on the most significant bits.
    ///
    /// # Errors
    /// * `CommandError::Transport` - If a communication error occurs on the serial port
    /// * `CommandError::ReaderError` - If the reader returns a non-success result code
    /// * `CommandError::Message` - If the response does not contain the expected AVP
    pub fn cmd_get_io(&mut self) -> Result<u32, CommandError> {

        let avp_cmd_name = protocol::Avp::new(
            constants::AVP_COMMAND_NAME, 
            constants::CMD_GET_IO.to_be_bytes().to_vec(),
        )?;
        let message = protocol::Message::new(
            self.message_id,
            vec![avp_cmd_name]
        )?;

        let bytes = Vec::<u8>::from(message);
        self.transport.write_all(&bytes)?;
        let response = self.receive_message()?;

        self.check_result_code(&response)?;

        // Find protocol avp in response
        let result_avp = response.avp_list.iter()
            .find(|avp| avp.attr_type == constants::AVP_IO_REGISTER)
            .ok_or(CommandError::Message(protocol::MessageError::TooShort))?;

        Ok(u32::from_be_bytes(result_avp.value[0..4].try_into().unwrap()))
    }

    /// Sets the level of the output I/O lines of the reader.
    ///
    /// # Arguments
    /// * `io_register` - A `u32` bitmask representing the output lines to set
    ///
    /// # Errors
    /// * `CommandError::Transport` - If a communication error occurs on the serial port
    /// * `CommandError::ReaderError` - If the reader returns a non-success result code
    pub fn cmd_set_io(&mut self, io_register: u32) -> Result<(), CommandError> {

        let avp_cmd_name = protocol::Avp::new(
            constants::AVP_COMMAND_NAME, 
            constants::CMD_SET_IO.to_be_bytes().to_vec(),
        )?;
        let avp_set_io = protocol::Avp::new(
            constants::AVP_IO_REGISTER, 
            io_register.to_be_bytes().to_vec(),
        )?;
        let message = protocol::Message::new(
            self.message_id,
            vec![avp_cmd_name, avp_set_io],
        )?;
        
        let bytes = Vec::<u8>::from(message);
        self.transport.write_all(&bytes)?;
        let response = self.receive_message()?;

        self.check_result_code(&response)?;
        Ok(())        
    }

    /// Gets the current direction setting of the I/O lines.
    ///
    /// # Returns
    /// A `u32` bitmask where `0` = input, `1` = output for each bit
    ///
    /// # Errors
    /// * `CommandError::Transport` - If a communication error occurs on the serial port
    /// * `CommandError::ReaderError` - If the reader returns a non-success result code
    /// * `CommandError::Message` - If the response does not contain the expected AVP
    pub fn cmd_get_io_direction(&mut self) -> Result<u32, CommandError> {
    
        let avp_cmd_name = protocol::Avp::new(
            constants::AVP_COMMAND_NAME, 
            constants::CMD_GET_IO_DIRECTION.to_be_bytes().to_vec(),
        )?;
        let message = protocol::Message::new(
            self.message_id,
            vec![avp_cmd_name]
        )?;

        let bytes = Vec::<u8>::from(message);
        self.transport.write_all(&bytes)?;
        let response = self.receive_message()?;

        self.check_result_code(&response)?;

        // Find protocol avp in response
        let result_avp = response.avp_list.iter()
            .find(|avp| avp.attr_type == constants::AVP_IO_REGISTER)
            .ok_or(CommandError::Message(protocol::MessageError::TooShort))?;

        Ok(u32::from_be_bytes(result_avp.value[0..4].try_into().unwrap()))   
    }

    /// Sets the direction of the I/O lines (input or output).
    ///
    /// # Arguments
    /// * `io_register` - A `u32` bitmask where `0` = input, `1` = output for each bit
    ///
    /// # Errors
    /// * `CommandError::Transport` - If a communication error occurs on the serial port
    /// * `CommandError::ReaderError` - If the reader returns a non-success result code
    pub fn cmd_set_io_direction(&mut self, io_register: u32) -> Result<(), CommandError> {
    
        let avp_cmd_name = protocol::Avp::new(
            constants::AVP_COMMAND_NAME, 
            constants::CMD_SET_IO_DIRECTION.to_be_bytes().to_vec(),
        )?;
        let avp_set_io = protocol::Avp::new(
            constants::AVP_IO_REGISTER, 
            io_register.to_be_bytes().to_vec(),
        )?;
        let message = protocol::Message::new(
            self.message_id,
            vec![avp_cmd_name, avp_set_io],
        )?;
        
        let bytes = Vec::<u8>::from(message);
        self.transport.write_all(&bytes)?;
        let response = self.receive_message()?;

        self.check_result_code(&response)?;
        Ok(())       
    }

    /// Gets a configuration parameter for a logical source.
    ///
    /// # Arguments
    /// * `source_name` - Name of the source to configure (e.g. "Source_0")
    /// * `parameter` - The configuration parameter to read
    ///
    /// # Returns
    /// The value of the requested configuration parameter
    ///
    /// # Errors
    /// * `CommandError::Transport` - If a communication error occurs on the serial port
    /// * `CommandError::ReaderError` - If the reader returns a non-success result code
    /// * `CommandError::Message` - If the response does not contain the expected AVP
    pub fn cmd_get_source_config(&mut self, source_name: &str, param: constants::ConfigParameter)
    -> Result<u32, CommandError> {

        let avp_cmd_name = protocol::Avp::new(
            constants::AVP_COMMAND_NAME, 
            constants::CMD_GET_SOURCE_CONFIG.to_be_bytes().to_vec(),
        )?;
        let avp_source_name = protocol::Avp::new(
            constants::AVP_SOURCE_NAME, 
            Self::null_terminate(source_name),
        )?;
        let avp_config_param = protocol::Avp::new(
            constants::AVP_CONFIG_PARAMETER, 
            (param as u32).to_be_bytes().to_vec(),
        )?;
        let message = protocol::Message::new(
            self.message_id,
            vec![avp_cmd_name, avp_source_name, avp_config_param],
        )?;

        let bytes = Vec::<u8>::from(message);
        self.transport.write_all(&bytes)?;
        let response = self.receive_message()?;

        self.check_result_code(&response)?;

        // Find protocol avp in response
        let result_avp = response.avp_list.iter()
            .find(|avp| avp.attr_type == constants::AVP_CONFIG_VALUE)
            .ok_or(CommandError::Message(protocol::MessageError::TooShort))?;

        Ok(u32::from_be_bytes(result_avp.value[0..4].try_into().unwrap()))   
    }

    /// Sets a configuration parameter for a logical source.
    ///
    /// # Arguments
    /// * `source_name` - Name of the source to configure (e.g. "Source_0")
    /// * `parameter` - The configuration parameter to set
    /// * `value` - The value for the parameter
    ///
    /// # Errors
    /// * `CommandError::Transport` - If a communication error occurs on the serial port
    /// * `CommandError::ReaderError` - If the reader returns a non-success result code
    pub fn cmd_set_source_config(
        &mut self, 
        source_name: &str, 
        param: constants::ConfigParameter, 
        value: u32
    ) -> Result<(), CommandError> { 

        let avp_cmd_name = protocol::Avp::new(
            constants::AVP_COMMAND_NAME, 
            constants::CMD_SET_SOURCE_CONFIG.to_be_bytes().to_vec(),
        )?;
        let avp_source_name = protocol::Avp::new(
            constants::AVP_SOURCE_NAME, 
            Self::null_terminate(source_name),
        )?;
        let avp_config_param = protocol::Avp::new(
            constants::AVP_CONFIG_PARAMETER, 
            (param as u32).to_be_bytes().to_vec(),
        )?;
        let avp_config_value = protocol::Avp::new(
            constants::AVP_CONFIG_VALUE, 
            value.to_be_bytes().to_vec(),
        )?;
        let message = protocol::Message::new(
            self.message_id,
            vec![avp_cmd_name, avp_source_name, avp_config_param, avp_config_value],
        )?;
        
        let bytes = Vec::<u8>::from(message);
        self.transport.write_all(&bytes)?;
        let response = self.receive_message()?;

        self.check_result_code(&response)?;
        Ok(())   
    }

    /// Checks the quality of the antenna connection for a given read point.
    ///
    /// # Arguments
    /// * `read_point_name` - Name of the read point to check (e.g. "Ant0")
    ///
    /// # Returns
    /// The [`constants::ReadPointStatus`] of the antenna connection
    ///
    /// # Errors
    /// * `CommandError::Transport` - If a communication error occurs on the serial port
    /// * `CommandError::ReaderError` - If the reader returns a non-success result code
    /// * `CommandError::Message` - If the response does not contain the expected AVP
    pub fn cmd_check_read_point_status(&mut self, read_point_name: &str)
    -> Result<constants::ReadPointStatus, CommandError> {
        
        let avp_cmd_name = protocol::Avp::new(
            constants::AVP_COMMAND_NAME, 
            constants::CMD_CHECK_READ_POINT_STATUS.to_be_bytes().to_vec(),
        )?;
        let avp_read_point_name = protocol::Avp::new(
            constants::AVP_READ_POINT_NAME, 
            Self::null_terminate(read_point_name),
        )?;
        let message = protocol::Message::new(
            self.message_id,
            vec![avp_cmd_name, avp_read_point_name],
        )?;
        
        let bytes = Vec::<u8>::from(message);
        self.transport.write_all(&bytes)?;
        let response = self.receive_message()?;

        self.check_result_code(&response)?;

        // Find protocol avp in response
        let result_avp = response.avp_list.iter()
            .find(|avp| avp.attr_type == constants::AVP_READ_POINT_STATUS)
            .ok_or(CommandError::Message(protocol::MessageError::TooShort))?;
        
        let read_point_status = constants::ReadPointStatus::try_from(
            u32::from_be_bytes(result_avp.value[0..4].try_into().unwrap()))?;
        
        Ok(read_point_status)
    }

    /// Adds a read point (antenna) to a logical source.
    ///
    /// # Arguments
    /// * `source_name` - Name of the source (e.g. "Source_0")
    /// * `read_point_name` - Name of the read point to add (e.g. "Ant0")
    ///
    /// # Errors
    /// * `CommandError::Transport` - If a communication error occurs on the serial port
    /// * `CommandError::ReaderError` - If the reader returns a non-success result code
    /// 
    pub fn cmd_add_read_point_to_source(&mut self, source_name: &str, read_point_name: &str)
    -> Result<(), CommandError> {

        let avp_cmd_name = protocol::Avp::new(
            constants::AVP_COMMAND_NAME, 
            constants::CMD_ADD_READ_POINT_TO_SOURCE.to_be_bytes().to_vec(),
        )?;
        let avp_source_name = protocol::Avp::new(
            constants::AVP_SOURCE_NAME, 
            Self::null_terminate(source_name),
        )?;
        let avp_read_point_name = protocol::Avp::new(
            constants::AVP_READ_POINT_NAME, 
            Self::null_terminate(read_point_name),
        )?;
        let message = protocol::Message::new(
            self.message_id,
            vec![avp_cmd_name, avp_source_name, avp_read_point_name],
        )?;

        let bytes = Vec::<u8>::from(message);
        self.transport.write_all(&bytes)?;
        let response = self.receive_message()?;

        self.check_result_code(&response)?;
        Ok(())
    }

    /// Removes a read point (antenna) from a logical source.
    ///
    /// # Arguments
    /// * `source_name` - Name of the source (e.g. "Source_0")
    /// * `read_point_name` - Name of the read point to remove (e.g. "Ant0")
    ///
    /// # Errors
    /// * `CommandError::Transport` - If a communication error occurs on the serial port
    /// * `CommandError::ReaderError` - If the reader returns a non-success result code
    pub fn cmd_remove_read_point_from_source(&mut self, source_name: &str, read_point_name: &str)
    -> Result<(), CommandError> {

        let avp_cmd_name = protocol::Avp::new(
            constants::AVP_COMMAND_NAME, 
            constants::CMD_REMOVE_READ_POINT_FROM_SOURCE.to_be_bytes().to_vec(),
        )?;
        let avp_source_name = protocol::Avp::new(
            constants::AVP_SOURCE_NAME, 
            Self::null_terminate(source_name),
        )?;
        let avp_read_point_name = protocol::Avp::new(
            constants::AVP_READ_POINT_NAME, 
            Self::null_terminate(read_point_name),
        )?;
        let message = protocol::Message::new(
            self.message_id,
            vec![avp_cmd_name, avp_source_name, avp_read_point_name],
        )?;

        let bytes = Vec::<u8>::from(message);
        self.transport.write_all(&bytes)?;
        let response = self.receive_message()?;

        self.check_result_code(&response)?;
        Ok(())
    }

    /// Reads data from a Gen2 tag memory bank.
    ///
    /// # Arguments
    /// * `source_name` - Name of the source to use (e.g. "Source_0")
    /// * `tag_id` - The ID of the tag to read
    /// * `memory_bank` - The memory bank to read from
    /// * `tag_address` - The address where to start reading
    /// * `length` - Number of bytes to read (must be even)
    /// * `password` - Optional EPC access password
    ///
    /// # Returns
    /// A `Vec<u8>` containing the data read from the tag
    ///
    /// # Errors
    /// * `CommandError::Transport` - If a communication error occurs on the serial port
    /// * `CommandError::ReaderError` - If the reader returns a non-success result code
    /// * `CommandError::Message` - If the response does not contain the expected AVP
    pub fn cmd_read_tag_data_epc_c1g2(
        &mut self, 
        source_name: &str, 
        tag_id: &[u8], 
        memory_bank: constants::MemoryBank, 
        tag_address: u16, 
        length: u16, 
        passwd: Option<u32>
    ) -> Result<Vec<u8>, CommandError> {
        
        let avp_cmd_name = protocol::Avp::new(
            constants::AVP_COMMAND_NAME, 
            constants::CMD_READ_TAG_DATA_EPC_C1G2.to_be_bytes().to_vec(),
        )?;
        let avp_source_name = protocol::Avp::new(
            constants::AVP_SOURCE_NAME, 
            Self::null_terminate(source_name),
        )?;
        let avp_tag_id = protocol::Avp::new(
            constants::AVP_TAG_ID, 
            tag_id.to_vec(),
        )?;
        let avp_memory_bank = protocol::Avp::new(
            constants::AVP_MEMORY_BANK, 
            (memory_bank as u16).to_be_bytes().to_vec(),
        )?;
        let avp_tag_address = protocol::Avp::new(
            constants::AVP_TAG_ADDRESS, 
            (tag_address as u16).to_be_bytes().to_vec(),
        )?;
        let avp_length = protocol::Avp::new(
            constants::AVP_LENGTH, 
            (length as u16).to_be_bytes().to_vec(),
        )?;

        let mut avp_list = vec![
            avp_cmd_name, avp_source_name, avp_tag_id, avp_memory_bank, avp_tag_address, avp_length
            ];

        if let Some(pwd) = passwd {
            let avp_password = protocol::Avp::new(
                constants::AVP_G2_PASSWORD, 
                pwd.to_be_bytes().to_vec(),
            )?;
            avp_list.push(avp_password);
        }

        let message = protocol::Message::new(
            self.message_id, 
            avp_list
        )?;

        let bytes = Vec::<u8>::from(message);
        self.transport.write_all(&bytes)?;
        let response = self.receive_message()?;

        self.check_result_code(&response)?;

        // Find protocol avp in response
        let result_avp = response.avp_list.iter()
            .find(|avp| avp.attr_type == constants::AVP_TAG_VALUE)
            .ok_or(CommandError::Message(protocol::MessageError::TooShort))?;

        Ok(result_avp.value.clone())
    }

    /// Writes data to a Gen2 tag memory bank.
    ///
    /// # Arguments
    /// * `source_name` - Name of the source to use (e.g. "Source_0")
    /// * `tag_id` - The ID of the tag to write
    /// * `memory_bank` - The memory bank to write to
    /// * `tag_address` - The address where to start writing
    /// * `data` - The data to write (must be even number of bytes)
    /// * `passwd` - Optional EPC access password
    ///
    /// # Errors
    /// * `CommandError::Transport` - If a communication error occurs on the serial port
    /// * `CommandError::ReaderError` - If the reader returns a non-success result code
    pub fn cmd_write_tag_data_epc_c1g2(
        &mut self,
        source_name: &str,
        tag_id: &[u8],
        memory_bank: constants::MemoryBank,
        tag_address: u16,
        data: &[u8],
        passwd: Option<u32>,
    ) -> Result<(), CommandError> {
        let avp_cmd_name = protocol::Avp::new(
            constants::AVP_COMMAND_NAME, 
            constants::CMD_WRITE_TAG_DATA_EPC_C1G2.to_be_bytes().to_vec(),
        )?;
        let avp_source_name = protocol::Avp::new(
            constants::AVP_SOURCE_NAME, 
            Self::null_terminate(source_name),
        )?;
        let avp_tag_id = protocol::Avp::new(
            constants::AVP_TAG_ID, 
            tag_id.to_vec(),
        )?;
        let avp_memory_bank = protocol::Avp::new(
            constants::AVP_MEMORY_BANK, 
            (memory_bank as u16).to_be_bytes().to_vec(),
        )?;
        let avp_tag_address = protocol::Avp::new(
            constants::AVP_TAG_ADDRESS, 
            (tag_address as u16).to_be_bytes().to_vec(),
        )?;
        let avp_data = protocol::Avp::new(
            constants::AVP_TAG_VALUE, 
            data.to_vec(),
        )?;

        let mut avp_list = vec![
            avp_cmd_name, avp_source_name, avp_tag_id, avp_memory_bank, avp_tag_address, avp_data
            ];

        if let Some(pwd) = passwd {
            let avp_password = protocol::Avp::new(
                constants::AVP_G2_PASSWORD, 
                pwd.to_be_bytes().to_vec(),
            )?;
            avp_list.push(avp_password);
        }

        let message = protocol::Message::new(
            self.message_id, 
            avp_list
        )?;

        let bytes = Vec::<u8>::from(message);
        self.transport.write_all(&bytes)?;
        let response = self.receive_message()?;

        self.check_result_code(&response)?;
        Ok(())
    } 
}