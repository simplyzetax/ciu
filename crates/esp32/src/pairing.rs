use std::sync::Mutex;

use anyhow::Context;
use ciu_core::iec::protocol::{DeviceId, PairAck, PairHello, PairingMessage, WireMessage};

use crate::{
    saved_state::{SavedState, StateStore},
    wire::{Wire, WireIo},
};

/// Goggle side of pairing.
///
/// Waits for a helmet hello, persists its ESP-NOW STA MAC, then acknowledges
/// with the goggle's ESP-NOW STA MAC.
pub fn pair_as_goggle<Io>(
    wire: &mut Wire<Io>,
    my_mac: [u8; 6],
    state: &Mutex<SavedState>,
    store: &mut StateStore,
) -> anyhow::Result<[u8; 6]>
where
    Io: WireIo,
{
    let message = wire.receive().context("receiving PairHello")?;

    let WireMessage::Pairing(PairingMessage::Hello(hello)) = message else {
        anyhow::bail!("expected PairHello");
    };

    if hello.device != DeviceId::Helmet {
        anyhow::bail!("expected Helmet, got {:?}", hello.device);
    }

    {
        let mut state = state.lock().expect("saved state mutex poisoned");

        state.set_peer(DeviceId::Helmet, hello.mac);
        store.save(&state).context("persisting helmet peer")?;
    }

    wire.send_pairing(&PairingMessage::Ack(PairAck {
        device: DeviceId::Goggle,
        mac: my_mac,
    }))
    .context("sending PairAck")?;

    Ok(hello.mac)
}

/// Helmet side of pairing.
///
/// Sends the helmet's ESP-NOW STA MAC, waits for a goggle acknowledgement,
/// validates it, and persists the goggle's ESP-NOW STA MAC.
pub fn pair_as_helmet<Io>(
    wire: &mut Wire<Io>,
    my_mac: [u8; 6],
    state: &Mutex<SavedState>,
    store: &mut StateStore,
) -> anyhow::Result<[u8; 6]>
where
    Io: WireIo,
{
    wire.send_pairing(&PairingMessage::Hello(PairHello {
        device: DeviceId::Helmet,
        mac: my_mac,
    }))
    .context("sending PairHello")?;

    let message = wire.receive().context("receiving PairAck")?;

    let WireMessage::Pairing(PairingMessage::Ack(ack)) = message else {
        anyhow::bail!("expected PairAck");
    };

    if ack.device != DeviceId::Goggle {
        anyhow::bail!("expected Goggle, got {:?}", ack.device);
    }

    let mut state = state.lock().expect("saved state mutex poisoned");

    state.set_peer(DeviceId::Goggle, ack.mac);
    store.save(&state).context("persisting goggle peer")?;

    Ok(ack.mac)
}
