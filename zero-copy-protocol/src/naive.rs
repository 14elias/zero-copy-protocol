#[derive(Debug)]
pub struct Packet {
    pub version: u8,
    pub message_type: u8,
    pub payload_length: u16,
    pub payload: Vec<u8>,
}

#[derive(Debug)]
pub enum ParseError {
    InvalidPacket,
}

pub fn parse_packet(input: &[u8]) -> Result<Packet, ParseError> {
    if input.len() < 8 {
        return Err(ParseError::InvalidPacket);
    }

    if &input[0..4] != b"RUST" {
        return Err(ParseError::InvalidPacket);
    }

    let version = input[4];

    if version != 1 {
        return Err(ParseError::InvalidPacket);
    }

    let message_type = input[5];

    let payload_length =
        u16::from_be_bytes([input[6], input[7]]);

    let expected_length = 8 + payload_length as usize;

    if input.len() != expected_length {
        return Err(ParseError::InvalidPacket);
    }

    let payload = input[8..].to_vec();

    Ok(Packet {
        version,
        message_type,
        payload_length,
        payload,
    })
}