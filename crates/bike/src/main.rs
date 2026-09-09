use esp_idf_hal::{delay::FreeRtos, gpio::PinDriver, peripherals::Peripherals};

fn main() -> anyhow::Result<()> {
    esp_idf_svc::sys::link_patches();

    let peripherals = Peripherals::take()?;
    let mut led = PinDriver::output(peripherals.pins.gpio2)?;

    let _wifi = ciu_esp32::wifi::start(peripherals.modem)?;

    loop {
        led.set_high()?;
        FreeRtos::delay_ms(500);

        led.set_low()?;
        FreeRtos::delay_ms(500);
    }
}
