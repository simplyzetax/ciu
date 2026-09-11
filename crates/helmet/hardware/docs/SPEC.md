# AT86RF233 SPI Carrier Board Specification

## Device

A minimal 2-layer carrier board provides SPI/GPIO access to a Microchip AT86RF233-ZU 2.4 GHz O-QPSK transceiver from an externally wired Seeed XIAO ESP32-C6 host. The XIAO is not populated on this board; firmware uses the AT86RF233 proprietary configuration registers for Sena Mesh (30K) and Cardo DMC experiments.

## Required circuitry

- U1 shall be an AT86RF233-ZU in QFN-32; AT86RF231, AT86RF212, and other transceiver families are forbidden.
- The schematic shall implement the datasheet basic application circuit: decoupling, DVDD/AVDD filtering, crystal, RESET, SLP_TR, and IRQ.
- A 2.4 GHz chip antenna or a PCB inverted-F antenna with a suitable reference layout shall use the Microchip reference matching network.
- The antenna feed shall be controlled impedance, 50 ohm.
- A 2.54 mm header shall expose 3V3, GND, MOSI, MISO, SCLK, /SEL, IRQ, RESET, and SLP_TR.
- Test points shall be present on MOSI, MISO, SCLK, /SEL, and the antenna feed.

## Constraints and budgets

| Constraint | Requirement | Status |
| --- | --- | --- |
| Supply | One externally supplied 3.3 V rail only; no onboard regulator | Stated |
| Average current | Less than 40 mA target | Stated |
| TX incremental peak current | Less than 20 mA additional during TX | Stated |
| Board outline | At most 25 mm x 20 mm where practicable; smaller is acceptable | Stated |
| Stackup | Two layers, standard 1.6 mm FR-4 | Stated |
| RF exception | More than two layers is permitted only when required to realize the 50 ohm antenna feed/matching network, and must be explicitly flagged | Stated |
| Copper weight | 1 oz copper assumed for initial 50 ohm impedance calculation; confirm against the selected fabricator stackup before layout release | ASSUMED |
| Host logic level | Header SPI/GPIO signals are 3.3 V logic only; no level shifting is included | ASSUMED |

## Explicit exclusions

No onboard ESP32 or other MCU, voltage regulation, battery management, charger, audio codec, microphone, speaker, enclosure, or mechanical product design is in scope.

## Deliverables

- KiCad schematic and PCB, ERC/DRC clean.
- Gerbers, drill files, DXF outline, and STEP model.
- BOM with real sourceable part numbers and datasheet references, including RF matching components.
- Bring-up note: first read `PART_NUM` (expect `0x0b`) and `VERSION_NUM` (expect `0x01` or `0x02`) over SPI before trusting further operation.

## References

- Microchip AT86RF233 datasheet, Atmel-8351: basic application schematic and PCB antenna reference layout.
- Product brief: `brief.md`.

## Decisions

- Use the datasheet reference RF topology and retain a two-layer board target; this minimizes implementation risk while preserving the stated size and cost goal.
