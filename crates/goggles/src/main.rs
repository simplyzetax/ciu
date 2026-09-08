use ciu_esp32::{pairing::PairingWire, saved_state::StateStore};

use esp_idf_hal::peripherals::Peripherals;

fn main() -> anyhow::Result<()> {
    esp_idf_svc::sys::link_patches();

    let peripherals = Peripherals::take()?;

    let _wifi = ciu_esp32::wifi::start(peripherals.modem, ciu_esp32::wifi::WIFI_CHANNEL)?;

    let my_mac = ciu_esp32::wifi::mac_address()?;

    let mut store = StateStore::new()?;
    let mut state = store.load()?;

    let io = GpioWireIo::new(peripherals.pins.gpio4)?;
    let mut pairing = PairingWire::new(io);

    std::thread::spawn(move || {
        loop {
            match pairing.pair_as_goggle(my_mac, &mut state, &mut store) {
                Ok(helmet_mac) => {
                    println!("Paired with helmet {:02X?}", helmet_mac);
                }

                Err(error) => {
                    println!("Pairing failed: {error}");
                }
            }
        }
    });

    loop {
        std::thread::park();
    }
}
