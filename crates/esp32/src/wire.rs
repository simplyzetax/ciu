use std::time::{Duration, Instant};

use ciu_core::iec::protocol::{Message, PairingMessage, WireMessage};
use esp_idf_hal::{
    delay::{Ets, FreeRtos},
    gpio::{InputOutput, InputPin, OutputPin, PinDriver, Pull},
};

const MAX_PAYLOAD_SIZE: usize = 250;
const FRAME_SYNC: [u8; 2] = [0xA5, 0x5A];
const MAX_FRAME_SIZE: usize = FRAME_SYNC.len() + 1 + MAX_PAYLOAD_SIZE + 1;

const BIT_TIME_US: u32 = 500;
const START_POLL_US: u32 = 25;
const WAKE_PULSE_MS: u32 = 20;
const MIN_WAKE_PULSE: Duration = Duration::from_millis(10);
const START_GAP_US: u32 = 20_000;
const FIRST_BYTE_TIMEOUT: Duration = Duration::from_millis(30);
const INTER_BYTE_TIMEOUT: Duration = Duration::from_millis(2);
pub const DEFAULT_RECEIVE_TIMEOUT: Duration = Duration::from_secs(2);

/// One complete physical-wire frame transport.
///
/// Implementations delimit frame bursts and must return within `timeout` when
/// no complete burst arrives.
pub trait WireIo {
    fn write_frame(&mut self, data: &[u8]) -> anyhow::Result<()>;
    fn read_frame(&mut self, data: &mut [u8], timeout: Duration) -> anyhow::Result<usize>;
}

/// Open-drain, software-UART transport for the shared PAIR_DATA GPIO.
///
/// A high output releases the line; a low output pulls it down. Both devices
/// must share ground and use an external pull-up to 3.3 V. The internal pull-up
/// is enabled only so a disconnected input has a defined idle state.
pub struct GpioWireIo<'d> {
    pin: PinDriver<'d, InputOutput>,
}

impl<'d> GpioWireIo<'d> {
    pub fn new<T>(pin: T) -> anyhow::Result<Self>
    where
        T: InputPin + OutputPin + 'd,
    {
        let mut pin = PinDriver::input_output_od(pin, Pull::Up)?;
        pin.set_high()?;

        Ok(Self { pin })
    }

    fn write_uart_byte(&mut self, byte: u8) -> anyhow::Result<()> {
        self.pin.set_low()?;
        Ets::delay_us(BIT_TIME_US);

        for bit in 0..8 {
            if byte & (1 << bit) == 0 {
                self.pin.set_low()?;
            } else {
                self.pin.set_high()?;
            }

            Ets::delay_us(BIT_TIME_US);
        }

        self.pin.set_high()?;
        Ets::delay_us(BIT_TIME_US);

        Ok(())
    }

    fn read_uart_byte(&self, timeout: Duration) -> anyhow::Result<Option<u8>> {
        let deadline = Instant::now() + timeout;

        loop {
            while self.pin.is_high() {
                if Instant::now() >= deadline {
                    return Ok(None);
                }

                Ets::delay_us(START_POLL_US);
            }

            Ets::delay_us(BIT_TIME_US / 2);

            if self.pin.is_high() {
                continue;
            }

            let mut byte = 0u8;

            for bit in 0..8 {
                Ets::delay_us(BIT_TIME_US);

                if self.pin.is_high() {
                    byte |= 1 << bit;
                }
            }

            Ets::delay_us(BIT_TIME_US);

            if self.pin.is_low() {
                anyhow::bail!("wire UART stop bit was low");
            }

            return Ok(Some(byte));
        }
    }

    fn wait_for_wake(&self, timeout: Duration) -> anyhow::Result<()> {
        let deadline = Instant::now() + timeout;

        loop {
            while self.pin.is_high() {
                if Instant::now() >= deadline {
                    anyhow::bail!("wire receive timed out");
                }

                FreeRtos::delay_ms(1);
            }

            let low_since = Instant::now();

            while self.pin.is_low() {
                if Instant::now() >= deadline {
                    anyhow::bail!("wire receive timed out during wake pulse");
                }

                if low_since.elapsed() >= MIN_WAKE_PULSE {
                    while self.pin.is_low() {
                        if Instant::now() >= deadline {
                            anyhow::bail!("wire receive timed out during wake pulse");
                        }

                        FreeRtos::delay_ms(1);
                    }

                    return Ok(());
                }

                FreeRtos::delay_ms(1);
            }
        }
    }
}

