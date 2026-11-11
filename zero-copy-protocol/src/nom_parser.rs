
use nom::{
    bytes::complete::{tag, take},
    combinator::all_consuming,
    number::complete::{be_u8, be_u16},
    IResult,
};

#[derive(Debug)]
pub struct Packet<'a> {
    pub version: u8,
    pub message_type: u8,
    pub payload_length: u16,
    pub payload: &'a [u8],
}

#[derive(Debug)]
pub enum ParseError {
    InvalidPacket,
}

fn packet_parser(input: &[u8]) -> IResult<&[u8], Packet<'_>> {
    // Validate magic bytes
    let (input, _) = tag(&b"RUST"[..])(input)?;

    // Read version
    let (input, version) = be_u8(input)?;

    if version != 1 {
        return Err(nom::Err::Failure(
            nom::error::Error::new(
                input,
                nom::error::ErrorKind::Verify,
            ),
        ));
    }

    // Read message type
    let (input, message_type) = be_u8(input)?;

    // Read payload length as big-endian u16
    let (input, payload_length) = be_u16(input)?;

    // Read the declared number of payload bytes
    let (input, payload) = take(payload_length)(input)?;

    Ok((
        input,
        Packet {
            version,
            message_type,
            payload_length,
            payload,
        },
    ))
}

pub fn parse_packet(input: &[u8]) -> Result<Packet<'_>, ParseError> {
    // Require exactly one complete packet:
    // no incomplete data and no trailing bytes.
    match all_consuming(packet_parser)(input) {
        Ok((_, packet)) => Ok(packet),
        Err(_) => Err(ParseError::InvalidPacket),
    }
}
