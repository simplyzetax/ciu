# Proposal: firmware

> Marker: AUTO (autonomous mode; auto-approved, reviewable after the fact)

## Why

Provide the smallest ESP-IDF firmware scaffold needed to reset the AT86RF233 and verify its identity registers over SPI, while keeping docs/PINOUT.md authoritative for carrier connector nets.

## What Changes

- Create a minimal ESP-IDF project under firmware/.
- Add pins.h derived from docs/PINOUT.md, with connector pin/net provenance and documented XIAO ESP32-C6 GPIO assignments only.
- Add a small AT86RF233 driver exposing reset and register read.
- Add one app_main happy path that resets U1, reads PART_NUM and VERSION_NUM, and validates the expected values.
- Add DEVPLAN.md with build status, including the exact phrase `not compiled here` if ESP-IDF is unavailable.
- Record the firmware-stage rationale and changelog entry without changing schematic, PCB, BOM, or electrical constraints.
