# Proposal: outputs

> Marker: AUTO (autonomous mode; auto-approved, reviewable after the fact)

## Why

Create the required fabrication, mechanical, visual-review, and ordering package from the existing KiCad design without changing its electrical or RF design.

## What Changes

- Export JLCPCB-profile Gerbers and drill files into `outputs/`.
- Export the board outline as DXF and the assembled board as STEP into `outputs/`.
- Export schematic and PCB SVG renders.
- Generate `outputs/BOM.csv` from `docs/BOM.md` with columns `refdes,MPN,qty`, grouping identical MPNs where appropriate.
- Record the outputs-stage rationale, generated files, and verification in the design documentation.
