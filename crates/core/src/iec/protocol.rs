//! Binary protocol shared by CIU devices.
//!
//! Normal messages are transport-independent and may be sent over:
//!
//! - ESP-NOW
//! - the physical wire
//!
//! Pairing messages are different: they may ONLY be sent over the physical
//! wire. This is enforced by keeping pairing out of `Message` entirely.
//!
//! # Normal messages
//!
//! - Throttle:   `[1, amount]`
//! - ABS:        `[2, mode]`
//! - RPM:        `[3, low_byte, high_byte]`
//! - Clutch:     `[4, engaged]`
//! - KillSwitch: `[5, engaged]`
//!
//! # Wire-only pairing messages
//!
//! - PairHello: `[6, 1, device, mac...]`
//! - PairAck:   `[6, 2, device, mac...]`

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

/// Numeric ID stored in the first byte of a packet.
///
/// Pairing is included here because it exists on the physical wire format,
/// but it is deliberately not a variant of `Message`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum MessageType {
    Throttle = 1,
    ABS = 2,
    Rpm = 3,
    Clutch = 4,
    KillSwitch = 5,
    Pairing = 6,
    Ping = 7,
    Pong = 8,
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
            7 => Ok(Self::Ping),
            8 => Ok(Self::Pong),
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

            // type + pairing type + device + MAC
            Self::Pairing => 1 + 1 + 1 + 6,

            // type + u16 sequence
            Self::Ping => 3,
            Self::Pong => 3,
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
pub struct Ping {
    pub sequence: u16,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Pong {
    pub sequence: u16,
}

/// A CIU message delivered to application code.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ApplicationMessage {
    Throttle(Throttle),
    ABS(ABSMode),
    Rpm(Rpm),
    Clutch(Clutch),
    KillSwitch(KillSwitch),
}

/// A normal CIU message.
///
/// These messages may be transported over both ESP-NOW and the physical wire.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Message {
    Throttle(Throttle),
    ABS(ABSMode),
    Rpm(Rpm),
    Clutch(Clutch),
    KillSwitch(KillSwitch),
    Ping(Ping),
    Pong(Pong),
}
impl Message {
    pub fn message_type(&self) -> MessageType {
        match self {
            Self::Throttle(_) => MessageType::Throttle,
            Self::ABS(_) => MessageType::ABS,
            Self::Rpm(_) => MessageType::Rpm,
            Self::Clutch(_) => MessageType::Clutch,
            Self::KillSwitch(_) => MessageType::KillSwitch,
            Self::Ping(_) => MessageType::Ping,
            Self::Pong(_) => MessageType::Pong,
        }
    }

    /// Writes a normal message into the provided buffer.
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

            Self::Ping(ping) => {
                buffer[1..3].copy_from_slice(&ping.sequence.to_le_bytes());
            }

