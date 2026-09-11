# Layout

The first-draft board retains the 30 mm × 20 mm outline and two-layer, 1.6 mm FR-4 stackup. J1 is rotated onto the top board axis so its rotated 23.86 mm courtyard fits inside the outline. U1 remains central; its supply bypass, AVDD bead, reset bias, crystal, and RF network are grouped around the corresponding pin sides. AE1 is on the right edge with a 4.5 mm × 9 mm F.Cu/B.Cu copper, via, footprint, and pour keepout around the antenna body; its feed pad is allowed inside the keepout.

Connector ESD protection is intentionally absent: docs/SPEC.md explicitly excludes it for this short-jumper prototype, so no unrequested protection parts were added at J1.

## Draft quality

Fine for draft review: the 30 mm × 20 mm outline, two-layer stackup, edge-mounted and axis-rotated J1, central U1 grouping, compact test-point access, right-edge antenna placement, and explicit two-layer antenna keepout.

Redo before fabrication: an RF specialist must replace the draft RF placement/routing with the Johanson/Microchip reference geometry, calculate the 50-ohm feed from the selected fabricator's actual dielectric and 1 oz copper stackup, tune C9/C10/L1/L2 using VNA data, confirm the antenna keepout against the exact 2450AT18A100 application layout, and review ground-via placement and return-current continuity. A human layout review must also optimize SPI/control routing, decoupling loop area, exposed-pad thermal vias, silkscreen readability, courtyard clearances, test-point ergonomics, and connector strain clearance. Connector ESD protection remains intentionally excluded unless the prototype cable length or ESD environment changes.
