use std::sync::{Arc, Mutex};

use ciu_core::iec::protocol::DeviceId;
use ciu_esp32::{
    attachment::AttachmentPin,
    pairing::pair_as_helmet,
    saved_state::{PeerRuntime, RuntimeState, StateStore},
    wire::{GpioWireIo, Wire},
};
use esp_idf_hal::{delay::FreeRtos, gpio::PinDriver, peripherals::Peripherals};

const APP_STACK_SIZE: usize = 10 * 1024;
const PAIRING_STACK_SIZE: usize = 8 * 1024;

fn main() {
    esp_idf_svc::sys::link_patches();

    std::thread::Builder::new()
        .name("helmet".into())
        .stack_size(APP_STACK_SIZE)
        .spawn(run)
        .expect("failed to start helmet task")
        .join()
        .expect("helmet task panicked")
        .expect("helmet initialization failed");
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

    esp_now.on_receive(
        Arc::clone(&saved_state),
        Arc::clone(&runtime_state),
        |device, message| {
            println!("Helmet ESP-NOW RX from {:?}: {:?}", device, message);
        },
    )?;

    let io = GpioWireIo::new(peripherals.pins.gpio4)?;
    let mut wire = Wire::new(io);

    let detect = AttachmentPin::new(peripherals.pins.gpio33)?;

    let pairing_saved_state = Arc::clone(&saved_state);
    let pairing_runtime_state = Arc::clone(&runtime_state);
    let pairing_esp_now = Arc::clone(&esp_now);

    let _pairing_thread = std::thread::Builder::new()
        .name("pairing".into())
        .stack_size(PAIRING_STACK_SIZE)
        .spawn(move || {
            loop {
                detect.wait_for_attach();

                let result = pair_as_helmet(&mut wire, my_mac, &pairing_saved_state, &mut store);

                match result {
                    Ok(goggle_mac) => {
                        if let Err(error) = pairing_esp_now.add_peer(goggle_mac) {
                            println!("Failed to register goggle with ESP-NOW: {error:#}");
                            led.set_low().ok();
                        } else {
                            led.set_high().ok();

                            println!("Paired with goggle {:02X?}", goggle_mac);

                            let mut runtime = pairing_runtime_state
                                .lock()
                                .expect("runtime state mutex poisoned");

                            if runtime.peer(DeviceId::Goggle).is_none() {
                                runtime.peers.push(PeerRuntime::new(DeviceId::Goggle));
                            }
                        }
                    }

                    Err(error) => {
                        println!("Pairing failed: {error:#}");
                    }
                }

                detect.wait_for_detach();
                led.set_low().ok();
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
