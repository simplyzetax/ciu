use ciu_core::iec::protocol::{DeviceId, PairAck, PairHello, PairingMessage, WireMessage};

use crate::{
    saved_state::{SavedState, StateStore},
    wire::{Wire, WireIo},
};

pub struct PairingWire<Io>
where
    Io: WireIo,
{
    wire: Wire<Io>,
}

impl<Io> PairingWire<Io>
where
    Io: WireIo,
{
    pub fn new(io: Io) -> Self {
        Self {
            wire: Wire::new(io),
        }
    }

    /// Goggle side of pairing.
    ///
    /// Waits for the helmet to send its identity over the physical wire,
    /// saves its ESP-NOW MAC, and returns our MAC in a PairAck.
    pub fn pair_as_goggle(
        &mut self,
        my_mac: [u8; 6],
        state: &mut SavedState,
        store: &mut StateStore,
    ) -> anyhow::Result<[u8; 6]> {
        let message = self.wire.receive()?;

        let WireMessage::Pairing(PairingMessage::Hello(hello)) = message else {
            anyhow::bail!("expected PairHello");
        };

        if hello.device != DeviceId::Helmet {
            anyhow::bail!("expected Helmet, got {:?}", hello.device);
        }

        state.set_peer(DeviceId::Helmet, hello.mac);

        store.save(state)?;

        self.wire.send_pairing(&PairingMessage::Ack(PairAck {
            device: DeviceId::Goggle,
            mac: my_mac,
        }))?;

        Ok(hello.mac)
    }

    /// Helmet side of pairing.
    ///
    /// Sends our identity over the physical wire, waits for the goggle's
    /// response, and saves its ESP-NOW MAC.
    pub fn pair_as_helmet(
        &mut self,
        my_mac: [u8; 6],
        state: &mut SavedState,
        store: &mut StateStore,
    ) -> anyhow::Result<[u8; 6]> {
        self.wire.send_pairing(&PairingMessage::Hello(PairHello {
            device: DeviceId::Helmet,
            mac: my_mac,
        }))?;

        let message = self.wire.receive()?;

        let WireMessage::Pairing(PairingMessage::Ack(ack)) = message else {
            anyhow::bail!("expected PairAck");
        };

        if ack.device != DeviceId::Goggle {
            anyhow::bail!("expected Goggle, got {:?}", ack.device);
        }

        state.set_peer(DeviceId::Goggle, ack.mac);

        store.save(state)?;

        Ok(ack.mac)
    }

    pub fn into_inner(self) -> Io {
        self.wire.into_inner()
    }
}
