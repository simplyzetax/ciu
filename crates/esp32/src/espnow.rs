use std::sync::{Arc, Mutex};

use esp_idf_svc::espnow::{EspNow as EspIdfEspNow, PeerInfo};

use ciu_core::iec::protocol::{Message, Ping, Pong};

use crate::saved_state::{RuntimeState, SavedState};

/// Converts normal CIU messages to/from bytes and sends/receives them through
/// ESP-IDF.
///
/// Pairing messages are intentionally not supported here; pairing is
/// performed exclusively over the physical wire.
pub struct EspNow {
    radio: EspIdfEspNow<'static>,
}

impl EspNow {
    /// Wraps an initialized ESP-NOW service.
    ///
    /// Keep Wi-Fi running for as long as this object exists.
    pub fn new(radio: EspIdfEspNow<'static>) -> Self {
        Self { radio }
    }

    pub fn add_peer(&self, peer_address: [u8; 6]) -> anyhow::Result<()> {
        if self.radio.peer_exists(peer_address)? {
            return Ok(());
        }

        let mut peer = PeerInfo::default();
        peer.peer_addr = peer_address;
        peer.channel = 0;
        peer.encrypt = false;

        self.radio.add_peer(peer)?;

        Ok(())
    }

    /// Queues a normal message for a registered peer.
    ///
    /// Success means ESP-IDF accepted the packet for transmission; it does not
    /// confirm application-level delivery.
    pub fn send(&self, peer_address: [u8; 6], message: &Message) -> anyhow::Result<()> {
        let mut buffer = [0u8; 250];
        let packet_length = message.encode(&mut buffer)?;

        self.radio.send(peer_address, &buffer[..packet_length])?;

        Ok(())
    }

    /// Registers a callback for normal CIU messages received over ESP-NOW.
    ///
    /// Invalid packets, including pairing packets, are ignored.
    pub fn on_receive(
        self: &Arc<Self>,
        saved_state: Arc<Mutex<SavedState>>,
        runtime_state: Arc<Mutex<RuntimeState>>,
    ) -> anyhow::Result<()> {
        let esp_now = Arc::clone(self);

        self.radio.register_recv_cb(move |info, data| {
            let Ok(message) = Message::decode(data) else {
                return;
            };

            let device = {
                let state = saved_state.lock().expect("saved state mutex poisoned");

                state
                    .peers
                    .iter()
                    .find(|peer| peer.mac == *info.src_addr)
                    .map(|peer| peer.device)
            };

            let Some(device) = device else {
                println!(
                    "Ignoring ESP-NOW message from unknown peer {:02X?}",
                    info.src_addr
                );
                return;
            };

            runtime_state
                .lock()
                .expect("runtime state mutex poisoned")
                .mark_seen(device);

            match message {
                Message::Ping(ping) => {
                    let pong = Message::Pong(Pong {
                        sequence: ping.sequence,
                    });

                    if let Err(error) = esp_now.send(*info.src_addr, &pong) {
                        println!("Failed to send Pong to {:?}: {error:#}", device);
                    }
                }

                Message::Pong(pong) => {
                    println!("Pong from {:?}, sequence {}", device, pong.sequence);
                }

                other => {
                    println!("ESP-NOW RX from {:?}: {:?}", device, other);
                }
            }
        })?;

        Ok(())
    }

    pub fn send_ping(&self, peer_address: [u8; 6], sequence: u16) -> anyhow::Result<()> {
        self.send(peer_address, &Message::Ping(Ping { sequence }))
    }
}
