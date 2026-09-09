mod state;

use std::sync::{Arc, Mutex};

use ciu_core::iec::protocol::{ApplicationMessage, DeviceId};
use ciu_esp32::{
    pairing::pair_as_goggle,
    saved_state::{PeerRuntime, RuntimeState, StateStore},
    wire::{GpioWireIo, Wire},
};
use esp_idf_hal::{delay::FreeRtos, gpio::PinDriver, peripherals::Peripherals};

use crate::state::BikeState;

const APP_STACK_SIZE: usize = 10 * 1024;
const PAIRING_STACK_SIZE: usize = 8 * 1024;

fn main() {
    esp_idf_svc::sys::link_patches();

    std::thread::Builder::new()
        .name("goggles".into())
        .stack_size(APP_STACK_SIZE)
        .spawn(run)
        .expect("failed to start goggles task")
        .join()
        .expect("goggles task panicked")
        .expect("goggles initialization failed");
}

fn run() -> anyhow::Result<()> {
    let peripherals = Peripherals::take()?;
    let mut led = PinDriver::output(peripherals.pins.gpio2)?;

    let wifi = ciu_esp32::wifi::start(peripherals.modem, ciu_esp32::wifi::WIFI_CHANNEL)?;
    let my_mac = ciu_esp32::wifi::station_mac(&wifi)?;

    let mut store = StateStore::new()?;
    let initial_state = store.load()?;

    for peer in &initial_state.peers {
        println!("Peer {:?} is paired", peer.device);
    }

    let saved_state = Arc::new(Mutex::new(initial_state));

    let runtime_state = Arc::new(Mutex::new({
        let state = saved_state.lock().expect("saved state mutex poisoned");

        RuntimeState::from_saved_state(&state)
    }));

    let radio = esp_idf_svc::espnow::EspNow::take()?;
    let esp_now = Arc::new(ciu_esp32::espnow::EspNow::new(radio)?);
    {
        let state = saved_state.lock().expect("saved state mutex poisoned");

        for peer in &state.peers {
            esp_now.add_peer(peer.mac)?;
        }
    }

    let mut bike_state = BikeState::default();

    esp_now.on_receive(
        Arc::clone(&saved_state),
        Arc::clone(&runtime_state),
        move |device, message| {
            println!("Goggles ESP-NOW RX from {:?}: {:?}", device, message);
            match message {
                ApplicationMessage::Throttle(v) => {
                    bike_state.update(|c| c.throttle = Some(v.amount));
                }
                ApplicationMessage::ABS(v) => {
                    bike_state.update(|c| c.abs_mode = Some(v));
                }
                ApplicationMessage::Clutch(v) => {
                    bike_state.update(|c| c.clutch = Some(v.engaged));
                }
                ApplicationMessage::KillSwitch(v) => {
                    bike_state.update(|c| c.kill_switch = Some(v.engaged));
                }
                ApplicationMessage::Rpm(v) => {
                    bike_state.update(|c| c.rpm = Some(v.amount));
                }
                ApplicationMessage::Gear(v) => {
                    bike_state.update(|c| c.gear = Some(v.number));
                }
                ApplicationMessage::Speed(v) => {
                    bike_state.update(|c| c.speed = Some(v.amount));
                }
            }
        },
    )?;

    let io = GpioWireIo::new(peripherals.pins.gpio4)?;
    let mut wire = Wire::new(io);

    let pairing_saved_state = Arc::clone(&saved_state);
    let pairing_runtime_state = Arc::clone(&runtime_state);
    let pairing_esp_now = Arc::clone(&esp_now);

    let _pairing_thread = std::thread::Builder::new()
        .name("pairing".into())
        .stack_size(PAIRING_STACK_SIZE)
        .spawn(move || {
            loop {
                match pair_as_goggle(&mut wire, my_mac, &pairing_saved_state, &mut store) {
                    Ok(helmet_mac) => {
                        if let Err(error) = pairing_esp_now.add_peer(helmet_mac) {
                            println!("Failed to register helmet with ESP-NOW: {error:#}");
                            led.set_low().ok();
                        } else {
                            println!("Paired with helmet {:02X?}", helmet_mac);
                            led.set_high().ok();

                            let mut runtime = pairing_runtime_state
                                .lock()
                                .expect("runtime state mutex poisoned");

                            if runtime.peer(DeviceId::Helmet).is_none() {
                                runtime.peers.push(PeerRuntime::new(DeviceId::Helmet));
                            }
                        }
                    }

                    Err(error) => {
                        let is_timeout = error
                            .chain()
                            .any(|cause| cause.to_string().contains("wire receive timed out"));

                        if !is_timeout {
                            println!("Pairing failed: {error:#}");
                            led.set_low().ok();
                        }

                        FreeRtos::delay_ms(100);
                    }
                }
            }
        })?;

    let mut ping_sequence = 0u16;

    loop {
        {
            let state = saved_state.lock().expect("saved state mutex poisoned");

            for peer in &state.peers {
                if let Err(error) = esp_now.send_ping(peer.mac, ping_sequence) {
                    println!("Failed to send Ping to {:?}: {error:#}", peer.device);
                }

                ping_sequence = ping_sequence.wrapping_add(1);
            }
        }

        FreeRtos::delay_ms(500);
    }
}
