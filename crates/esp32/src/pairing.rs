use core::num::NonZero;

use esp_idf_hal::{
    delay::FreeRtos,
    gpio::{Input, InputPin, InterruptType, PinDriver, Pull},
    task::notification::Notification,
};

pub struct PairingPin {
    pin: PinDriver<'static, Input>,
}

impl PairingPin {
    pub fn new(pin: impl InputPin + 'static) -> anyhow::Result<Self> {
        let mut pin = PinDriver::input(pin, Pull::Up)?;

        pin.set_interrupt_type(InterruptType::AnyEdge)?;

        Ok(Self { pin })
    }

    pub fn run<Connected, Disconnected>(
        mut self,
        mut on_connected: Connected,
        mut on_disconnected: Disconnected,
    ) -> anyhow::Result<()>
    where
        Connected: FnMut(),
        Disconnected: FnMut(),
    {
        loop {
            let notification = Notification::new();
            let notifier = notification.notifier();

            unsafe {
                self.pin.subscribe_nonstatic(move || {
                    notifier.notify(NonZero::new(1).unwrap());
                })?;
            }

            self.pin.enable_interrupt()?;

            // Sleep until the GPIO interrupt fires.
            notification.wait_any();

            // Give the magnetic contact some time to settle.
            FreeRtos::delay_ms(30);

            if self.pin.is_low() {
                on_connected();
            } else {
                on_disconnected();
            }
        }
    }
}
