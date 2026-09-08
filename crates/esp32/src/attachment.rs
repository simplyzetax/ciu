use std::time::{Duration, Instant};

use esp_idf_hal::{
    delay::FreeRtos,
    gpio::{Input, InputPin, PinDriver, Pull},
};

const DEBOUNCE_TIME: Duration = Duration::from_millis(50);
const POLL_INTERVAL_MS: u32 = 5;

/// Active-low physical attachment input for the dedicated PAIR_DETECT signal.
///
/// The helmet side requires an external 10 kΩ pull-up to 3.3 V. The internal
/// pull-up is also enabled so the input remains detached-high by default.
pub struct AttachmentPin<'d> {
    pin: PinDriver<'d, Input>,
}

impl<'d> AttachmentPin<'d> {
    pub fn new<T>(pin: T) -> anyhow::Result<Self>
    where
        T: InputPin + 'd,
    {
        Ok(Self {
            pin: PinDriver::input(pin, Pull::Up)?,
        })
    }

    pub fn is_attached(&self) -> bool {
        self.pin.is_low()
    }

    pub fn wait_for_attach(&self) {
        self.wait_for_state(true);
    }

    pub fn wait_for_detach(&self) {
        self.wait_for_state(false);
    }

    fn wait_for_state(&self, attached: bool) {
        loop {
            while self.is_attached() != attached {
                FreeRtos::delay_ms(POLL_INTERVAL_MS);
            }

            let stable_since = Instant::now();

            while self.is_attached() == attached {
                if stable_since.elapsed() >= DEBOUNCE_TIME {
                    return;
                }

                FreeRtos::delay_ms(POLL_INTERVAL_MS);
            }
        }
    }
}
