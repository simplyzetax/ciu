# Proposal: devplan

> Marker: AUTO (autonomous mode; auto-approved, reviewable after the fact)

## Why

Stage 8 requires an actionable development and prototype plan that turns the verified carrier design, documented test points, bring-up register checks, and unresolved RF/manufacturing risks into an ordered checklist.

## What Changes

- Create `docs/DEVPLAN.md` with ordered inspection, power, SPI identity, control-signal, and RF bring-up steps.
- Document each available test point and the first voltage/continuity/signal checks to perform.
- List electrical, RF, component-verification, manufacturing, and firmware risks without claiming the draft RF layout is fabrication-ready.
- Add a prototype ordering plan with pre-order gates, a small initial quantity, staged assembly, and acceptance criteria.
- Update `docs/DECISIONS.md` and prepend one run entry to `docs/CHANGELOG.md`.
