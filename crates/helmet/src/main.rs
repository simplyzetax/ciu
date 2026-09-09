use std::{
    sync::{Arc, Mutex},
    time::{Duration, Instant},
};

use ciu_core::iec::protocol::{BikeSnapshot, DeviceId, Message};
use ciu_esp32::{
    attachment::AttachmentPin,
    network::Network,
    pairing::pair_as_helmet,
    run_app, spawn_pairing_task,
    wire::{GpioWireIo, Wire},
};
use esp_idf_hal::{delay::FreeRtos, gpio::PinDriver, peripherals::Peripherals};

const SNAPSHOT_INTERVAL_MS: u32 = 100;
const PING_INTERVAL: Duration = Duration::from_millis(500);

fn main() {
    run_app("helmet", run);
}

fn run() -> anyhow::Result<()> {
    let peripherals = Peripherals::take()?;
    let mut led = PinDriver::output(peripherals.pins.gpio2)?;

    let wifi = ciu_esp32::wifi::start(peripherals.modem)?;
    let my_mac = ciu_esp32::wifi::station_mac(&wifi)?;

    let network = Network::new()?;
    let bike_state = Arc::new(Mutex::new(BikeSnapshot::default()));
    let received_bike_state = Arc::clone(&bike_state);

    network.on_message(move |device, snapshot| {
        println!("Helmet ESP-NOW RX from {:?}: {:?}", device, snapshot);

        if device == DeviceId::Bike {
            received_bike_state
                .lock()
                .expect("bike state mutex poisoned")
                .merge(snapshot);
        }
    })?;

    let io = GpioWireIo::new(peripherals.pins.gpio4)?;
    let mut wire = Wire::new(io);
    let detect = AttachmentPin::new(peripherals.pins.gpio33)?;
    let pairing_network = Arc::clone(&network);

    let _pairing_thread = spawn_pairing_task(move || {
        loop {
            detect.wait_for_attach();

            let result = pair_as_helmet(&mut wire, my_mac, &pairing_network);

            match result {
                Ok(goggle_mac) => {
                    println!("Paired with goggle {:02X?}", goggle_mac);
                    led.set_high().ok();
                }
                Err(error) => {
                    println!("Pairing failed: {error:#}");
                }
            }

            detect.wait_for_detach();
            led.set_low().ok();
        }
    })?;

    let mut ping_sequence = 0;
    let mut last_ping = Instant::now();

    loop {
        let snapshot = *bike_state.lock().expect("bike state mutex poisoned");

        if let Err(error) = network.send(DeviceId::Goggle, &Message::BikeSnapshot(snapshot)) {
            println!("Failed to send bike snapshot to Goggles: {error:#}");
        }

        if last_ping.elapsed() >= PING_INTERVAL {
            network.ping_all(&mut ping_sequence);
            last_ping = Instant::now();
        }

        FreeRtos::delay_ms(SNAPSHOT_INTERVAL_MS);
    }
}
