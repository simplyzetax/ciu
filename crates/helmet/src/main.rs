use ciu_esp32::{
    attachment::AttachmentPin,
    pairing::pair_as_helmet,
    saved_state::StateStore,
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
    let mut state = store.load()?;

    for peer in state.clone().peers {
        println!("Peer with device id {:?} is paired", peer.device);
    }

    let io = GpioWireIo::new(peripherals.pins.gpio4)?;
    let mut wire = Wire::new(io);
    // GPIO33 is unused elsewhere and is not a classic ESP32 boot-strapping or
    // flash pin. PAIR_DETECT is active-low through the goggle-side ground.
    let detect = AttachmentPin::new(peripherals.pins.gpio33)?;

    let _pairing_thread = std::thread::Builder::new()
        .name("pairing".into())
        .stack_size(PAIRING_STACK_SIZE)
        .spawn(move || {
            loop {
                detect.wait_for_attach();

                match pair_as_helmet(&mut wire, my_mac, &mut state, &mut store) {
                    Ok(goggle_mac) => {
                        led.set_high();
                        println!("Paired with goggle {:02X?}", goggle_mac);
                    }
                    Err(error) => {
                        println!("Pairing failed: {error:#}");
                    }
                }

                detect.wait_for_detach();
                led.set_low();
            }
        })?;

    loop {
        FreeRtos::delay_ms(500);
    }
}
