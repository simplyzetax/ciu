use esp_idf_svc::nvs::{EspDefaultNvsPartition, EspNvs, NvsDefault};
use serde::{Deserialize, Serialize};

use ciu_core::iec::protocol::DeviceId;

const NAMESPACE: &str = "ciu";
const STATE_KEY: &str = "state";

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Peer {
    pub device: DeviceId,
    pub mac: [u8; 6],
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct SavedState {
    pub peers: Vec<Peer>,
}

impl SavedState {
    pub fn peer(&self, device: DeviceId) -> Option<&Peer> {
        self.peers.iter().find(|peer| peer.device == device)
    }

    /*
    TODO: This shoould run when the googles connect to the helmet via magnetic pins
    or when the bike connects to the helmet via phone app (or somehow else)
    */
    pub fn set_peer(&mut self, device: DeviceId, mac: [u8; 6]) {
        if let Some(peer) = self.peers.iter_mut().find(|peer| peer.device == device) {
            peer.mac = mac;
            return;
        }

        self.peers.push(Peer { device, mac });
    }

    pub fn remove_peer(&mut self, device: DeviceId) {
        self.peers.retain(|peer| peer.device != device);
    }
}

pub struct StateStore {
    nvs: EspNvs<NvsDefault>,
}

impl StateStore {
    pub fn new() -> anyhow::Result<Self> {
        let partition = EspDefaultNvsPartition::take()?;
        let nvs = EspNvs::new(partition, NAMESPACE, true)?;

        Ok(Self { nvs })
    }

    pub fn load(&self) -> anyhow::Result<SavedState> {
        let Some(length) = self.nvs.blob_len(STATE_KEY)? else {
            return Ok(SavedState::default());
        };

        let mut buffer = vec![0u8; length];

        let Some(bytes) = self.nvs.get_blob(STATE_KEY, &mut buffer)? else {
            return Ok(SavedState::default());
        };

        Ok(serde_json::from_slice(bytes)?)
    }

    pub fn save(&mut self, state: &SavedState) -> anyhow::Result<()> {
        let bytes = serde_json::to_vec(state)?;

        self.nvs.set_blob(STATE_KEY, &bytes)?;

        Ok(())
    }
}
