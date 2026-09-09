use std::{
    sync::{
        Arc,
        mpsc::{Receiver, SyncSender, TrySendError, sync_channel},
    },
    thread,
};

use esp_idf_hal::delay::FreeRtos;
use esp_idf_svc::{
    espnow::{EspNow as EspIdfEspNow, PeerInfo},
    sys::ESP_ERR_ESPNOW_NO_MEM,
};

use ciu_core::iec::protocol::Message;

/// Converts normal CIU messages to/from bytes and sends/receives them through
/// ESP-IDF.
///
/// Pairing messages are intentionally not supported here; pairing is
/// performed exclusively over the physical wire.
const SEND_QUEUE_CAPACITY: usize = 8;
const MAX_PACKET_SIZE: usize = Message::MAX_ENCODED_LEN;
const RECEIVE_QUEUE_CAPACITY: usize = 8;
const SEND_RETRY_DELAY_MS: u32 = 10;
const SEND_SPACING_MS: u32 = 20;
const TX_STACK_SIZE: usize = 10 * 1024;
const RX_STACK_SIZE: usize = 10 * 1024;

struct OutgoingPacket {
    peer_address: [u8; 6],
    buffer: [u8; MAX_PACKET_SIZE],
    length: usize,
}

struct IncomingPacket {
    peer_address: [u8; 6],
    buffer: [u8; MAX_PACKET_SIZE],
    length: usize,
}

pub struct EspNow {
    radio: Arc<EspIdfEspNow<'static>>,
    send_queue: SyncSender<OutgoingPacket>,
}

fn transmit_packets(radio: Arc<EspIdfEspNow<'static>>, packets: Receiver<OutgoingPacket>) {
    while let Ok(packet) = packets.recv() {
        loop {
            match radio.send(packet.peer_address, &packet.buffer[..packet.length]) {
                Ok(()) => {
                    FreeRtos::delay_ms(SEND_SPACING_MS);
                    break;
                }
                Err(error) if error.code() == ESP_ERR_ESPNOW_NO_MEM => {
                    FreeRtos::delay_ms(SEND_RETRY_DELAY_MS);
                }
                Err(error) => {
                    println!(
                        "ESP-NOW send to {:02X?} failed: {error}",
                        packet.peer_address
                    );
                    break;
                }
            }
        }
    }
}

fn receive_packets(
    packets: Receiver<IncomingPacket>,
    mut on_message: impl FnMut([u8; 6], Message),
) {
    while let Ok(packet) = packets.recv() {
        let Ok(message) = Message::decode(&packet.buffer[..packet.length]) else {
            continue;
        };

        on_message(packet.peer_address, message);
    }
}

impl EspNow {
    /// Wraps an initialized ESP-NOW service.
    ///
    /// Keep Wi-Fi running for as long as this object exists.
    pub fn new(radio: EspIdfEspNow<'static>) -> anyhow::Result<Self> {
        let radio = Arc::new(radio);
        let (send_queue, packets) = sync_channel(SEND_QUEUE_CAPACITY);
        let transmitter_radio = Arc::clone(&radio);

        thread::Builder::new()
            .name("esp-now-tx".into())
            .stack_size(TX_STACK_SIZE)
            .spawn(move || {
                transmit_packets(transmitter_radio, packets);
            })?;

        Ok(Self { radio, send_queue })
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

    /// Queues a normal message for serialized transmission.
    ///
    /// Success means the local queue accepted the packet. The transmitter
    /// spaces ESP-IDF calls apart to avoid exhausting Wi-Fi buffers.
    pub fn send(&self, peer_address: [u8; 6], message: &Message) -> anyhow::Result<()> {
        let mut buffer = [0u8; MAX_PACKET_SIZE];
        let length = message.encode(&mut buffer)?;
        let packet = OutgoingPacket {
            peer_address,
            buffer,
            length,
        };

        match self.send_queue.try_send(packet) {
            Ok(()) => Ok(()),
            Err(TrySendError::Full(_)) => anyhow::bail!("ESP-NOW send queue is full"),
            Err(TrySendError::Disconnected(_)) => {
                anyhow::bail!("ESP-NOW transmitter stopped")
            }
        }
    }

    /// Registers a callback for valid normal messages received over ESP-NOW.
    ///
    /// Invalid packets, including pairing packets, are ignored. The Wi-Fi task
    /// only copies packets into a queue; decoding and the callback run on a
    /// separate task with enough stack.
    pub fn on_receive(
        &self,
        on_message: impl FnMut([u8; 6], Message) + Send + 'static,
    ) -> anyhow::Result<()> {
        let (receive_queue, packets) = sync_channel(RECEIVE_QUEUE_CAPACITY);

        thread::Builder::new()
            .name("esp-now-rx".into())
            .stack_size(RX_STACK_SIZE)
            .spawn(move || {
                receive_packets(packets, on_message);
            })?;

        self.radio.register_recv_cb(move |info, data| {
            if data.len() > MAX_PACKET_SIZE {
                return;
            }

            let mut buffer = [0u8; MAX_PACKET_SIZE];
            buffer[..data.len()].copy_from_slice(data);

            let packet = IncomingPacket {
                peer_address: *info.src_addr,
                buffer,
                length: data.len(),
            };

            // ESP-NOW is lossy. Never block or log from the Wi-Fi task.
            let _ = receive_queue.try_send(packet);
        })?;

        Ok(())
    }
}
