# Central intelligence unit

## End goal: ESP32 that is Sena and Cardo Mesh compatible. Replaces Sena completely. Also, built in HUD inside of Goggles.

### Status

The Sena Mesh 3 air protocol has been recovered from firmware by static analysis: frame format, packet subtypes, discovery and admission, routing, timing, audio packing, and the protected-record crypto.

- [Mesh protocol static analysis](reports/sena-30k-mesh-protocol-static-analysis-2026-09-08.md) — current reference, including the ESP32-C6 implementation requirements and the over-the-air capture plan.
- [First-pass ESP32 feasibility](reports/sena-30k-mesh-3-firmware-esp32-feasibility.md) — superseded baseline, kept for the FCC and firmware-distribution evidence.
- `tools/mesh3.py` — clean-room reference implementation; `python3 tools/mesh3.py verify` re-derives every published vector from the local image.
- `firmware/` — downloaded images, extracted components, and `manifest.json` with verified hashes.

Open items are capture-dependent, not static: the lower MAC/FCS boundary, the exact Speex submode, and the predicted Group Mesh network-ID truncation.

### Development

Run `npm install` after cloning to install the Husky Git hooks. Before each
commit, the pre-commit hook runs `cargo check-esp` against every workspace
crate for the ESP32 target.

### Sena and Cardo replacement

Parts:

- [ESP32-C6](https://www.amazon.de/gp/product/B0GZTV9NDV/ref=ox_sc_act_title_1?smid=A2OYQMAUQB5NOO&psc=1)
- [500 mAh Akku](https://www.amazon.de/EEMB-500mAh-562438-JST2-0-Stecker-St%C3%BCck-blau/dp/B09DPPMX14)

### Goggles HUD

Parts:

- [0.96" OLED Display I2C/IIC/TWI 128x64 Pixel für Arduino](https://www.roboter-bausatz.de/p/0.96-oled-display-i2c-iic-twi-128x64-pixel-fuer-arduino)
- [Optical Prism 45° Splitting Ratio 50%:50%](https://www.amazon.de/gp/product/B0F7LHQWTG/ref=ox_sc_act_title_3?smid=A3FMVC8KTJNQWA&th=1)

INFO:

Serial 10 = Goggles
Serial 210 = Helmet
