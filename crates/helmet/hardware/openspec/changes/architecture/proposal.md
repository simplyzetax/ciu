# Proposal: architecture

> Marker: AUTO (autonomous mode; auto-approved, reviewable after the fact)

## Why

Record the carrier-board subsystem architecture before schematic capture, keeping all electrical, RF, and mechanical constraints explicit.

## What Changes

- Add docs/SUBSYSTEMS.md with a prose block diagram and subsystem sections for power, externally connected MCU host, AT86RF233 connectivity/RF, header/test access, and intentionally absent UI/regulation/battery subsystems.
- State the applicable budget, voltage, layer, size, impedance, and device-family constraints with one-line rationales.
- Append the architecture-stage change to docs/CHANGELOG.md and record any architectural decisions in docs/DECISIONS.md.
