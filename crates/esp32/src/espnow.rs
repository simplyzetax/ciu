use std::{
    sync::{
        Arc, Mutex,
        mpsc::{Receiver, SyncSender, TrySendError, sync_channel},
    },
    thread,
};

use esp_idf_hal::delay::FreeRtos;
use esp_idf_svc::{
    espnow::{EspNow as EspIdfEspNow, PeerInfo, SendStatus},
    sys::ESP_ERR_ESPNOW_NO_MEM,
};

use ciu_core::iec::protocol::{DeviceId, Message, Ping, Pong};

use crate::saved_state::{RuntimeState, SavedState};

/// Converts normal CIU messages to/from bytes and sends/receives them through
/// ESP-IDF.
///
/// Pairing messages are intentionally not supported here; pairing is
/// performed exclusively over the physical wire.
const SEND_QUEUE_CAPACITY: usize = 8;
const SEND_RETRY_DELAY_MS: u32 = 10;
const TX_STACK_SIZE: usize = 4 * 1024;

struct OutgoingPacket {
    peer_address: [u8; 6],
    buffer: [u8; 250],
    length: usize,
}

pub struct EspNow {
    radio: Arc<EspIdfEspNow<'static>>,
    send_queue: SyncSender<OutgoingPacket>,
}

fn transmit_packets(
    radio: Arc<EspIdfEspNow<'static>>,
    packets: Receiver<OutgoingPacket>,
    completions: Receiver<SendStatus>,
) {
    while let Ok(packet) = packets.recv() {
        loop {
            match radio.send(packet.peer_address, &packet.buffer[..packet.length]) {
                Ok(()) => {
                    match completions.recv() {
                        Ok(SendStatus::SUCCESS) => {}
                        Ok(SendStatus::FAIL) => {
                            println!("ESP-NOW delivery to {:02X?} failed", packet.peer_address);
                        }
                        Err(_) => return,
                    }

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

impl EspNow {
    /// Wraps an initialized ESP-NOW service.
    ///
    /// Keep Wi-Fi running for as long as this object exists.
    pub fn new(radio: EspIdfEspNow<'static>) -> anyhow::Result<Self> {
        let radio = Arc::new(radio);
        let (send_queue, packets) = sync_channel(SEND_QUEUE_CAPACITY);
        let (completion_sender, completions) = sync_channel(1);

        radio.register_send_cb(move |_peer_address, status| {
            let _ = completion_sender.try_send(status);
        })?;

        let transmitter_radio = Arc::clone(&radio);

        thread::Builder::new()
            .name("esp-now-tx".into())
            .stack_size(TX_STACK_SIZE)
            .spawn(move || {
                transmit_packets(transmitter_radio, packets, completions);
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
    /// waits for ESP-IDF's send callback before sending the next packet.
    pub fn send(&self, peer_address: [u8; 6], message: &Message) -> anyhow::Result<()> {
        let mut buffer = [0u8; 250];
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

    /// Registers a callback for normal CIU messages received over ESP-NOW.
    ///
    /// Invalid packets, including pairing packets, are ignored. Ping and Pong
    /// are handled here; all application messages are passed to `on_message`.
    pub fn on_receive(
        self: &Arc<Self>,
        saved_state: Arc<Mutex<SavedState>>,
        runtime_state: Arc<Mutex<RuntimeState>>,
        mut on_message: impl FnMut(DeviceId, Message) + Send + 'static,
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

                other => on_message(device, other),
            }
        })?;

        Ok(())
    }

    pub fn send_ping(&self, peer_address: [u8; 6], sequence: u16) -> anyhow::Result<()> {
        self.send(peer_address, &Message::Ping(Ping { sequence }))
    }
}
