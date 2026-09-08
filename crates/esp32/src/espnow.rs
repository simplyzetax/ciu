use esp_idf_svc::espnow::EspNow as EspIdfEspNow;

use ciu_core::iec::protocol::Message;

/// Converts normal CIU messages to bytes and sends them through ESP-IDF.
///
/// Pairing messages are intentionally not supported here; pairing is
/// performed exclusively over the physical wire.
pub struct EspNow {
    radio: EspIdfEspNow<'static>,
}

impl EspNow {
    /// Wraps an initialized ESP-NOW service. Configure Wi-Fi, peers, and any
    /// callbacks on the service before passing it here; keep Wi-Fi running.
    pub fn new(radio: EspIdfEspNow<'static>) -> Self {
        Self { radio }
    }

    /// Queues a normal message for a registered peer.
    ///
    /// Success means ESP-IDF accepted the packet for transmission; it does not
    /// confirm application-level delivery.
    pub fn send(&mut self, peer_address: [u8; 6], message: &Message) -> anyhow::Result<()> {
        let mut buffer = [0u8; 250];
        let packet_length = message.encode(&mut buffer)?;

        self.radio.send(peer_address, &buffer[..packet_length])?;

        Ok(())
    }
}
