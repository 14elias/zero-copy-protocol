
use bytes::Bytes;

#[derive(Debug)]
pub struct Packet {
    pub version: u8,
    pub message_type: u8,
    pub payload_length: u16,
    pub payload: Bytes,
}

#[derive(Debug)]
pub enum ParseError {
    InvalidPacket,
}

pub fn parse_packet(input: &Bytes) -> Result<Packet, ParseError> {
    // 1. Check minimum header size
    if input.len() < 8 {
        return Err(ParseError::InvalidPacket);
    }

    // 2. Validate magic bytes
    if &input[0..4] != b"RUST" {
        return Err(ParseError::InvalidPacket);
    }

    // 3. Read version
    let version = input[4];

    if version != 1 {
        return Err(ParseError::InvalidPacket);
    }

    // 4. Read message type
    let message_type = input[5];

    // 5. Read payload length (big-endian)
    let payload_length =
        u16::from_be_bytes([input[6], input[7]]);

    // 6. Calculate expected packet size
    let expected_length = 8 + payload_length as usize;

    // 7. Require exactly one complete packet
    if input.len() != expected_length {
        return Err(ParseError::InvalidPacket);
    }

    // 8. Create a Bytes slice of the payload
    // This shares the original allocation; it does not copy payload bytes.
    let payload = input.slice(8..expected_length);

    // 9. Return parsed packet
    Ok(Packet {
        version,
        message_type,
        payload_length,
        payload,
    })
}
