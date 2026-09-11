# Development and prototype plan

## Bring-up order

1. **Before power:** inspect U1, B1, Y1, AE1, the exposed pad, and all 0402 joints under magnification. With the board unpowered, verify J1 pin 1 (3V3) is not shorted to J1 pin 2 (GND), and check continuity from J1 pin 2 to the ground plane and U1 exposed pad. Do not apply power if resistance is unexpectedly low.
2. **Current-limited power:** leave the XIAO and RF firmware disconnected. Apply 3.3 V only at J1 pin 1 with ground at J1 pin 2, starting with a 25 mA current limit. Confirm correct polarity, no heating, and idle current below the 40 mA average-current budget. Raise the limit only when intentional TX testing requires it; TX incremental current must remain below 20 mA.
3. **Meter the rails first:** measure 3V3 at J1 pin 1 and U1 DVDD pins, then AVDD after FB1. Both rails should be near the applied 3.3 V; a large AVDD drop indicates a short, wrong bead, or assembly fault. There is no dedicated power test point, so use J1 and accessible U1/FB1 pads carefully.
4. **Check static controls:** confirm RESET and SLP_TR are at valid 3.3 V logic levels and never exceed the rail. Keep SLP_TR low, pulse RESET as specified by the firmware, and verify IRQ is not shorted to a rail.
5. **Connect the host:** use 3.3 V logic only. Default XIAO ESP32-C6 jumper wiring is MOSI GPIO18, MISO GPIO20, SCLK GPIO19, SEL GPIO17, IRQ GPIO2, RESET GPIO1, and SLP_TR GPIO0; do not use strapping GPIO8, GPIO9, or GPIO15. Share ground before connecting signals.
6. **Prove SPI at low speed:** start at 1 MHz and observe TP1 MOSI, TP2 MISO, TP3 SCLK, and TP4 SEL. Read `PART_NUM` first (expect `0x0b`), then `VERSION_NUM` (expect `0x01` or `0x02`). Do not trust later radio behavior until both reads pass repeatedly.
7. **Exercise control paths:** verify RESET causes the expected SPI reinitialization, SLP_TR enters and exits the intended state, and IRQ changes in response to a known transceiver event.
8. **RF smoke test:** use a shielded setup or approved test environment. Observe TP5 only with a high-impedance RF probe or 50-ohm instrument through an appropriate fixture; an ordinary oscilloscope probe will detune the feed. Confirm basic receive/transmit behavior at minimum power before increasing output.
9. **Characterize before release:** measure supply current in reset, idle/RX, and TX; verify the less-than-40 mA average target and less-than-20 mA TX increment. Then validate output spectrum, link performance, matching, and antenna behavior with suitable RF equipment.

## Test points and first checks

| Test point | Net | Check first |
| --- | --- | --- |
| TP1 | MOSI | No short to 3V3/GND unpowered; 0–3.3 V host data during SPI |
| TP2 | MISO | No short unpowered; 0–3.3 V U1 response during identity reads |
| TP3 | SCLK | Clean 1 MHz clock initially, with 0–3.3 V levels |
| TP4 | SEL | Idle and asserted levels match the driver; activity frames each SPI transfer |
| TP5 | RF_FEED | Continuity only before power; use RF-rated probing after power |

Power must be checked at J1 pin 1, J1 pin 2, and accessible U1/FB1 pads because the board has no dedicated 3V3, AVDD, or GND test point.

## Risks and stop conditions

- **RF layout is draft-only:** the 50-ohm feed is not certified. Stop fabrication release until the selected fabricator's two-layer 1.6 mm FR-4/1 oz stackup is known, impedance is calculated, and the Johanson/Microchip reference geometry and antenna keepout are reviewed.
- **Matching is unvalidated:** C9, C10, L1, L2, C11, and C12 are starting values. Tune only from VNA and conducted/radiated measurements; do not change them from software symptoms alone.
- **Parts are unverified:** every BOM MPN still requires distributor availability, datasheet revision, footprint/land-pattern, polarity, and lifecycle checks before ordering.
- **Crystal margin is unverified:** confirm Y1 load-capacitance compatibility and startup across voltage and temperature; C7/C8 may require adjustment.
- **Assembly sensitivity:** U1 exposed-pad soldering, the QFN perimeter, 0402 RF parts, balun orientation, and antenna placement need inspection. Assemble incrementally so a fault does not consume the full lot.
- **Power budget:** stop if average current reaches 40 mA, TX adds 20 mA or more, AVDD is materially below 3.3 V, or any part heats unexpectedly.
- **Host damage risk:** the header is 3.3 V-only and has no level shifting or connector ESD protection. Keep jumpers short and use an ESD-controlled bench.
- **Firmware status:** the ESP-IDF scaffold has not been compiled in Copperhead; run the documented pin-generation check and vendor build before hardware bring-up.

## Prototype order plan

1. Select a fabricator and obtain its actual two-layer stackup. Confirm the 30 mm × 20 mm outline, 1 oz copper assumption, RF impedance geometry, antenna keepout, Gerber/drill previews, and board-house DRC.
2. Verify every BOM line against the latest datasheet and footprint, especially U1, B1, AE1, Y1, and the RF passives. Approve alternates only after electrical, package, and RF review.
3. Order the smallest economical prototype lot (typically 5 boards), not a production quantity. Include spare RF passives around the C9/C10/L1/L2 nominal values for tuning.
4. Assemble one board first. Populate and inspect power/support circuitry before committing the remaining boards; then complete U1, crystal, digital interface, balun, matching network, and antenna.
5. Run the bring-up sequence above on the first assembly. If power and SPI identity pass, assemble two more boards to distinguish a repeatable design issue from a single assembly defect.
6. Reserve at least one board for conducted/VNA work and one for radiated/link testing. Record fitted matching values, stackup, measured currents, register identities, and test results per serial number.
7. Release a second spin or larger order only after ERC/DRC remain clean, all nets are routed, power limits pass, RF matching and antenna performance are reviewed, and the BOM/footprints are verified.
