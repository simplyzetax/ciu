//! Each packet starts with one byte identifying the message type.
//! The remaining bytes hold the value:
//!
//! - Throttle: `[1, amount]`
//! - ABS: `[2, mode]`, where 1 = Street and 2 = Supermoto
//! - RPM: `[3, low_byte, high_byte]` (little-endian)
//! - Clutch: `[4, engaged]`, where 0 = false and 1 = true

use serde::{Deserialize, Serialize};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum ProtocolError {
    #[error("output buffer is too small")]
    BufferTooSmall,

    #[error("invalid message type: {0}")]
    InvalidMessageType(u8),

    #[error("invalid payload")]
    InvalidPayload,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[repr(u8)]
pub enum DeviceId {
    Bike = 1,
    Helmet = 2,
    Goggle = 3,
}

/// The numeric IDs sent in the first byte of a packet.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum MessageType {
    Throttle = 1,
    ABS = 2,
    Rpm = 3,
    Clutch = 4,
    KillSwitch = 5,
}

// TryFrom converts a byte into a MessageType, or returns an error for an unknown ID.
impl TryFrom<u8> for MessageType {
    type Error = ProtocolError;

    fn try_from(value: u8) -> Result<Self, ProtocolError> {
        match value {
            1 => Ok(MessageType::Throttle),
            2 => Ok(MessageType::ABS),
            3 => Ok(MessageType::Rpm),
            4 => Ok(MessageType::Clutch),
            _ => Err(ProtocolError::InvalidMessageType(value)),
        }
    }
}

impl MessageType {
    const fn encoded_len(self) -> usize {
        match self {
            MessageType::Rpm => 3,
            _ => 2,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum ABSMode {
    Street = 1,
    Supermoto = 2,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Throttle {
    pub amount: u8,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Rpm {
    pub amount: u16,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Clutch {
    pub engaged: bool,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct KillSwitch {
    pub engaged: bool,
}

/// A message contains both its type and the value to send.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Message {
    Throttle(Throttle),
    ABS(ABSMode),
    Rpm(Rpm),
    Clutch(Clutch),
    KillSwitch(KillSwitch),
}

impl Message {
    pub fn message_type(&self) -> MessageType {
        match self {
            Message::Throttle(_) => MessageType::Throttle,
            Message::ABS(_) => MessageType::ABS,
            Message::Rpm(_) => MessageType::Rpm,
            Message::Clutch(_) => MessageType::Clutch,
            Message::KillSwitch(_) => MessageType::KillSwitch,
        }
    }

    /// Writes the packet into the provided buffer and returns its length.
    pub fn encode(&self, buffer: &mut [u8]) -> Result<usize, ProtocolError> {
        let packet_length = self.message_type().encoded_len();

        // Check before writing so an error leaves the buffer unchanged.
        if buffer.len() < packet_length {
            return Err(ProtocolError::BufferTooSmall);
        }

        buffer[0] = self.message_type() as u8;
        match self {
            Message::Throttle(throttle) => buffer[1] = throttle.amount,
            Message::ABS(mode) => buffer[1] = *mode as u8,
            Message::Rpm(rpm) => {
                let bytes = rpm.amount.to_le_bytes();
                buffer[1] = bytes[0];
                buffer[2] = bytes[1];
            }
            Message::Clutch(clutch) => {
                buffer[1] = if clutch.engaged { 1 } else { 0 };
            }
            Message::KillSwitch(killswitch) => {
                buffer[1] = if killswitch.engaged { 1 } else { 0 };
            }
        }

        Ok(packet_length)
    }

    /// Reads exactly one packet. Missing or extra bytes are errors.
    pub fn decode(data: &[u8]) -> Result<Self, ProtocolError> {
        if data.is_empty() {
            return Err(ProtocolError::InvalidPayload);
        }

        // The ? returns early if the first byte is not a known message type.
        let message_type = MessageType::try_from(data[0])?;
        let expected_length = message_type.encoded_len();

        // Validate the length before accessing the value bytes below.
        if data.len() != expected_length {
            return Err(ProtocolError::InvalidPayload);
        }

        match message_type {
            MessageType::Throttle => {
                let throttle = Throttle { amount: data[1] };
                Ok(Message::Throttle(throttle))
            }
            MessageType::ABS => {
                let mode = match data[1] {
                    1 => ABSMode::Street,
                    2 => ABSMode::Supermoto,
                    _ => return Err(ProtocolError::InvalidPayload),
                };
                Ok(Message::ABS(mode))
            }
            MessageType::Rpm => {
                let amount = u16::from_le_bytes([data[1], data[2]]);
                Ok(Message::Rpm(Rpm { amount }))
            }
            MessageType::Clutch => {
                let engaged = match data[1] {
                    0 => false,
                    1 => true,
                    _ => return Err(ProtocolError::InvalidPayload),
                };
                Ok(Message::Clutch(Clutch { engaged }))
            }
            MessageType::KillSwitch => {
                let engaged = match data[1] {
                    0 => false,
                    1 => true,
                    _ => return Err(ProtocolError::InvalidPayload),
                };
                Ok(Message::KillSwitch(KillSwitch { engaged }))
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn wire_format_and_buffer_boundaries() {
        let cases: &[(Message, &[u8])] = &[
            (Message::Throttle(Throttle { amount: 255 }), &[1, 255]),
            (Message::ABS(ABSMode::Street), &[2, 1]),
            (Message::ABS(ABSMode::Supermoto), &[2, 2]),
            (Message::Rpm(Rpm { amount: 0x1234 }), &[3, 0x34, 0x12]),
            (Message::Clutch(Clutch { engaged: false }), &[4, 0]),
            (Message::Clutch(Clutch { engaged: true }), &[4, 1]),
        ];

        for &(message, wire) in cases {
            let mut out = [0xAA; 4];
            let len = message.encode(&mut out).unwrap();
            assert_eq!(&out[..len], wire);
            assert!(out[len..].iter().all(|&byte| byte == 0xAA));
            assert_eq!(Message::decode(wire).unwrap(), message);
            assert_eq!(message.encode(&mut out[..wire.len()]).unwrap(), wire.len());

            for len in 0..wire.len() {
                let mut out = [0xAA; 3];
                assert!(matches!(
                    message.encode(&mut out[..len]),
                    Err(ProtocolError::BufferTooSmall)
                ));
                assert_eq!(out, [0xAA; 3]);
                assert!(matches!(
                    Message::decode(&wire[..len]),
                    Err(ProtocolError::InvalidPayload)
                ));
            }

            let mut padded = [0; 4];
            padded[..wire.len()].copy_from_slice(wire);
            assert!(matches!(
                Message::decode(&padded[..wire.len() + 1]),
                Err(ProtocolError::InvalidPayload)
            ));
        }
    }

    #[test]
    fn rejects_unknown_types_and_invalid_values() {
        for kind in [0, 5, 255] {
            assert!(matches!(
                Message::decode(&[kind, 0]),
                Err(ProtocolError::InvalidMessageType(value)) if value == kind
            ));
        }
        for wire in [[2, 0], [2, 3], [2, 255], [4, 2], [4, 255]] {
            assert!(matches!(
                Message::decode(&wire),
                Err(ProtocolError::InvalidPayload)
            ));
        }
    }
}
