# Proposal: part-selection

> Marker: AUTO (autonomous mode; auto-approved, reviewable after the fact)

## Why

Create a stage-4-capturable BOM whose rows map one-to-one to schematic refdes while preserving the specified AT86RF233 carrier architecture and power limits.

## What Changes

- Rewrite docs/BOM.md in the fixed Refdes/Value/Footprint/MPN/Rationale format with one row per refdes.
- Keep Value cells limited to component values and move all descriptive prose to Rationale.
- Mark every introduced MPN UNVERIFIED and justify it with datasheet-verifiable electrical, package, RF, and current characteristics.
- Confirm installed KiCad symbols and real pin numbers for every IC, connector, and other active/non-passive symbol.
- Record the selection rationale and append the run to the changelog.
