
use bytes::Bytes;
use nom::{
    bytes::complete::tag,
    number::complete::{be_u8, be_u16},
    sequence::tuple,
    IResult,
};

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

// nom parses and validates the header fields.
fn parse_header(
    input: &[u8],
) -> IResult<&[u8], (u8, u8, u16)> {
    let (input, _) = tag(&b"RUST"[..])(input)?;
    let (input, version) = be_u8(input)?;
    let (input, message_type) = be_u8(input)?;
    let (input, payload_length) = be_u16(input)?;

    Ok((input, (version, message_type, payload_length)))
}

pub fn parse_packet(input: &Bytes) -> Result<Packet, ParseError> {
    // Parse the header using nom
    let (remaining, (version, message_type, payload_length)) =
        parse_header(input.as_ref())
            .map_err(|_| ParseError::InvalidPacket)?;

    // Only support protocol version 1
    if version != 1 {
        return Err(ParseError::InvalidPacket);
    }

    // Calculate the total expected packet length
    let expected_length = 8 + payload_length as usize;

    // Require exactly one complete packet
    if input.len() != expected_length {
        return Err(ParseError::InvalidPacket);
    }

    // Confirm nom consumed exactly the 8-byte header
    if remaining.len() != payload_length as usize {
        return Err(ParseError::InvalidPacket);
    }

    // Extract payload as a shared Bytes slice
    let payload = input.slice(8..expected_length);

    Ok(Packet {
        version,
        message_type,
        payload_length,
        payload,
    })
}
