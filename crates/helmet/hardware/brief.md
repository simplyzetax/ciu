# AT86RF233 SPI carrier board for ESP32-C6 host

A minimal 2-layer carrier board that breaks out a Microchip AT86RF233-ZU
2.4 GHz O-QPSK transceiver over SPI to a host MCU. The host is an ESP32-C6
(Seeed XIAO ESP32-C6 module, not populated on this board) which does not
have native access to the AT86RF233's proprietary high-rate O-QPSK PHY
modes. This board exists purely to give SPI/GPIO access to a real
AT86RF233, so firmware can drive it into non-standard configurations
(custom SFD value, non-standard OQPSK_DATA_RATE) to receive and transmit
Sena Mesh (30K) and Cardo DMC packets.

## Scope

- AT86RF233-ZU transceiver, QFN-32, with full basic application circuit
  from the datasheet (decoupling, DVDD/AVDD filtering, crystal, RESET,
  SLP_TR, IRQ).
- 2.4 GHz chip antenna (or PCB inverted-F antenna if a suitable reference
  layout is available) with matching network per Microchip's reference
  design. Antenna feed must be a controlled-impedance 50 ohm trace.
- SPI interface (MOSI, MISO, SCLK, /SEL) plus IRQ, RESET, and SLP_TR
  broken out to a 2.54mm pin header, pin-compatible in spirit with a
  Seeed XIAO ESP32-C6 wired by jumper wires (no XIAO on this board, no
  onboard MCU at all).
- Power input: single 3.3V rail from an external source via the same
  header (board does not regulate its own power; host supplies 3.3V).
- No battery, no charger, no display, no audio path on this board — those
  live on separate modules wired externally.
- Test points on SPI lines and on the antenna feed for probing during
  bring-up.
- This is a short-jumper prototype carrier. Connector ESD protection is not
  required; do not add ESD parts unless explicitly requested later.

## Explicit non-goals

- No onboard ESP32 / microcontroller.
- No onboard voltage regulation or battery management.
- No audio codec, microphone, or speaker circuitry.
- No enclosure or mechanical design.
- Do not substitute a different 2.4 GHz transceiver family (must be
  AT86RF233-ZU specifically, not AT86RF231 or AT86RF212).

## Budgets

- Board outline: target 25mm x 20mm if practical. The existing 30mm x 20mm
  outline is acceptable when the header, antenna keepout, and test points do
  not fit safely in the target size.
- Supply: single 3.3V rail, target current draw under 40mA average,
  peak under 20mA extra during TX per datasheet.
- Layer count: 2 layers, standard 1.6mm FR4, unless the antenna matching
  network genuinely requires more layers to hit a controlled 50 ohm
  trace — flag this explicitly if so rather than silently going to 4
  layers.

## Reference material

- Microchip AT86RF233 datasheet (Atmel-8351), section on the basic
  application schematic and PCB antenna reference layout.
- Part number for the transceiver: AT86RF233-ZU.

## Deliverables

- KiCad schematic and PCB layout, ERC/DRC clean. Route every electrical net;
  unconnected ratsnest items are not acceptable in the completed layout.
- Gerbers, drill files, and STEP model.
- BOM with real, sourceable part numbers and datasheet references for
  every part, especially the antenna matching network components.
- A short bring-up note describing how to verify SPI communication by
  reading the AT86RF233 PART_NUM (expect 0x0b) and VERSION_NUM
  (expect 0x01 or 0x02) registers before anything else is trusted.
