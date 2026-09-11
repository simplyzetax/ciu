# PCB layout

The KiCad board source is
`at86rf233-spi-carrier-board-for-esp32-c6-host.kicad_pcb` in this directory.

## Draft quality

The schematic footprints and nets have been imported. Placement and routing are
not complete. Route every electrical net and require zero ERC errors, zero DRC
violations, and zero unconnected items before treating the PCB as complete.

The existing 30 mm x 20 mm outline is allowed by `brief.md`. Connector ESD
protection is outside this prototype's scope.
