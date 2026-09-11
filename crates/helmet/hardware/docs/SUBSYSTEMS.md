# Subsystems

## Block diagram

An external 3.3 V Seeed XIAO ESP32-C6 host connects through a 2.54 mm header to U1, an AT86RF233-ZU. The host supplies 3V3 and GND and exchanges SPI (MOSI, MISO, SCLK, /SEL) plus IRQ, RESET, and SLP_TR. U1 uses the datasheet basic application circuitry—local decoupling, DVDD/AVDD filtering, crystal, reset, and control signals—then drives the Microchip reference matching network and a 2.4 GHz antenna. Header and RF test points support bring-up.

## Power

One externally supplied 3.3 V rail powers the carrier; no regulator, charger, battery circuitry, or power-path control is included. This keeps the board within its stated scope and avoids regulator quiescent current. The eventual assembled design must remain below 40 mA average and add less than 20 mA during TX; component selection and the host supply must be checked against both limits.

## MCU host

The MCU is an externally wired Seeed XIAO ESP32-C6, not a populated board subsystem. Header logic is 3.3 V only and has no level shifting. This preserves the 25 mm × 20 mm carrier target and makes the host replaceable without changing the RF design.

## Connectivity and control

The 2.54 mm header exposes 3V3, GND, MOSI, MISO, SCLK, /SEL, IRQ, RESET, and SLP_TR. SPI provides register access, IRQ reports radio events, and RESET/SLP_TR provide the mandatory transceiver control paths. MOSI, MISO, SCLK, and /SEL each have a test point for host-interface debugging.

## Radio and RF

U1 is exclusively the QFN-32 AT86RF233-ZU; AT86RF231, AT86RF212, and other transceiver families are excluded. The radio implements the AT86RF233 datasheet basic application circuit: RFP and RFN each connect through a 22 pF C0G series capacitor to the differential side of a Johanson 2450BM15A0015 balun/filter. The balun's 50-ohm single-ended output drives the chip-antenna matching network. Neither RF pin may be left unconnected. The antenna feed is 50 ohm controlled impedance, with a test point at the feed. The target board is two-layer, 1.6 mm FR-4 with assumed 1 oz copper; the selected fabricator stackup must confirm the feed geometry before layout release.

B1 uses the installed `Connector_Generic:Conn_01x06` symbol as a pin-accurate schematic representation and the project footprint `ciu:Johanson_2450BM15A0015`. Manufacturer pins are 1 unbalanced, 2 ground, 3 balanced, 4 balanced, 5 ground, and 6 ground.

## UI and audio

There is no onboard UI, audio codec, microphone, speaker, display, or user input. These functions remain with the external host or the end product, keeping the carrier focused on radio access.

## Mechanical and bring-up

The board target is at most 25 mm × 20 mm and uses a two-layer stackup unless documented RF evidence requires an exception. Initial bring-up reads `PART_NUM` (expected `0x0b`) and `VERSION_NUM` (expected `0x01` or `0x02`) over SPI before radio operation is trusted.
