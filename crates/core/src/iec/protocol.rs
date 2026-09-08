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

macro_rules! messages {
    (
        $(
            $name:ident = $id:literal => $payload:ty
        ),* $(,)?
    ) => {
        #[derive(Debug, Clone, Copy, PartialEq, Eq)]
        #[repr(u8)]
        pub enum MessageType {
            $(
                $name = $id,
            )*
        }

        impl TryFrom<u8> for MessageType {
            type Error = ProtocolError;

            fn try_from(value: u8) -> Result<Self, Self::Error> {
                match value {
                    $(
                        $id => Ok(Self::$name),
                    )*
                    _ => Err(ProtocolError::InvalidMessageType(value)),
                }
            }
        }

        #[derive(Debug, Clone, Copy, PartialEq)]
        pub enum Message {
            $(
                $name($payload),
            )*
        }

        impl Message {
            pub fn message_type(&self) -> MessageType {
                match self {
                    $(
                        Self::$name(_) => MessageType::$name,
                    )*
                }
            }
        }
    };
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

messages! {
    Throttle = 1 => Throttle,
    ABS      = 2 => ABSMode,
    Rpm      = 3 => Rpm,
    Clutch   = 4 => Clutch,
}

impl Message {
    /// Encodes a type byte followed by its payload. RPM is little-endian;
    /// clutch is 0 or 1. Returns the number of bytes written.
    pub fn encode(&self, out: &mut [u8]) -> Result<usize, ProtocolError> {
        let len = match self {
            Self::Rpm(_) => 3,
            _ => 2,
        };
        let out = out.get_mut(..len).ok_or(ProtocolError::BufferTooSmall)?;
        out[0] = self.message_type() as u8;
        match self {
            Self::Throttle(throttle) => out[1] = throttle.amount,
            Self::ABS(mode) => out[1] = *mode as u8,
            Self::Rpm(rpm) => out[1..].copy_from_slice(&rpm.amount.to_le_bytes()),
            Self::Clutch(clutch) => out[1] = u8::from(clutch.engaged),
        }
        Ok(len)
    }

    /// Decodes exactly one message, rejecting truncated or trailing payload bytes.
    pub fn decode(data: &[u8]) -> Result<Self, ProtocolError> {
        let (&kind, payload) = data.split_first().ok_or(ProtocolError::InvalidPayload)?;
        match (MessageType::try_from(kind)?, payload) {
            (MessageType::Throttle, &[amount]) => Ok(Self::Throttle(Throttle { amount })),
            (MessageType::ABS, &[1]) => Ok(Self::ABS(ABSMode::Street)),
            (MessageType::ABS, &[2]) => Ok(Self::ABS(ABSMode::Supermoto)),
            (MessageType::Rpm, &[lo, hi]) => Ok(Self::Rpm(Rpm {
                amount: u16::from_le_bytes([lo, hi]),
            })),
            (MessageType::Clutch, &[engaged @ 0..=1]) => Ok(Self::Clutch(Clutch {
                engaged: engaged != 0,
            })),
            _ => Err(ProtocolError::InvalidPayload),
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
