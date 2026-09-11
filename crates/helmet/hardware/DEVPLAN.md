# Development plan

## Firmware scaffold

The firmware is a minimal ESP-IDF project for the external Seeed XIAO ESP32-C6. `firmware/main/pins.h` is generated from the J1 table in `docs/PINOUT.md`; run `python3 firmware/generate_pins.py --check` to detect drift and rerun without `--check` to regenerate it.

The default jumper wiring is MOSI=GPIO18, MISO=GPIO20, SCLK=GPIO19, SEL=GPIO17, IRQ=GPIO2, RESET=GPIO1, and SLP_TR=GPIO0. These are host-side wiring choices, adjustable through menuconfig, and avoid ESP32-C6 strapping GPIO8, GPIO9, and GPIO15. The carrier remains 3.3 V logic only.

The happy path initializes SPI at 1 MHz, holds SLP_TR low, pulses RESET, then reads `PART_NUM` (expected `0x0b`) and `VERSION_NUM` (expected `0x01` or `0x02`). Further radio configuration is intentionally left as driver stubs until bring-up succeeds.

Build status: not compiled here. The available Copperhead tool catalog does not expose the ESP-IDF toolchain. Verify with:

```sh
cd firmware
python3 generate_pins.py --check
idf.py set-target esp32c6
idf.py build
```
