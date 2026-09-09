use anyhow::Context;
use ciu_core::iec::protocol::{DeviceId, PairAck, PairHello, PairingMessage, WireMessage};

use crate::{
    network::Network,
    wire::{Wire, WireIo},
};

/// Goggle side of the physical pairing handshake.
///
/// Waits for a helmet hello, persists its ESP-NOW station MAC, then
/// acknowledges it with the goggle's station MAC.
pub fn pair_as_goggle<Io>(
    wire: &mut Wire<Io>,
    my_mac: [u8; 6],
    network: &Network,
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

    network.add_peer(DeviceId::Helmet, hello.mac)?;

    wire.send_pairing(&PairingMessage::Ack(PairAck {
        device: DeviceId::Goggle,
        mac: my_mac,
    }))
    .context("sending PairAck")?;

    Ok(hello.mac)
}

/// Helmet side of the physical pairing handshake.
///
/// Sends the helmet's ESP-NOW station MAC, then waits for and validates a
/// goggle acknowledgement before persisting the peer.
pub fn pair_as_helmet<Io>(
    wire: &mut Wire<Io>,
    my_mac: [u8; 6],
    network: &Network,
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

    network.add_peer(DeviceId::Goggle, ack.mac)?;

    Ok(ack.mac)
}
