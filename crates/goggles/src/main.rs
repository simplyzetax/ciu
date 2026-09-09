use std::sync::Arc;

use ciu_core::iec::protocol::DeviceId;
use ciu_esp32::{
    network::Network,
    pairing::pair_as_goggle,
    run_app, spawn_pairing_task,
    wire::{GpioWireIo, Wire},
};
use esp_idf_hal::{delay::FreeRtos, gpio::PinDriver, peripherals::Peripherals};

fn main() {
    run_app("goggles", run);
}

fn run() -> anyhow::Result<()> {
    let peripherals = Peripherals::take()?;
    let mut led = PinDriver::output(peripherals.pins.gpio2)?;

    let wifi = ciu_esp32::wifi::start(peripherals.modem)?;
    let my_mac = ciu_esp32::wifi::station_mac(&wifi)?;

    let network = Network::new()?;
    network.on_message(|device, snapshot| {
        if device == DeviceId::Helmet {
            //TODO: Display in HUD when OLED arrives
            println!("Goggles HUD snapshot: {:?}", snapshot);
        }
    })?;

    let io = GpioWireIo::new(peripherals.pins.gpio4)?;
    let mut wire = Wire::new(io);
    let pairing_network = Arc::clone(&network);

    let _pairing_thread = spawn_pairing_task(move || {
        loop {
            let result = pair_as_goggle(&mut wire, my_mac, &pairing_network);

            match result {
                Ok(helmet_mac) => {
                    println!("Paired with helmet {:02X?}", helmet_mac);
                    led.set_high().ok();
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

    let mut ping_sequence = 0;

    loop {
        network.ping_all(&mut ping_sequence);
        FreeRtos::delay_ms(500);
    }
}
