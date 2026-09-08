use crate::iec::protocol::Message;

pub struct EspNow {
    // ESP-NOW handle/state
}

impl EspNow {
    pub fn send(&mut self, peer: [u8; 6], message: &Message) -> anyhow::Result<()> {
        let mut buf = [0u8; 250];
        let len = message.encode(&mut buf)?;

        // send buf[..len] via ESP-NOW

        Ok(())
    }
}
