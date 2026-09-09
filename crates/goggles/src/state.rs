use ciu_core::iec::protocol::ABSMode;

#[derive(Debug, Default)]
pub struct BikeSnapshot {
    pub gear: Option<u8>,
    pub rpm: Option<u16>,
    pub speed: Option<u32>,
    pub clutch: Option<bool>,
    pub throttle: Option<u8>,
    pub kill_switch: Option<bool>,
    pub abs_mode: Option<ABSMode>,
}

#[derive(Debug, Default)]
pub struct BikeState {
    pub snapshot: BikeSnapshot,
}

impl BikeState {
    /// Updates the observed state.
    pub fn update<F>(&mut self, f: F)
    where
        F: FnOnce(&mut BikeSnapshot),
    {
        f(&mut self.snapshot);
    }
}
