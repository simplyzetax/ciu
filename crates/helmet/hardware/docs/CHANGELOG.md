# Design changelog

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
