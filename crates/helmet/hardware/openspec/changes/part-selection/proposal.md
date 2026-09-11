# Proposal: part-selection

> Marker: AUTO (autonomous mode; auto-approved, reviewable after the fact)

## Why

Complete Stage 3 with a non-empty, machine-capturable BOM whose Value cells contain only component values, whose rows map one-to-one to refdes, and whose selections preserve the documented power and RF constraints.

## What Changes

- Verify and, only where needed, correct `docs/BOM.md` to use exactly `| Refdes | Value | Footprint | MPN | Rationale |`.
- Keep one row per individual refdes and value-only Value cells.
- Keep every MPN marked UNVERIFIED with datasheet-verifiable selection rationale and leakage/quiescent-current budget impact.
- Confirm installed KiCad symbols and authoritative pins for every IC, connector, module, and specialized schematic part.
- Record the completed stage in design documentation without changing the schematic or PCB.
