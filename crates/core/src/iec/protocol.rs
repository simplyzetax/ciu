//! Binary protocol shared by all CIU transports.
//!
//! The protocol is transport-independent:
//!
//! - ESP-NOW sends encoded `Message`s.
//! - The physical pairing wire sends encoded `Message`s.
//!
//! Every packet starts with one byte identifying the message type.
//!
//! - Throttle:   `[1, amount]`
//! - ABS:        `[2, mode]`
//! - RPM:        `[3, low_byte, high_byte]`
//! - Clutch:     `[4, engaged]`
//! - KillSwitch: `[5, engaged]`
//! - Pairing:    `[6, pairing_type, device, token...]`
//!
//! Pairing types:
//!
//! - Hello: `1`
//! - Ack:   `2`
//!
//! Tokens are encoded as little-endian `u64`.

use serde::{Deserialize, Serialize};
use thiserror::Error;

#[derive(Debug, Error, PartialEq, Eq)]
pub enum ProtocolError {
    #[error("output buffer is too small")]
    BufferTooSmall,

    #[error("invalid message type: {0}")]
    InvalidMessageType(u8),

    #[error("invalid pairing message type: {0}")]
    InvalidPairingMessageType(u8),

    #[error("invalid device ID: {0}")]
    InvalidDeviceId(u8),

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

impl TryFrom<u8> for DeviceId {
    type Error = ProtocolError;

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value {
            1 => Ok(Self::Bike),
            2 => Ok(Self::Helmet),
            3 => Ok(Self::Goggle),
            _ => Err(ProtocolError::InvalidDeviceId(value)),
        }
    }
}

/// Numeric ID stored in the first byte of every packet.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum MessageType {
    Throttle = 1,
    ABS = 2,
    Rpm = 3,
    Clutch = 4,
    KillSwitch = 5,
    Pairing = 6,
}

impl TryFrom<u8> for MessageType {
    type Error = ProtocolError;

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value {
            1 => Ok(Self::Throttle),
            2 => Ok(Self::ABS),
            3 => Ok(Self::Rpm),
            4 => Ok(Self::Clutch),
            5 => Ok(Self::KillSwitch),
            6 => Ok(Self::Pairing),
            _ => Err(ProtocolError::InvalidMessageType(value)),
        }
    }
}

