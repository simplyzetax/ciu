# Proposal: layout-draft

> Marker: AUTO (autonomous mode; auto-approved, reviewable after the fact)

## Why

Create the required first-draft physical implementation with constraint-driven placement, complete routing, and explicit documentation of draft limitations.

## What Changes

- Place all footprints at real coordinates within the existing board outline, with J1 on an edge, U1 support parts at their pins, the RF chain and antenna aligned to an edge, and antenna keepouts honored.
- Rotate long footprints onto a board axis where required, accounting for rotated footprint extents.
- Route every electrical net on the two-layer board, including the 50-ohm RF feed as a documented draft geometry, until DRC reports no unconnected items.
- Add or update docs/LAYOUT.md with a `## Draft quality` section stating exactly what is acceptable and what requires human or specialist-tool rework.
- Update design documentation and changelog for every placement, routing, keepout, and intentional protection decision.
