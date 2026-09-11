# Proposal: schematic

> Marker: AUTO (autonomous mode; auto-approved, reviewable after the fact)

## Why

Create the minimal AT86RF233-ZU SPI carrier schematic required by the approved architecture while preserving the 3.3 V single-rail, current, RF, and interface constraints.

## What Changes

- Add schematic.intent.json defining the BOM-matched carrier parts, real symbol pins, nets, unused pins, and subsystem groups.
- Draft the deterministic KiCad schematic from the intent.
- Add PINOUT.md and update design records/changelog to match the drafted connectivity.