            Self::Pong(pong) => {
                buffer[1..3].copy_from_slice(&pong.sequence.to_le_bytes());
            }
        }

        Ok(packet_length)
    }

    /// Decodes a normal message.
    ///
    /// Pairing packets are rejected because pairing is wire-only.
    pub fn decode(data: &[u8]) -> Result<Self, ProtocolError> {
        if data.is_empty() {
            return Err(ProtocolError::InvalidPayload);
        }

        let message_type = MessageType::try_from(data[0])?;

        if message_type == MessageType::Pairing {
            return Err(ProtocolError::InvalidMessageType(
                MessageType::Pairing as u8,
            ));
        }

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

            MessageType::Ping => Ok(Self::Ping(Ping {
                sequence: u16::from_le_bytes([data[1], data[2]]),
            })),

            MessageType::Pong => Ok(Self::Pong(Pong {
                sequence: u16::from_le_bytes([data[1], data[2]]),
            })),

            MessageType::Pairing => unreachable!(),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PairHello {
    pub device: DeviceId,
    pub mac: [u8; 6],
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PairAck {
    pub device: DeviceId,
    pub mac: [u8; 6],
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

/// Pairing messages exist only on the physical wire.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PairingMessage {
    Hello(PairHello),
    Ack(PairAck),
}

/// Anything that can travel over the physical wire.
///
/// The wire supports both normal messages and the wire-only pairing protocol.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WireMessage {
    Message(Message),
    Pairing(PairingMessage),
}

impl From<Message> for WireMessage {
    fn from(message: Message) -> Self {
        Self::Message(message)
    }
}

impl WireMessage {
    pub fn encode(&self, buffer: &mut [u8]) -> Result<usize, ProtocolError> {
        match self {
            Self::Message(message) => message.encode(buffer),

            Self::Pairing(pairing) => {
                let packet_length = MessageType::Pairing.encoded_len();

                if buffer.len() < packet_length {
                    return Err(ProtocolError::BufferTooSmall);
                }

                buffer[0] = MessageType::Pairing as u8;

                let (pairing_type, device, mac) = match pairing {
                    PairingMessage::Hello(message) => {
                        (PairingMessageType::Hello, message.device, message.mac)
                    }

                    PairingMessage::Ack(message) => {
                        (PairingMessageType::Ack, message.device, message.mac)
                    }
                };

                buffer[1] = pairing_type as u8;
                buffer[2] = device as u8;
                buffer[3..9].copy_from_slice(&mac);

                Ok(packet_length)
            }
        }
    }

    pub fn decode(data: &[u8]) -> Result<Self, ProtocolError> {
        if data.is_empty() {
            return Err(ProtocolError::InvalidPayload);
        }

        let message_type = MessageType::try_from(data[0])?;

        if message_type != MessageType::Pairing {
            return Ok(Self::Message(Message::decode(data)?));
        }

        let expected_length = MessageType::Pairing.encoded_len();

        if data.len() != expected_length {
            return Err(ProtocolError::InvalidPayload);
        }

        let pairing_type = PairingMessageType::try_from(data[1])?;
        let device = DeviceId::try_from(data[2])?;

        let mac: [u8; 6] = data[3..9]
            .try_into()
            .map_err(|_| ProtocolError::InvalidPayload)?;

        let pairing = match pairing_type {
            PairingMessageType::Hello => PairingMessage::Hello(PairHello { device, mac }),

            PairingMessageType::Ack => PairingMessage::Ack(PairAck { device, mac }),
        };

        Ok(Self::Pairing(pairing))
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
    fn normal_messages_round_trip() {
        let cases: &[(Message, &[u8])] = &[
            (Message::Throttle(Throttle { amount: 255 }), &[1, 255]),
            (Message::ABS(ABSMode::Street), &[2, 1]),
            (Message::ABS(ABSMode::Supermoto), &[2, 2]),
            (Message::Rpm(Rpm { amount: 0x1234 }), &[3, 0x34, 0x12]),
            (Message::Clutch(Clutch { engaged: false }), &[4, 0]),
            (Message::Clutch(Clutch { engaged: true }), &[4, 1]),
            (Message::KillSwitch(KillSwitch { engaged: false }), &[5, 0]),
            (Message::KillSwitch(KillSwitch { engaged: true }), &[5, 1]),
        ];

        for &(message, wire) in cases {
            let mut buffer = [0xAA; 32];

            let len = message.encode(&mut buffer).unwrap();

            assert_eq!(&buffer[..len], wire);
            assert_eq!(Message::decode(wire).unwrap(), message);

            assert!(buffer[len..].iter().all(|&byte| byte == 0xAA));
        }
    }

    #[test]
    fn normal_messages_can_travel_over_wire() {
        let message = Message::Rpm(Rpm { amount: 5000 });

        let wire_message = WireMessage::from(message);

        let mut buffer = [0; 32];
        let len = wire_message.encode(&mut buffer).unwrap();

        assert_eq!(
            WireMessage::decode(&buffer[..len]).unwrap(),
            WireMessage::Message(message),
        );
    }

    #[test]
    fn pair_hello_round_trip() {
        let message = WireMessage::Pairing(PairingMessage::Hello(PairHello {
            device: DeviceId::Helmet,
            mac: [0xAA, 0xBB, 0xCC, 0xDD, 0xEE, 0xFF],
        }));

        let expected = [6, 1, 2, 0xAA, 0xBB, 0xCC, 0xDD, 0xEE, 0xFF];

        let mut buffer = [0; 32];
        let len = message.encode(&mut buffer).unwrap();

        assert_eq!(&buffer[..len], &expected);

        assert_eq!(WireMessage::decode(&buffer[..len]).unwrap(), message,);
    }

    #[test]
    fn pair_ack_round_trip() {
        let message = WireMessage::Pairing(PairingMessage::Ack(PairAck {
            device: DeviceId::Goggle,
            mac: [1, 2, 3, 4, 5, 6],
        }));

        let mut buffer = [0; 32];
        let len = message.encode(&mut buffer).unwrap();

        assert_eq!(WireMessage::decode(&buffer[..len]).unwrap(), message,);
    }

    #[test]
    fn normal_protocol_rejects_pairing_packets() {
        let wire = [6, 1, DeviceId::Helmet as u8, 1, 2, 3, 4, 5, 6];

        assert_eq!(
            Message::decode(&wire),
            Err(ProtocolError::InvalidMessageType(6)),
        );
    }

    #[test]
    fn rejects_invalid_pairing_type() {
        let wire = [6, 99, DeviceId::Helmet as u8, 1, 2, 3, 4, 5, 6];

        assert_eq!(
            WireMessage::decode(&wire),
            Err(ProtocolError::InvalidPairingMessageType(99)),
        );
    }

    #[test]
    fn rejects_invalid_device_id() {
        let wire = [6, PairingMessageType::Hello as u8, 99, 1, 2, 3, 4, 5, 6];

        assert_eq!(
            WireMessage::decode(&wire),
            Err(ProtocolError::InvalidDeviceId(99)),
        );
    }

    #[test]
    fn rejects_invalid_values() {
        for wire in [&[2, 0][..], &[2, 3][..], &[4, 2][..], &[5, 2][..]] {
            assert_eq!(Message::decode(wire), Err(ProtocolError::InvalidPayload),);
        }
    }

    #[test]
    fn buffer_boundaries() {
        let message = WireMessage::Pairing(PairingMessage::Hello(PairHello {
            device: DeviceId::Helmet,
            mac: [1, 2, 3, 4, 5, 6],
        }));

        let required = MessageType::Pairing.encoded_len();

        for len in 0..required {
            let mut buffer = [0xAA; 32];

            assert_eq!(
                message.encode(&mut buffer[..len]),
                Err(ProtocolError::BufferTooSmall),
            );

            assert!(buffer.iter().all(|&byte| byte == 0xAA));
        }
    }

    #[test]
    fn normal_buffer_boundaries() {
        let message = Message::Rpm(Rpm { amount: 9_000 });
        let required = message.message_type().encoded_len();

        for len in 0..required {
            let mut buffer = [0xAA; 32];

            assert_eq!(
                message.encode(&mut buffer[..len]),
                Err(ProtocolError::BufferTooSmall),
            );
            assert!(buffer.iter().all(|&byte| byte == 0xAA));
        }
    }

    #[test]
    fn rejects_truncated_and_extra_normal_packets() {
        let packets: &[&[u8]] = &[
            &[1, 42],
            &[2, ABSMode::Street as u8],
            &[3, 0x34, 0x12],
            &[4, 1],
            &[5, 0],
        ];

        for packet in packets {
            for length in 0..packet.len() {
                assert!(Message::decode(&packet[..length]).is_err());
            }
        }

        for packet in [
            &[1, 42, 0][..],
            &[2, ABSMode::Street as u8, 0][..],
            &[3, 0x34, 0x12, 0][..],
            &[4, 1, 0][..],
            &[5, 0, 0][..],
        ] {
            assert_eq!(Message::decode(packet), Err(ProtocolError::InvalidPayload));
        }
    }

    #[test]
    fn rejects_truncated_and_extra_pairing_packets() {
        let packet = [6, PairingMessageType::Hello as u8, 2, 1, 2, 3, 4, 5, 6];

        for length in 0..packet.len() {
            assert!(WireMessage::decode(&packet[..length]).is_err());
        }

        let packet_with_extra_byte = [6, PairingMessageType::Hello as u8, 2, 1, 2, 3, 4, 5, 6, 0];
        assert_eq!(
            WireMessage::decode(&packet_with_extra_byte),
            Err(ProtocolError::InvalidPayload),
        );
    }
}
