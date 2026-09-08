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

            pub fn encode(&self, out: &mut [u8]) -> Result<usize, ProtocolError> {
                    // ...
                }

                pub fn decode(data: &[u8]) -> Result<Self, ProtocolError> {
                    // ...
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
