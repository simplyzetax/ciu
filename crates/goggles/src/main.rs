use std::thread::Thread;

use ciu_esp32::{pairing::PairingPin, saved_state::StateStore};
use esp_idf_hal::{
    delay::FreeRtos,
    gpio::{AnyIOPin, PinDriver},
    peripherals::Peripherals,
};

fn main() -> anyhow::Result<()> {
    esp_idf_svc::sys::link_patches();

    let peripherals = Peripherals::take()?;

    let mut store = StateStore::new()?;
    let mut state = store.load()?;

    let pairing = PairingPin::new(peripherals.pins.gpio4)?;
    std::thread::spawn(move || {
        pairing
            .run(
                || {
                    println!("Helmet connected");
                    // initiate pairing
                },
                || {
                    println!("Helmet disconnected");
                },
            )
            .unwrap();
    });

    let _wifi = ciu_esp32::wifi::start(peripherals.modem, ciu_esp32::wifi::WIFI_CHANNEL)?;

    loop {
        std::thread::sleep(std::time::Duration::from_secs(1));
    }
}
