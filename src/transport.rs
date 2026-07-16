use serialport::SerialPort;


#[derive(Debug)]
pub enum TransportError {
    Timeout,
    Disconnected,
    IoError(String)
}

pub trait Transport {
    fn read_exact(&mut self, buf: &mut[u8]) -> Result<(), TransportError>;
    fn write_all(&mut self, buf: &[u8]) -> Result<(), TransportError>;
}

pub struct SerialTransport {
    port: Box<dyn SerialPort>,
}

impl SerialTransport {

    pub fn new(port_name: &str, baud_rate: u32) -> Result<Self, TransportError> {
        let port = serialport::new(port_name, baud_rate)
            .open()
            .map_err(|e| TransportError::IoError(e.to_string()))?;
        
        Ok(SerialTransport { port })
    }
}

impl Transport for SerialTransport {

    fn read_exact(&mut self, buf: &mut [u8]) -> Result<(), TransportError> {
        self.port.read_exact(buf)
        .map_err(|e| TransportError::IoError(e.to_string()))
    }

    fn write_all(&mut self, buf: &[u8]) -> Result<(), TransportError> {
        self.port.write_all(buf)
        .map_err(|e| TransportError::IoError(e.to_string()))
    }
}