impl WireIo for GpioWireIo<'_> {
    fn write_frame(&mut self, data: &[u8]) -> anyhow::Result<()> {
        if data.is_empty() {
            anyhow::bail!("wire frame is empty");
        }

        self.pin.set_high()?;
        FreeRtos::delay_ms(2);

        if self.pin.is_low() {
            anyhow::bail!("wire is busy");
        }

        let result: anyhow::Result<()> = (|| {
            self.pin.set_low()?;
            FreeRtos::delay_ms(WAKE_PULSE_MS);
            self.pin.set_high()?;
            Ets::delay_us(START_GAP_US);

            for &byte in data {
                self.write_uart_byte(byte)?;
            }

            Ok(())
        })();

        let release_result = self.pin.set_high().map_err(anyhow::Error::from);
        result.and(release_result)
    }

    fn read_frame(&mut self, data: &mut [u8], timeout: Duration) -> anyhow::Result<usize> {
        if data.is_empty() {
            anyhow::bail!("wire receive buffer is empty");
        }

        self.pin.set_high()?;
        self.wait_for_wake(timeout)?;

        let Some(first_byte) = self.read_uart_byte(FIRST_BYTE_TIMEOUT)? else {
            anyhow::bail!("wire frame contained no data");
        };

        data[0] = first_byte;
        let mut length = 1;

        while length < data.len() {
            let Some(byte) = self.read_uart_byte(INTER_BYTE_TIMEOUT)? else {
                return Ok(length);
            };

            data[length] = byte;
            length += 1;
        }

        if self.read_uart_byte(INTER_BYTE_TIMEOUT)?.is_some() {
            anyhow::bail!("wire frame exceeds receive buffer");
        }

        Ok(length)
    }
}

/// Encodes and frames CIU messages for the physical wire.
pub struct Wire<Io> {
    io: Io,
}

impl<Io> Wire<Io>
where
    Io: WireIo,
{
    pub fn new(io: Io) -> Self {
        Self { io }
    }

    pub fn send(&mut self, message: &WireMessage) -> anyhow::Result<()> {
        let mut frame = [0u8; MAX_FRAME_SIZE];
        frame[..FRAME_SYNC.len()].copy_from_slice(&FRAME_SYNC);

        let payload_start = FRAME_SYNC.len() + 1;
        let payload_length = message.encode(&mut frame[payload_start..MAX_FRAME_SIZE - 1])?;
        let payload_length = u8::try_from(payload_length)
            .map_err(|_| anyhow::anyhow!("wire payload is too large"))?;

        frame[FRAME_SYNC.len()] = payload_length;

        let checksum_index = payload_start + payload_length as usize;
        frame[checksum_index] = crc8(&frame[FRAME_SYNC.len()..checksum_index]);

        self.io.write_frame(&frame[..=checksum_index])
    }

    pub fn send_message(&mut self, message: &Message) -> anyhow::Result<()> {
        self.send(&WireMessage::Message(*message))
    }

    pub fn send_pairing(&mut self, message: &PairingMessage) -> anyhow::Result<()> {
        self.send(&WireMessage::Pairing(*message))
    }

    pub fn receive(&mut self) -> anyhow::Result<WireMessage> {
        self.receive_timeout(DEFAULT_RECEIVE_TIMEOUT)
    }

    pub fn receive_timeout(&mut self, timeout: Duration) -> anyhow::Result<WireMessage> {
        let mut frame = [0u8; MAX_FRAME_SIZE];
        let frame_length = self.io.read_frame(&mut frame, timeout)?;

        if frame_length < FRAME_SYNC.len() + 3 {
            anyhow::bail!("wire frame is truncated");
        }

        if frame[..FRAME_SYNC.len()] != FRAME_SYNC {
            anyhow::bail!("invalid wire frame sync");
        }

        let payload_length = frame[FRAME_SYNC.len()] as usize;

        if payload_length == 0 || payload_length > MAX_PAYLOAD_SIZE {
            anyhow::bail!("invalid wire payload length: {payload_length}");
        }

        let payload_start = FRAME_SYNC.len() + 1;
        let checksum_index = payload_start + payload_length;
        let expected_frame_length = checksum_index + 1;

        if frame_length != expected_frame_length {
            anyhow::bail!(
                "invalid wire frame length: expected {expected_frame_length}, got {frame_length}"
            );
        }

        let expected_checksum = crc8(&frame[FRAME_SYNC.len()..checksum_index]);

        if frame[checksum_index] != expected_checksum {
            anyhow::bail!("invalid wire frame checksum");
        }

        Ok(WireMessage::decode(&frame[payload_start..checksum_index])?)
    }

    pub fn into_inner(self) -> Io {
        self.io
    }
}

fn crc8(data: &[u8]) -> u8 {
    let mut crc = 0u8;

    for &byte in data {
        crc ^= byte;

        for _ in 0..8 {
            crc = if crc & 0x80 == 0 {
                crc << 1
            } else {
                (crc << 1) ^ 0x07
            };
        }
    }

    crc
}
