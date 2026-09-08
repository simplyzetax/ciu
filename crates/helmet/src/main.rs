use ciu_esp32::{
    attachment::AttachmentPin,
    pairing::pair_as_helmet,
    saved_state::StateStore,
    wire::{GpioWireIo, Wire},
};
use esp_idf_hal::{delay::FreeRtos, gpio::PinDriver, peripherals::Peripherals};

fn main() -> anyhow::Result<()> {
    esp_idf_svc::sys::link_patches();

    let peripherals = Peripherals::take()?;
    let mut led = PinDriver::output(peripherals.pins.gpio2)?;

    let wifi = ciu_esp32::wifi::start(peripherals.modem, ciu_esp32::wifi::WIFI_CHANNEL)?;
    let my_mac = ciu_esp32::wifi::station_mac(&wifi)?;

    let mut store = StateStore::new()?;
    let mut state = store.load()?;
    let io = GpioWireIo::new(peripherals.pins.gpio4)?;
    let mut wire = Wire::new(io);
    // GPIO33 is unused elsewhere and is not a classic ESP32 boot-strapping or
    // flash pin. PAIR_DETECT is active-low through the goggle-side ground.
    let detect = AttachmentPin::new(peripherals.pins.gpio33)?;

    let _pairing_thread = std::thread::spawn(move || {
        loop {
            detect.wait_for_attach();

            match pair_as_helmet(&mut wire, my_mac, &mut state, &mut store) {
                Ok(goggle_mac) => {
                    println!("Paired with goggle {:02X?}", goggle_mac);
                }
                Err(error) => {
                    println!("Pairing failed: {error:#}");
                }
            }

            detect.wait_for_detach();
        }
    });

    loop {
        led.set_high()?;
        FreeRtos::delay_ms(500);

        led.set_low()?;
        FreeRtos::delay_ms(500);
    }
}
