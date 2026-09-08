use ciu_core::iec::protocol::{Message, PairingMessage, WireMessage};

const MAX_PAYLOAD_SIZE: usize = 250;

/// Low-level byte transport used by the physical wire.
///
/// The actual GPIO implementation only needs to provide these two operations.
/// Framing and CIU protocol encoding stay in `Wire`.
pub trait WireIo {
    fn write_all(&mut self, data: &[u8]) -> anyhow::Result<()>;
    fn read_exact(&mut self, data: &mut [u8]) -> anyhow::Result<()>;
}

/// Sends and receives CIU messages over the physical wire.
///
/// Unlike ESP-NOW, the wire can carry both:
///
/// - normal `Message`s
/// - wire-only `PairingMessage`s
///
/// A one-byte length prefix is added by this transport so the receiver knows
/// how many protocol bytes belong to the packet.
pub struct Wire<Io> {
    io: Io,
}

impl<Io> Wire<Io>
where
    Io: WireIo,
{
    pub fn new(io: Io) -> Self {
        Self { io }
    }

    /// Sends any message supported by the physical wire.
    pub fn send(&mut self, message: &WireMessage) -> anyhow::Result<()> {
        let mut payload = [0u8; MAX_PAYLOAD_SIZE];
        let payload_length = message.encode(&mut payload)?;

        let payload_length = u8::try_from(payload_length)
            .map_err(|_| anyhow::anyhow!("wire payload is too large"))?;

        // Transport framing:
        //
        // [payload length][encoded WireMessage...]
        self.io.write_all(&[payload_length])?;
        self.io.write_all(&payload[..payload_length as usize])?;

        Ok(())
    }

    /// Convenience method for sending a normal message over the wire.
    pub fn send_message(&mut self, message: &Message) -> anyhow::Result<()> {
        self.send(&WireMessage::Message(*message))
    }

    /// Convenience method for sending a wire-only pairing message.
    pub fn send_pairing(&mut self, message: &PairingMessage) -> anyhow::Result<()> {
        self.send(&WireMessage::Pairing(*message))
    }

    /// Blocks until one complete wire message has been received.
    pub fn receive(&mut self) -> anyhow::Result<WireMessage> {
        let mut length = [0u8; 1];
        self.io.read_exact(&mut length)?;

        let payload_length = length[0] as usize;

        if payload_length == 0 || payload_length > MAX_PAYLOAD_SIZE {
            anyhow::bail!("invalid wire payload length: {payload_length}");
        }

        let mut payload = [0u8; MAX_PAYLOAD_SIZE];

        self.io.read_exact(&mut payload[..payload_length])?;

        Ok(WireMessage::decode(&payload[..payload_length])?)
    }

    pub fn into_inner(self) -> Io {
        self.io
    }
}
