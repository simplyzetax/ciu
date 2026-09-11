# Design changelog

## 2026-09-11 — create pipeline stage: devplan

- Change: devplan
- Files: docs/DEVPLAN.md, docs/CHANGELOG.md, docs/DECISIONS.md
- Verification: ERC clean, DRC clean

## 2026-09-11 — create pipeline stage: devplan

- Change: devplan
- Files: docs/DEVPLAN.md, docs/DECISIONS.md, docs/CHANGELOG.md
- Verification: ERC clean, DRC clean, documentation drift checked

## 2026-09-11 — create pipeline stage: firmware

- Change: firmware
- Files: firmware/CMakeLists.txt, firmware/main/CMakeLists.txt, firmware/main/Kconfig.projbuild, firmware/generate_pins.py, firmware/main/pins.h, firmware/main/at86rf233.h, firmware/main/at86rf233.c, firmware/main/main.c, DEVPLAN.md, docs/CHANGELOG.md, docs/DECISIONS.md
- Verification: ERC clean, DRC clean

## 2026-09-11 — create pipeline stage: firmware

- Change: firmware
- Files: firmware/, DEVPLAN.md, docs/DECISIONS.md, docs/CHANGELOG.md
- Verification: pins.h generated from docs/PINOUT.md; ESP-IDF toolchain unavailable through Copperhead, so not compiled here; documentation drift checked

## 2026-09-11 — create pipeline stage: outputs

- Change: outputs
- Files: outputs/, outputs/BOM.csv, docs/DECISIONS.md, docs/CHANGELOG.md
- Verification: ERC clean, DRC clean

## 2026-09-11 — create pipeline stage: outputs

- Change: outputs
- Files: outputs/ Gerbers and drill files, outputs/outline.dxf, outputs/board.step, outputs/board.svg, outputs/schematic.svg, outputs/BOM.csv, docs/DECISIONS.md, docs/CHANGELOG.md
- Verification: every fabrication, mechanical, and SVG export succeeded; ordering BOM generated from docs/BOM.md; ERC clean; DRC clean; legibility has 0 errors (2 non-blocking advisories); documentation drift clean

## 2026-09-11 — create pipeline stage: part-selection

- Change: part-selection
- Files: docs/CHANGELOG.md, docs/DECISIONS.md
- Verification: ERC not required

## 2026-09-11 — verify pipeline stage: part-selection

- Change: part-selection
- Files: docs/BOM.md, docs/DECISIONS.md, docs/CHANGELOG.md
- Verification: BOM non-empty with exact five-column format and one row per refdes; U1, B1, AE1, J1, Y1, FB1, and TP1-TP5 installed symbols and real pins confirmed; all MPNs remain UNVERIFIED; current-budget impacts reviewed; documentation drift checked

## 2026-09-11 — create pipeline stage: part-selection

- Change: part-selection
- Files: docs/BOM.md, docs/DECISIONS.md, docs/CHANGELOG.md, schematic.intent.json, docs/PINOUT.md, at86rf233-spi-carrier-board-for-esp32-c6-host.kicad_sch
- Verification: ERC clean

## 2026-09-11 — create pipeline stage: part-selection

- Change: part-selection
- Files: docs/BOM.md, docs/PINOUT.md, docs/DECISIONS.md, docs/CHANGELOG.md, schematic.intent.json, at86rf233-spi-carrier-board-for-esp32-c6-host.kicad_sch
- Verification: installed symbols and real pins confirmed for U1, B1, J1, Y1, AE1, FB1, and TP1-TP5; every MPN marked UNVERIFIED; ERC, symbol, legibility, and drift checks run

Append-only, newest first. One entry per committed copperhead run.

## 2026-09-11 — create pipeline stage: schematic

- Change: schematic
- Files: schematic.intent.json, at86rf233-spi-carrier-board-for-esp32-c6-host.kicad_sch, docs/PINOUT.md, docs/DECISIONS.md
- Verification: ERC clean

## 2026-09-11 — resolve the text-collision legibility finding between #PWR06 and the AVDD label near (303.53, 108.2) by moving the AVDD label clear of the 3V3 value text

- Change: clear-avdd-label-collision
- Files: docs/CHANGELOG.md, docs/DECISIONS.md, at86rf233-spi-carrier-board-for-esp32-c6-host.kicad_sch
- Verification: ERC clean

## 2026-09-11 — clear AVDD label collision

- Change: clear-avdd-label-collision
- Files: at86rf233-spi-carrier-board-for-esp32-c6-host.kicad_sch, docs/DECISIONS.md, docs/CHANGELOG.md
- Verification: ERC clean; legibility has 0 errors (2 non-blocking advisories); documentation drift clean

## 2026-09-11 — create pipeline stage: part-selection

- Change: part-selection
- Files: docs/BOM.md, docs/DECISIONS.md, docs/CHANGELOG.md
- Verification: ERC not required

## 2026-09-11 — create pipeline stage: part-selection

- Change: part-selection
- Files: docs/BOM.md, docs/DECISIONS.md, docs/CHANGELOG.md
- Verification: installed symbols and pins confirmed for U1, J1, Y1, AE1, and TP1-TP5; no schematic exists, so ERC/DRC are not applicable

## 2026-09-11 — create pipeline stage: architecture

- Change: architecture
- Files: docs/SUBSYSTEMS.md, docs/CHANGELOG.md, docs/DECISIONS.md
- Verification: ERC not required

## 2026-09-11 — create pipeline stage: architecture

- Change: architecture
- Files: docs/SUBSYSTEMS.md, docs/DECISIONS.md, docs/CHANGELOG.md
- Verification: documentation reviewed; ERC/DRC not required (no KiCad change)

## 2026-09-11 — create pipeline stage: spec-seed

- Change: seed-at86rf233-carrier-spec
- Files: docs/SPEC.md, docs/DECISIONS.md
- Verification: ERC not required
