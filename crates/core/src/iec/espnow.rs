use esp_idf_svc::espnow::EspNow as EspIdfEspNow;

use crate::iec::protocol::Message;

/// Converts our messages to bytes and sends them through ESP-IDF.
pub struct EspNow {
    radio: EspIdfEspNow<'static>,
}

impl EspNow {
    /// Wraps an initialized ESP-NOW service. Configure Wi-Fi, peers, and any
    /// callbacks on the service before passing it here; keep Wi-Fi running.
    pub fn new(radio: EspIdfEspNow<'static>) -> Self {
        Self { radio }
    }

    /// Queues a message for a registered peer; success does not confirm delivery.
    pub fn send(&mut self, peer_address: [u8; 6], message: &Message) -> anyhow::Result<()> {
        // ESP-NOW supports packets up to 250 bytes in its original format.
        let mut buffer = [0u8; 250];
        let packet_length = message.encode(&mut buffer)?;

        // Send only the encoded bytes, not the unused part of the buffer.
        self.radio.send(peer_address, &buffer[..packet_length])?;

        Ok(())
    }
}
