use std::sync::{Arc, Mutex};

use anyhow::Context;
use ciu_core::iec::protocol::{BikeSnapshot, DeviceId, Message, Ping, Pong};
use esp_idf_svc::espnow::EspNow as EspIdfEspNow;

use crate::{
    espnow::EspNow,
    saved_state::{SavedState, StateStore},
};

/// Persistent peers and ESP-NOW behavior shared by every CIU device.
pub struct Network {
    esp_now: EspNow,
    saved_state: Mutex<SavedState>,
    store: Mutex<StateStore>,
}

impl Network {
    /// Starts ESP-NOW and restores every previously paired device.
    pub fn new() -> anyhow::Result<Arc<Self>> {
        let store = StateStore::new().context("opening saved peer state")?;
        let saved_state = store.load().context("loading saved peer state")?;
        let esp_now = EspNow::new(EspIdfEspNow::take()?)?;

        for peer in &saved_state.peers {
            println!("Peer {:?} is paired", peer.device);
            esp_now.add_peer(peer.mac)?;
        }

        Ok(Arc::new(Self {
            esp_now,
            saved_state: Mutex::new(saved_state),
            store: Mutex::new(store),
        }))
    }

    /// Handles Ping and Pong internally and forwards bike snapshots to the app.
    pub fn on_snapshot(
        self: &Arc<Self>,
        mut callback: impl FnMut(DeviceId, BikeSnapshot) + Send + 'static,
    ) -> anyhow::Result<()> {
        let network = Arc::clone(self);

        self.esp_now.on_receive(move |peer_address, message| {
            let Some(device) = network.device_for_address(peer_address) else {
                println!(
                    "Ignoring ESP-NOW message from unknown peer {:02X?}",
                    peer_address
                );
                return;
            };

            match message {
                Message::Ping(ping) => {
                    let pong = Message::Pong(Pong {
                        sequence: ping.sequence,
                    });

                    if let Err(error) = network.esp_now.send(peer_address, &pong) {
                        println!("Failed to send Pong to {:?}: {error:#}", device);
                    }
                }
                Message::Pong(pong) => {
                    println!("Pong from {:?}, sequence {}", device, pong.sequence);
                }
                Message::BikeSnapshot(snapshot) => callback(device, snapshot),
            }
        })
    }

    /// Persists a pairing and makes the peer immediately available to ESP-NOW.
    pub(crate) fn add_peer(&self, device: DeviceId, mac: [u8; 6]) -> anyhow::Result<()> {
        {
            let mut state = self.saved_state.lock().expect("saved state mutex poisoned");
            state.set_peer(device, mac);

            self.store
                .lock()
                .expect("state store mutex poisoned")
                .save(&state)
                .context("persisting paired device")?;
        }

        self.esp_now.add_peer(mac)
    }

    /// Sends a message when the device is paired. An unpaired device is skipped.
    pub fn send(&self, device: DeviceId, message: &Message) -> anyhow::Result<()> {
        let peer_address = {
            let state = self.saved_state.lock().expect("saved state mutex poisoned");
            state.peer(device).map(|peer| peer.mac)
        };

        if let Some(peer_address) = peer_address {
            self.esp_now.send(peer_address, message)?;
        }

        Ok(())
    }

    /// Sends one Ping to every paired device and advances the shared sequence.
    pub fn ping_all(&self, sequence: &mut u16) {
        let state = self.saved_state.lock().expect("saved state mutex poisoned");

        for peer in &state.peers {
            let ping = Message::Ping(Ping {
                sequence: *sequence,
            });
            *sequence = sequence.wrapping_add(1);

            if let Err(error) = self.esp_now.send(peer.mac, &ping) {
                println!("Failed to send Ping to {:?}: {error:#}", peer.device);
            }
        }
    }

    fn device_for_address(&self, peer_address: [u8; 6]) -> Option<DeviceId> {
        self.saved_state
            .lock()
            .expect("saved state mutex poisoned")
            .peers
            .iter()
            .find(|peer| peer.mac == peer_address)
            .map(|peer| peer.device)
    }
}
