use esp_idf_svc::espnow::EspNow as EspIdfEspNow;

use crate::iec::protocol::Message;

pub struct EspNow {
    inner: EspIdfEspNow<'static>,
}

impl EspNow {
    /// Wraps an initialized ESP-NOW service. Configure Wi-Fi, peers, and any
    /// callbacks on the service before passing it here; keep Wi-Fi running.
    pub fn new(inner: EspIdfEspNow<'static>) -> Self {
        Self { inner }
    }

    /// Queues a message for a registered peer; success does not confirm delivery.
    pub fn send(&mut self, peer: [u8; 6], message: &Message) -> anyhow::Result<()> {
        let mut buf = [0u8; 250];
        let len = message.encode(&mut buf)?;

        self.inner.send(peer, &buf[..len])?;

        Ok(())
    }
}
