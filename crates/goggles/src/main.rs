use ciu_esp32::{
    pairing::pair_as_goggle,
    saved_state::StateStore,
    wire::{GpioWireIo, Wire},
};
use esp_idf_hal::{delay::FreeRtos, gpio::PinDriver, peripherals::Peripherals};

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
    esp_idf_svc::sys::link_patches();

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

    let _pairing_thread = std::thread::Builder::new()
        .name("pairing".into())
        .stack_size(PAIRING_STACK_SIZE)
        .spawn(move || {
            loop {
                match pair_as_goggle(&mut wire, my_mac, &mut state, &mut store) {
                    Ok(helmet_mac) => {
                        println!("Paired with helmet {:02X?}", helmet_mac);
                        led.set_high();
                    }
                    Err(error) => {
                        let is_timeout = error
                            .chain()
                            .any(|cause| cause.to_string().contains("wire receive timed out"));

                        if !is_timeout {
                            println!("Pairing failed: {error:#}");
                        }

                        FreeRtos::delay_ms(100);
                    }
                }
            }
        })?;

    loop {
        FreeRtos::delay_ms(500);
    }
}
