# Sena 30K Mesh 3.0 Firmware and ESP32 Feasibility

## Conclusion

**A regular ESP32 plus antenna cannot behave as a native Sena Mesh 3.0 node.** It lacks the required O-QPSK/802.15.4-class radio PHY. An antenna changes range, not modulation or packet decoding.

**ESP32-C6 or ESP32-H2 is a plausible experimental platform**, because both have a 250-kbit/s IEEE 802.15.4 O-QPSK radio. However, native Sena interoperability remains an RF/protocol reverse-engineering project. The firmware and FCC evidence do not reveal enough to implement it yet.

Building an **independent mesh-like motorcycle intercom** is feasible on ESP32-class hardware. Building one that joins a Sena Mesh 3.0 conversation is not currently demonstrated.

## Firmware source

Sena Device Manager 4.4.19 has this base URL embedded:

```text
https://firmware.sena.com/senabluetoothmanager/
```

It retrieves:

- [Current firmware catalog](https://firmware.sena.com/senabluetoothmanager/Firmware)
- [Previous firmware catalog](https://firmware.sena.com/senabluetoothmanager/Revisions)

The catalog filename is appended directly to the base URL.

### Current 30K downloads

| Hardware | Firmware | Direct download |
|---|---:|---|
| SP46: 30K-01/-01D/-02/-02D | v3.5 | [30K_v3.5-build0.img](https://firmware.sena.com/senabluetoothmanager/30K_v3.5-build0.img) |
| SP113: 30K-03/-03D | v4.5.1 | [30K-v4.5.1-build0.img](https://firmware.sena.com/senabluetoothmanager/30K-v4.5.1-build0.img) |

Do not install one branch on the other hardware revision.

## Local firmware dump

Saved under `firmware/`:

```text
firmware/
├── Firmware
├── Revisions
├── manifest.json
├── 30K_v3.5-build0.img
├── 30K-v4.5.1-build0.img
└── extracted/
    ├── 30K_v3.5-build0/
    │   ├── 30KBT.dfu
    │   ├── vp.bin
    │   └── MESH3.0_v3.0.0.7.ldr
    └── 30K-v4.5.1-build0/
        ├── 30K.img
        ├── vp.bin
        └── Submesh_v0.5b0.bin
```

The accompanying `*.strings.json` files contain extracted printable strings.

Verified SHA-256:

```text
e326ed13e013e69dba9e6a1989a445e94b22a935b18b2317821ba0bda8a77e92
  firmware/30K_v3.5-build0.img

736b07791e3768ec679ea8664b07cd29412c2c8f5d4e5ed1536c4a567125cf76
  firmware/30K-v4.5.1-build0.img

ff2b896a2ecb8a3704d0a2c368c1caae23a76a2fa79d29cd87b14e55d919f3b6
  firmware/extracted/30K_v3.5-build0/MESH3.0_v3.0.0.7.ldr

825e806ee87373a749a66766ac3ef9b1c7601fcc1e32b8dcd2fa1ef09f5ac7cf
  firmware/extracted/30K-v4.5.1-build0/Submesh_v0.5b0.bin
```

Every extracted component's MD5 matches the checksum stored in its parent firmware container. `firmware/manifest.json` records all URLs, sizes, MD5s, and SHA-256 hashes.

## What the firmware reveals

### Original SP46

The v3.5 image contains three independently checksummed payloads:

1. `30KBT.dfu` — Bluetooth firmware.
2. `vp.bin` — voice prompts/resources.
3. `MESH3.0_v3.0.0.7.ldr` — dedicated Mesh 3.0 program.

The Mesh payload is a valid Analog Devices BF70x loader stream:

- 90 checksum-valid boot blocks.
- `Mesh_Intercom_3.0.dxe`
- `Closed Loop Design BF707 HID Library`
- Speex `nb_celp.c`
- Speex DSP `jitter.c`
- `narrowband`
- `wideband (sub-band CELP)`
- `MESHVERSIONCHECK`
- `Mesh_v2.0`
- Winbond `W25Q32BV`

This is strong evidence that the SP46 uses an ADSP-BF707-class processor for mesh/audio processing and includes Speex CELP plus jitter-buffer code.

It does **not** prove:

- which Speex mode is active;
- its bitrate or frame length;
- that all mesh audio uses Speex;
- the radio frame format;
- timing or routing behavior.

### Revised SP113

The v4.5.1 image contains:

- `30K.img`
- `vp.bin`
- `Submesh_v0.5b0.bin`

The submesh payload is much larger and mostly opaque/high-entropy. Its processor and packaging appear different, but the available evidence is insufficient to identify them defensibly.

## Mesh 3.0 behavior

Sena's [30K Mesh 3-era manual](https://firmware.sena.com/senabluetoothmanager/UserGuide_30K_4.4.1_en_250331.pdf) documents product behavior, not a wire protocol.

### Open Mesh

- Six **user-facing conversation channels**.
- Virtually unlimited members per channel.
- No individual headset pairing.
- Dynamic joining and reconnecting.

### Group Mesh

- Private/stored group.
- Up to 24 connected members.
- Existing members explicitly admit new members.
- Membership information survives switching back to Open Mesh.

### Range

Sena advertises:

- Up to 2 km between 30K units in open terrain.
- Up to 8 km when at least six users extend the mesh.

These are product claims, not a specified hop count or routing guarantee.

### Mesh 2 compatibility

Mesh 3 devices must be switched to Mesh 2 in the app to communicate with legacy Mesh 2 peers. Mesh 2 and Mesh 3 are not transparently interoperable simultaneously.

Official releases:

- [SP46 firmware v3.5 added Mesh 3.0](https://www.sena.com/en-us/stories/notice/sena-releases-firmware-updates-for-the-30k-v3-5/)
- [SP113 firmware v4.4 added Mesh 3.0](https://www.sena.com/en-us/stories/notice/sena-releases-firmware-updates-for-the-30k-2/)

## Actual radio characteristics

FCC filings show Bluetooth and mesh are separate radio functions with separate antennas.

### SP46

The [SP46 mesh test report](https://fccid.io/S7A-SP46/Test-Report/Test-Report-1-ZigBee-3574016.pdf) specifies:

- O-QPSK modulation.
- 2410–2475 MHz.
- 14 RF channels.
- Approximately 5 MHz channel spacing.
- 1.433–1.456 MHz measured 6-dB bandwidth.
- Maximum 16.96 dBm conducted power.
- Test mode is named `ZIGBEE`.

### SP113

The [SP113 mesh test report](https://fccid.io/S7A-SP113/Test-Report/Test-Report-DTS-MESH-5800108.pdf) specifies:

- O-QPSK modulation.
- 2410–2475 MHz.
- 14 RF channels.
- 1.44–1.48 MHz measured 6-dB bandwidth.
- 2.42–2.48 MHz 99% occupied bandwidth.
- Maximum 17.22 dBm conducted power.

The 14 RF centers appear to be:

```text
2410, 2415, 2420, ... 2475 MHz
```

Those match IEEE 802.15.4 channels 12–25. The modulation and bandwidth are also consistent with a standard-like 2.4-GHz 802.15.4 waveform.

But this does **not** prove Sena uses Zigbee at the network layer. `ZIGBEE` may only be the certification test mode. The six Open Mesh conversation channels are also not necessarily six physical RF frequencies; they are a logical product setting above the 14-channel radio.

## ESP32 feasibility

### Regular ESP32 / ESP32-S3

No viable native-Sena path:

- Wi-Fi 802.11.
- Bluetooth/BLE.
- No documented 802.15.4 O-QPSK PHY.
- Raw Wi-Fi injection still produces 802.11 frames; it cannot generate arbitrary preambles, spreading codes, chip rates, or O-QPSK waveforms.
- A better antenna cannot change this.

An external O-QPSK transceiver or radio coprocessor would be required.

### ESP32-C6 / ESP32-H2

Potential PHY candidate:

- IEEE 802.15.4-2015.
- O-QPSK.
- 250 kbit/s.
- Raw frame transmission/reception.
- Channels 11–26.
- C6 supports up to +20 dBm.

Relevant sources:

- [ESP32-C6 datasheet](https://documentation.espressif.com/esp32-c6_datasheet_en.pdf)
- [ESP32-H2 datasheet](https://documentation.espressif.com/esp32-h2_datasheet_en.pdf)
- [ESP-IDF raw 802.15.4 API](https://raw.githubusercontent.com/espressif/esp-idf/v5.4.2/components/ieee802154/include/esp_ieee802154.h)

Critical limitation: the C6/H2 radio handles the 802.15.4 PHY and automatically generates/checks FCS. It is not an SDR. If Sena changes the physical preamble, spreading, symbol rate, frame boundary, or CRC, the ESP radio may neither receive nor transmit compatible frames.

## Information still missing

A native implementation needs all of the following:

1. PHY rate, spreading, preamble, SFD, whitening and frequency-selection behavior.
2. MAC headers, IDs, packet types, sizes, CRC and retransmission rules.
3. Beacon and discovery formats.
4. Joining, Group Mesh admission and rejoining state machines.
5. Scheduling/access method—TDMA, contention, coordinator, or something else.
6. Multi-hop forwarding, duplicate suppression and topology repair.
7. Active audio codec, sample rate, mode, bitrate, frame duration and packet packing.
8. Mixing/arbitration rules for multiple speakers.
9. Encryption, authentication, key exchange, nonce and replay behavior.
10. Mesh 2/3 version negotiation.

The firmware dump helps with items 2–9, but static strings alone cannot recover them.

## Recommended next experiment

Use two owned Sena units and an ESP32-C6/H2:

1. Put both Sena units on Mesh 3 Open Mesh.
2. Scan raw 802.15.4 channels 12–25 in promiscuous mode.
3. Generate controlled events:
   - mesh on/off;
   - conversation channel changes;
   - mute/unmute;
   - reach-out;
   - group creation/join;
   - silence versus a fixed audio tone.
4. Record timestamps, channel, frame length, RSSI and payload bytes.
5. Check whether valid 802.15.4 frames are received.

Possible outcomes:

- **Valid frames received:** the C6/H2 may cover the PHY, making a custom MAC implementation plausible.
- **Energy but no valid frames:** Sena probably uses nonstandard framing/CRC or a different O-QPSK rate; move to an IQ SDR.
- **No observable energy:** wrong RF channel/time, hopping, capture setup, or a nonstandard waveform.

For deeper RF work, a 2.4-GHz IQ SDR such as HackRF is a better discovery tool than any ESP32.

## Practical alternative

If the goal is communication with Sena users rather than a native clone, use a supported bridge:

- [MeshPort Blue manual](https://firmware.sena.com/senabluetoothmanager/QuickStartGuide_MeshPort_Blue_1.8.0_en_250516.pdf): HFP Bluetooth headset to Open Mesh, Mesh 2/3 selection, repeater mode.
- [MeshPort Red](https://senaindustrial.com/products/meshport-red): phone/tablet/VoIP to Mesh.
- [MeshON](https://store-us.sena.com/products/meshon): current Mesh 3 bridge supporting compatible wideband HFP devices.

That delegates the proprietary mesh radio/protocol to Sena hardware. It is much lower risk than reproducing the air interface.