impl MessageType {
    const fn encoded_len(self) -> usize {
        match self {
            Self::Throttle => 2,
            Self::ABS => 2,
            Self::Rpm => 3,
            Self::Clutch => 2,
            Self::KillSwitch => 2,

            // message type + pairing type + device + u64 token
            Self::Pairing => 1 + 1 + 1 + 8,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum ABSMode {
    Street = 1,
    Supermoto = 2,
}

impl TryFrom<u8> for ABSMode {
    type Error = ProtocolError;

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value {
            1 => Ok(Self::Street),
            2 => Ok(Self::Supermoto),
            _ => Err(ProtocolError::InvalidPayload),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Throttle {
    pub amount: u8,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Rpm {
    pub amount: u16,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Clutch {
    pub engaged: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct KillSwitch {
    pub engaged: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PairHello {
    pub device: DeviceId,
    pub token: u64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PairAck {
    pub device: DeviceId,
    pub token: u64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum PairingMessageType {
    Hello = 1,
    Ack = 2,
}

impl TryFrom<u8> for PairingMessageType {
    type Error = ProtocolError;

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value {
            1 => Ok(Self::Hello),
            2 => Ok(Self::Ack),
            _ => Err(ProtocolError::InvalidPairingMessageType(value)),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PairingMessage {
    Hello(PairHello),
    Ack(PairAck),
}

/// A logical CIU protocol message.
///
/// This type is transport-independent. It can be encoded and sent through
/// ESP-NOW, the physical pairing wire, or any future transport.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Message {
    Throttle(Throttle),
    ABS(ABSMode),
    Rpm(Rpm),
    Clutch(Clutch),
    KillSwitch(KillSwitch),
    Pairing(PairingMessage),
}

impl Message {
    pub fn message_type(&self) -> MessageType {
        match self {
            Self::Throttle(_) => MessageType::Throttle,
            Self::ABS(_) => MessageType::ABS,
            Self::Rpm(_) => MessageType::Rpm,
            Self::Clutch(_) => MessageType::Clutch,
            Self::KillSwitch(_) => MessageType::KillSwitch,
            Self::Pairing(_) => MessageType::Pairing,
        }
    }

    /// Writes the message into the provided buffer and returns its encoded length.
    pub fn encode(&self, buffer: &mut [u8]) -> Result<usize, ProtocolError> {
        let packet_length = self.message_type().encoded_len();

        if buffer.len() < packet_length {
            return Err(ProtocolError::BufferTooSmall);
        }

        buffer[0] = self.message_type() as u8;

        match self {
            Self::Throttle(throttle) => {
                buffer[1] = throttle.amount;
            }

            Self::ABS(mode) => {
                buffer[1] = *mode as u8;
            }

            Self::Rpm(rpm) => {
                buffer[1..3].copy_from_slice(&rpm.amount.to_le_bytes());
            }

            Self::Clutch(clutch) => {
                buffer[1] = u8::from(clutch.engaged);
            }

            Self::KillSwitch(kill_switch) => {
                buffer[1] = u8::from(kill_switch.engaged);
            }

            Self::Pairing(pairing) => {
                let (pairing_type, device, token) = match pairing {
                    PairingMessage::Hello(message) => {
                        (PairingMessageType::Hello, message.device, message.token)
                    }

                    PairingMessage::Ack(message) => {
                        (PairingMessageType::Ack, message.device, message.token)
                    }
                };

                buffer[1] = pairing_type as u8;
                buffer[2] = device as u8;
                buffer[3..11].copy_from_slice(&token.to_le_bytes());
            }
        }

        Ok(packet_length)
    }

    /// Reads exactly one encoded message.
    ///
    /// Missing or extra bytes are rejected.
    pub fn decode(data: &[u8]) -> Result<Self, ProtocolError> {
        if data.is_empty() {
            return Err(ProtocolError::InvalidPayload);
        }

        let message_type = MessageType::try_from(data[0])?;
        let expected_length = message_type.encoded_len();

        if data.len() != expected_length {
            return Err(ProtocolError::InvalidPayload);
        }

        match message_type {
            MessageType::Throttle => Ok(Self::Throttle(Throttle { amount: data[1] })),

            MessageType::ABS => Ok(Self::ABS(ABSMode::try_from(data[1])?)),

            MessageType::Rpm => Ok(Self::Rpm(Rpm {
                amount: u16::from_le_bytes([data[1], data[2]]),
            })),

            MessageType::Clutch => Ok(Self::Clutch(Clutch {
                engaged: decode_bool(data[1])?,
            })),

            MessageType::KillSwitch => Ok(Self::KillSwitch(KillSwitch {
                engaged: decode_bool(data[1])?,
            })),

            MessageType::Pairing => {
                let pairing_type = PairingMessageType::try_from(data[1])?;
                let device = DeviceId::try_from(data[2])?;

                let token = u64::from_le_bytes(
                    data[3..11]
                        .try_into()
                        .map_err(|_| ProtocolError::InvalidPayload)?,
                );

                let pairing = match pairing_type {
                    PairingMessageType::Hello => PairingMessage::Hello(PairHello { device, token }),

                    PairingMessageType::Ack => PairingMessage::Ack(PairAck { device, token }),
                };

                Ok(Self::Pairing(pairing))
            }
        }
    }
}

fn decode_bool(value: u8) -> Result<bool, ProtocolError> {
    match value {
        0 => Ok(false),
        1 => Ok(true),
        _ => Err(ProtocolError::InvalidPayload),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn wire_format_and_round_trip() {
        let cases: &[(Message, &[u8])] = &[
            (Message::Throttle(Throttle { amount: 255 }), &[1, 255]),
            (Message::ABS(ABSMode::Street), &[2, 1]),
            (Message::ABS(ABSMode::Supermoto), &[2, 2]),
            (Message::Rpm(Rpm { amount: 0x1234 }), &[3, 0x34, 0x12]),
            (Message::Clutch(Clutch { engaged: false }), &[4, 0]),
            (Message::Clutch(Clutch { engaged: true }), &[4, 1]),
            (Message::KillSwitch(KillSwitch { engaged: false }), &[5, 0]),
            (Message::KillSwitch(KillSwitch { engaged: true }), &[5, 1]),
            (
                Message::Pairing(PairingMessage::Hello(PairHello {
                    device: DeviceId::Helmet,
                    token: 0x1122_3344_5566_7788,
                })),
                &[6, 1, 2, 0x88, 0x77, 0x66, 0x55, 0x44, 0x33, 0x22, 0x11],
            ),
            (
                Message::Pairing(PairingMessage::Ack(PairAck {
                    device: DeviceId::Goggle,
                    token: 0x8877_6655_4433_2211,
                })),
                &[6, 2, 3, 0x11, 0x22, 0x33, 0x44, 0x55, 0x66, 0x77, 0x88],
            ),
        ];

        for &(message, wire) in cases {
            let mut out = [0xAA; 32];

            let len = message.encode(&mut out).unwrap();

            assert_eq!(&out[..len], wire);
            assert_eq!(Message::decode(wire).unwrap(), message);

            assert!(out[len..].iter().all(|&byte| byte == 0xAA));
        }
    }

    #[test]
    fn buffer_boundaries() {
        let messages = [
            Message::Throttle(Throttle { amount: 255 }),
            Message::Rpm(Rpm { amount: 5000 }),
            Message::Pairing(PairingMessage::Hello(PairHello {
                device: DeviceId::Helmet,
                token: 123,
            })),
        ];

        for message in messages {
            let required = message.message_type().encoded_len();

            for len in 0..required {
                let mut out = [0xAA; 32];

                assert_eq!(
                    message.encode(&mut out[..len]),
                    Err(ProtocolError::BufferTooSmall),
                );

                assert!(out.iter().all(|&byte| byte == 0xAA));
            }
        }
    }

    #[test]
    fn rejects_incorrect_packet_lengths() {
        let message = Message::Pairing(PairingMessage::Hello(PairHello {
            device: DeviceId::Helmet,
            token: 123,
        }));

        let mut buffer = [0; 32];
        let len = message.encode(&mut buffer).unwrap();

        for short_len in 0..len {
            assert_eq!(
                Message::decode(&buffer[..short_len]),
                Err(ProtocolError::InvalidPayload),
            );
        }

        buffer[len] = 0;

        assert_eq!(
            Message::decode(&buffer[..len + 1]),
            Err(ProtocolError::InvalidPayload),
        );
    }

    #[test]
    fn rejects_unknown_message_types() {
        for kind in [0, 7, 100, 255] {
            assert_eq!(
                Message::decode(&[kind, 0]),
                Err(ProtocolError::InvalidMessageType(kind)),
            );
        }
    }

    #[test]
    fn rejects_invalid_values() {
        for wire in [&[2, 0][..], &[2, 3][..], &[4, 2][..], &[5, 2][..]] {
            assert_eq!(Message::decode(wire), Err(ProtocolError::InvalidPayload),);
        }
    }

    #[test]
    fn rejects_invalid_pairing_type() {
        let wire = [6, 99, DeviceId::Helmet as u8, 0, 0, 0, 0, 0, 0, 0, 0];

        assert_eq!(
            Message::decode(&wire),
            Err(ProtocolError::InvalidPairingMessageType(99)),
        );
    }

    #[test]
    fn rejects_invalid_device_id() {
        let wire = [
            6,
            PairingMessageType::Hello as u8,
            99,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
        ];

        assert_eq!(
            Message::decode(&wire),
            Err(ProtocolError::InvalidDeviceId(99)),
        );
    }
}
