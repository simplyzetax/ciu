# Tasks

- [ ] Confirm the documented XIAO-to-J1 GPIO mapping and reuse any existing project convention.
- [ ] Create only the ESP-IDF files required to configure, build, reset U1, and read identity registers.
- [ ] Derive pins.h net and connector definitions strictly from docs/PINOUT.md; do not invent undocumented GPIO assignments.
- [ ] Build with the vendor toolchain when available; otherwise write `not compiled here` in DEVPLAN.md.
- [ ] Update docs/DECISIONS.md and docs/CHANGELOG.md with one-line rationales.
- [ ] Run documentation drift checks and finish only after all obligations pass.
