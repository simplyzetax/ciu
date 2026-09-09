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
//! - BikeSnapshot: `[11, present_fields, gear, rpm..., speed..., clutch, throttle, kill_switch, abs]`
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
    Pairing = 6,
    Ping = 7,
    Pong = 8,
    BikeSnapshot = 11,
}

impl TryFrom<u8> for MessageType {
    type Error = ProtocolError;

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value {
            6 => Ok(Self::Pairing),
            7 => Ok(Self::Ping),
            8 => Ok(Self::Pong),
            11 => Ok(Self::BikeSnapshot),
            _ => Err(ProtocolError::InvalidMessageType(value)),
        }
    }
}

impl MessageType {
    const fn encoded_len(self) -> usize {
        match self {
            // type + pairing type + device + MAC
            Self::Pairing => 1 + 1 + 1 + 6,

            // type + u16 sequence
            Self::Ping | Self::Pong => 3,

            // type + present-fields mask + fixed field storage
            Self::BikeSnapshot => 13,
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

/// Bike telemetry carried between CIU devices.
///
/// A producer may send only changed fields. The Helmet merges those partial
/// updates, then periodically sends its complete current view to the Goggles.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct BikeSnapshot {
    pub gear: Option<u8>,
    pub rpm: Option<u16>,
    pub speed: Option<u32>,
    pub clutch: Option<bool>,
    pub throttle: Option<u8>,
    pub kill_switch: Option<bool>,
    pub abs_mode: Option<ABSMode>,
}

impl BikeSnapshot {
    /// Replaces each field included in `update` and keeps every omitted field.
    pub fn merge(&mut self, update: Self) {
        if let Some(gear) = update.gear {
            self.gear = Some(gear);
        }
        if let Some(rpm) = update.rpm {
            self.rpm = Some(rpm);
        }
        if let Some(speed) = update.speed {
            self.speed = Some(speed);
        }
        if let Some(clutch) = update.clutch {
            self.clutch = Some(clutch);
        }
        if let Some(throttle) = update.throttle {
            self.throttle = Some(throttle);
        }
        if let Some(kill_switch) = update.kill_switch {
            self.kill_switch = Some(kill_switch);
        }
        if let Some(abs_mode) = update.abs_mode {
            self.abs_mode = Some(abs_mode);
        }
    }
}

const BIKE_SNAPSHOT_GEAR_PRESENT: u8 = 1 << 0;
const BIKE_SNAPSHOT_RPM_PRESENT: u8 = 1 << 1;
const BIKE_SNAPSHOT_SPEED_PRESENT: u8 = 1 << 2;
const BIKE_SNAPSHOT_CLUTCH_PRESENT: u8 = 1 << 3;
const BIKE_SNAPSHOT_THROTTLE_PRESENT: u8 = 1 << 4;
const BIKE_SNAPSHOT_KILL_SWITCH_PRESENT: u8 = 1 << 5;
const BIKE_SNAPSHOT_ABS_MODE_PRESENT: u8 = 1 << 6;
const BIKE_SNAPSHOT_ALL_FIELDS: u8 = BIKE_SNAPSHOT_GEAR_PRESENT
    | BIKE_SNAPSHOT_RPM_PRESENT
    | BIKE_SNAPSHOT_SPEED_PRESENT
    | BIKE_SNAPSHOT_CLUTCH_PRESENT
    | BIKE_SNAPSHOT_THROTTLE_PRESENT
    | BIKE_SNAPSHOT_KILL_SWITCH_PRESENT
    | BIKE_SNAPSHOT_ABS_MODE_PRESENT;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Ping {
    pub sequence: u16,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Pong {
    pub sequence: u16,
}

/// A normal CIU message.
///
/// These messages may be transported over both ESP-NOW and the physical wire.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Message {
    BikeSnapshot(BikeSnapshot),
    Ping(Ping),
    Pong(Pong),
}
impl Message {
    pub fn message_type(&self) -> MessageType {
        match self {
            Self::BikeSnapshot(_) => MessageType::BikeSnapshot,
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
            Self::BikeSnapshot(snapshot) => {
                buffer[1..packet_length].fill(0);
                let mut present_fields = 0;

                if let Some(gear) = snapshot.gear {
                    present_fields |= BIKE_SNAPSHOT_GEAR_PRESENT;
                    buffer[2] = gear;
                }
                if let Some(rpm) = snapshot.rpm {
                    present_fields |= BIKE_SNAPSHOT_RPM_PRESENT;
                    buffer[3..5].copy_from_slice(&rpm.to_le_bytes());
                }
                if let Some(speed) = snapshot.speed {
                    present_fields |= BIKE_SNAPSHOT_SPEED_PRESENT;
                    buffer[5..9].copy_from_slice(&speed.to_le_bytes());
                }
                if let Some(clutch) = snapshot.clutch {
                    present_fields |= BIKE_SNAPSHOT_CLUTCH_PRESENT;
                    buffer[9] = u8::from(clutch);
                }
                if let Some(throttle) = snapshot.throttle {
                    present_fields |= BIKE_SNAPSHOT_THROTTLE_PRESENT;
                    buffer[10] = throttle;
                }
                if let Some(kill_switch) = snapshot.kill_switch {
                    present_fields |= BIKE_SNAPSHOT_KILL_SWITCH_PRESENT;
                    buffer[11] = u8::from(kill_switch);
                }
                if let Some(abs_mode) = snapshot.abs_mode {
                    present_fields |= BIKE_SNAPSHOT_ABS_MODE_PRESENT;
                    buffer[12] = abs_mode as u8;
                }

                buffer[1] = present_fields;
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
            MessageType::BikeSnapshot => {
                let present_fields = data[1];

                if present_fields & !BIKE_SNAPSHOT_ALL_FIELDS != 0 {
                    return Err(ProtocolError::InvalidPayload);
                }

                let gear = if present_fields & BIKE_SNAPSHOT_GEAR_PRESENT != 0 {
                    Some(data[2])
                } else {
                    None
                };
                let rpm = if present_fields & BIKE_SNAPSHOT_RPM_PRESENT != 0 {
                    Some(u16::from_le_bytes([data[3], data[4]]))
                } else {
                    None
                };
                let speed = if present_fields & BIKE_SNAPSHOT_SPEED_PRESENT != 0 {
                    Some(u32::from_le_bytes([data[5], data[6], data[7], data[8]]))
                } else {
                    None
                };
                let clutch = if present_fields & BIKE_SNAPSHOT_CLUTCH_PRESENT != 0 {
                    Some(decode_bool(data[9])?)
                } else {
                    None
                };
                let throttle = if present_fields & BIKE_SNAPSHOT_THROTTLE_PRESENT != 0 {
                    Some(data[10])
                } else {
                    None
                };
                let kill_switch = if present_fields & BIKE_SNAPSHOT_KILL_SWITCH_PRESENT != 0 {
                    Some(decode_bool(data[11])?)
                } else {
                    None
                };
                let abs_mode = if present_fields & BIKE_SNAPSHOT_ABS_MODE_PRESENT != 0 {
                    Some(ABSMode::try_from(data[12])?)
                } else {
                    None
                };

                Ok(Self::BikeSnapshot(BikeSnapshot {
                    gear,
                    rpm,
                    speed,
                    clutch,
                    throttle,
                    kill_switch,
                    abs_mode,
                }))
            }

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
            (
                Message::BikeSnapshot(BikeSnapshot {
                    gear: Some(6),
                    rpm: Some(0x1234),
                    speed: Some(0x1234_5678),
                    clutch: Some(true),
                    throttle: Some(255),
                    kill_switch: Some(false),
                    abs_mode: Some(ABSMode::Supermoto),
                }),
                &[
                    11, 0x7F, 6, 0x34, 0x12, 0x78, 0x56, 0x34, 0x12, 1, 255, 0, 2,
                ],
            ),
            (Message::Ping(Ping { sequence: 0x1234 }), &[7, 0x34, 0x12]),
            (Message::Pong(Pong { sequence: 0x5678 }), &[8, 0x78, 0x56]),
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
    fn bike_snapshot_preserves_unknown_fields() {
        let message = Message::BikeSnapshot(BikeSnapshot {
            speed: Some(123),
            ..BikeSnapshot::default()
        });
        let expected = [
            11,
            BIKE_SNAPSHOT_SPEED_PRESENT,
            0,
            0,
            0,
            123,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
        ];

        let mut buffer = [0xAA; 32];
        let len = message.encode(&mut buffer).unwrap();

        assert_eq!(&buffer[..len], &expected);
        assert_eq!(Message::decode(&expected).unwrap(), message);
    }

    #[test]
    fn bike_snapshot_merges_partial_updates() {
        let mut state = BikeSnapshot {
            gear: Some(3),
            speed: Some(50),
            ..BikeSnapshot::default()
        };

        state.merge(BikeSnapshot {
            rpm: Some(4_500),
            speed: Some(60),
            ..BikeSnapshot::default()
        });

        assert_eq!(
            state,
            BikeSnapshot {
                gear: Some(3),
                rpm: Some(4_500),
                speed: Some(60),
                ..BikeSnapshot::default()
            },
        );
    }

    #[test]
    fn normal_messages_can_travel_over_wire() {
        let message = Message::BikeSnapshot(BikeSnapshot {
            rpm: Some(5_000),
            ..BikeSnapshot::default()
        });
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
        for wire in [
            &[11, 0x80, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0][..],
            &[
                11,
                BIKE_SNAPSHOT_CLUTCH_PRESENT,
                0,
                0,
                0,
                0,
                0,
                0,
                0,
                2,
                0,
                0,
                0,
            ][..],
            &[
                11,
                BIKE_SNAPSHOT_ABS_MODE_PRESENT,
                0,
                0,
                0,
                0,
                0,
                0,
                0,
                0,
                0,
                0,
                3,
            ][..],
        ] {
            assert_eq!(Message::decode(wire), Err(ProtocolError::InvalidPayload));
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
        let message = Message::BikeSnapshot(BikeSnapshot::default());
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
            &[11, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0],
            &[7, 0x34, 0x12],
            &[8, 0x78, 0x56],
        ];

        for packet in packets {
            for length in 0..packet.len() {
                assert!(Message::decode(&packet[..length]).is_err());
            }

            let mut packet_with_extra_byte = packet.to_vec();
            packet_with_extra_byte.push(0);
            assert_eq!(
                Message::decode(&packet_with_extra_byte),
                Err(ProtocolError::InvalidPayload),
            );
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
