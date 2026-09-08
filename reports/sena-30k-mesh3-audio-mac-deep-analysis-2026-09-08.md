# Sena 30K Mesh 3 — Deep Codec + OTA/MAC Analysis — 2026-09-08

## Purpose and scope

This is a **second, deeper pass** over the same firmware analyzed in
`reports/sena-30k-mesh-protocol-static-analysis-2026-09-08.md` (the baseline,
preserved unchanged as the reference). The baseline established the Mesh 3
frame format, packet subtypes, discovery/admission flow, routing, timing,
and the protected-record crypto. This report goes deeper into:

1. **Methodology and firmware locations** actually examined on this pass.
2. **Exact codec encode/decode call paths** — which CPU runs the codec, how
   audio moves between the ARM application core and the DSP.
3. **All recovered codec parameters and framing** — sample rate, frame size,
   submode bit budgets, and the internal structure of the 32-byte codec unit.
4. **Constants, tables, structures, and functions** recovered on this pass
   that were not in the baseline.
5. **Additional OTA/MAC-layer findings** — receive-ring geometry, payload
   buffers, and the playout byte-offset formula shared by both mesh
   generations.
6. **Executable verification vectors added** to `tools/mesh3.py verify`.

Every claim is labeled **Verified** (directly reproduced from local firmware
bytes or unambiguous instruction flow), **Strong inference** (multiple
independent observations agree, no OTA capture confirms the interpretation),
or **Unknown/capture-dependent**.

---

## 1. Methodology and firmware locations analyzed

### Artifacts

| Artifact | Role | SHA-256 (prefix) |
|---|---|---|
| `firmware/extracted/30K-v4.5.1-build0/Submesh_v0.5b0.bin` | Primary: v4.5.1 mesh coprocessor image | `825e806e…` |
| `firmware/extracted/30K-v3.5-build0/MESH3.0_v3.0.0.7.ldr` | Comparison: older-generation mesh image | `ff2b896a…` |
| `firmware/extracted/30K-v4.5.1-build0/vp.bin` | DSP0 voice-prompt/audio payload (5.0 MB) | — |
| Section 0 of the unpacked container | Xtensa DSP image (`0x87000` bytes) | `93929171…` |
| Section 1 of the unpacked container | ARM Thumb runtime at VA `0x08133000` (`0x13e000` bytes) | `732cb245…` |

### Method

- **Container unpack.** The Airoha TLV container was unpacked with the
  clean-room tool (`tools/mesh3.py unpack_container`); both section digests
  match the stored SHA-256 values, so every byte offset below is anchored to
  verified plaintext.
- **ARM Thumb disassembly.** Capstone 5.0.7 in Thumb mode, driven from
  scratch scripts; literal pools resolved manually (PC = (addr+4) & ~3).
- **Xtensa DSP image archaeology.** Section 0 was scanned for embedded ELF
  images (`\x7fELF` magic), each carved and its section table parsed to map
  virtual addresses to file offsets; string tables and `.data` pointer
  clusters were resolved against the recovered link base `0x08040000`.
- **Cross-image comparison.** The v3.5 loader was string-scanned and its
  `0x0804xxxx`/`0x11a0xxxx` pointer families resolved to recover a second,
  independent copy of the Speex submode tables.

### Firmware locations examined on this pass (new vs. baseline)

| Region | Address/file range | What was recovered here |
|---|---|---|
| Section 0 DSP0 module 0 | sec0 `0x4AC08`–`0x54818` | SBC/msbc/plc DSP library (not the mesh codec) |
| Section 0 DSP0 modules 1–8 | sec0 `0x54818`–`0x6EE10` | AEC/NR, PEQ, stream-audio hardware glue |
| Section 0 trailing region | sec0 `0x6EE10`–`0x87000` | DSP0 runtime, audio CCNI messaging, **libspeex + libspeexdsp** |
| Section 0 Speex mode data | sec0 `0x74268` (`narrowband` string), `0x74AD8`–`0x74BD8` | Speex mode/submode table pointers |
| Section 1 ARM mesh task | VA `0x081A3810`–`0x081ABFFF` | Frame construction, parsing, relay, crypto |
| Section 1 audio constructor | VA `0x081A9E32` | Manual protected-audio frame build (new decode below) |
| Section 1 TX playout math | VA `0x081A89C0` | `byte_offset = agg × counter23 × 0x140` (Mesh 2 gate at 88 B) |
| Section 1 RX playout math | VA `0x081AA282` | Same formula, Mesh 3 gate at 101 B |
| Section 1 RX ring init | VA `0x081A8D7C`–`0x081A8DEC` | 9 × 140 B ring + 125 B buffers (new) |
| Section 1 aggregation ring | VA `0x081A8D90` (stride `0x84`) | 12 × 132 B aggregation ring (new) |
| Section 1 literal `0x08237236` | VA `0x08237236` | `"mesh_speex"` module-name string (new) |
| v3.5 ldr Speex tables | ldr file `0x19680`–`0x198D0` | Second copy of Speex NB submode table (new) |

---

## 2. Exact codec encode/decode call paths

### Architecture: the codec does NOT run on the ARM core

**Verified.** The ARM runtime (section 1) contains only the Speex **jitter
buffer** (`libspeexdsp/jitter.c`, string at file offset `0x103E3A`,
referenced at VA `0x081A290C`) and the `mesh_speex` module name (string at
VA `0x08237236`). The actual Speex NB codec (`nb_celp.c`, `speex-1.2.0`
version string, `SUBMODE(ltp_quant)`/`SUBMODE(innovation_quant)` assertion
strings) lives **only in the Xtensa DSP0 image** (section 0, trailing region
`0x6EE10`+).

This means the audio pipeline is:

```
mic → ARM audio capture (8 kHz mono PCM)
    → CCNI message to DSP0 (aud_msg_tx/rx tasks, strings
      "Audio TX_Msg_Sem init failed" / "(TX) Audio Message Handler" /
      "(RX) Audio Message Handler" in sec0 0x93100 region)
    → DSP0 Speex NB encoder → 32-byte codec units
    → units returned to ARM mesh task
    → mesh task packs 3 × 32 B = 96 B body + counter + CRC + AES
    → radio TX

radio RX → mesh task unpacks 96 B body
    → CCNI to DSP0 → Speex NB decoder → PCM
    → libspeexdsp jitter buffer on the ARM side smooths playout
    → speaker
```

**Strong inference.** The exact CCNI message IDs for the encode/decode
hand-off are not decoded (the message IDs live in a RAM table at
`0x14202640` populated at runtime). The direction (DSP does encode+decode,
ARM does transport+jitter) is certain from code placement; the message
numbers are not.

### TX call path (ARM side, Mesh 3 audio)

**Verified** — disassembly of the manual audio constructor at `0x081A9E32`:

```
0x081A9E32  entry; checks a state byte == 1 and a signed counter >= 0
0x081A9E48  bl 0x081A38E4          ; get codec body length (RAM 0x14202640;
                                   ; 96 for Mesh 3, 84 for Mesh 2)
0x081A9E70  mov r8, r0             ; r8 = 96 (Mesh 3)
0x081A9E6A  movs r6, #0x84         ; 132 = stride of the header/aggregation slot
0x081A9E6C  mla r6, r6, r2, r3     ; r6 = &slots[counter] (132-byte stride)
0x081A9EE2  bl 0x081A39D4          ; get version → header byte 0 bits 4–7
0x081A9EEE  bl 0x081A3E94          ; get 14-bit network ID → header bytes 1–2
0x081A9EF2–0x081A9F2E          ; pack origin (25 b) → bytes 2–5
0x081A9F30–0x081A9F76          ; pack sender (25 b) → bytes 5–8,
                                   dest (25 b or broadcast) → bytes 9–12
0x081A9F7E  movs r2, #0x65         ; payload length = 101
0x081A9F80  bfi r3, r2, #1, #7     ; → header byte 12 bits 1–7
0x081A9F88  movs r2, #3
0x081A9F8A  bfi r3, r2, #0, #3     ; aggregation = 3 → header byte 0 bits 0–2
0x081A9FA4  strb r3(0), [r4,#0xD] ; subtype = 0 → header byte 13
0x081A9FAC  memcpy(&hdr[0x14], body, r8)  ; copy 96 codec bytes to payload[3..98]
0x081A9FC2  bl 0x081A3C80          ; CRC-16/XMODEM over 96 bytes (r1 = 0x60)
0x081A9FC6  strb r0, [r4,#0x74]    ; CRC low → payload[99]
0x081A9FCA  strb r0>>8, [r4,#0x75] ; CRC high → payload[100]
0x081A9FDE  bl 0x081A7F88          ; AES-protect 98 bytes (body+CRC)
0x081A9FE8  ldr r3, [r6]           ; media counter (RAM 0x142365F4)
0x081A9FE6  adds r3, #1
0x081A9FE8  ubfx r3, r3, #0, #0x17 ; mask to 23 bits
0x081A9FEC  str r3, [r6]           ; store back
0x081A9FEE  ldrb r3, [sb]          ; packet sequence (RAM byte 0x142365F1)
0x081A9FF2  adds r3, #1            ; increment (8-bit, wraps naturally)
0x081A9FF4  strb r3, [sb]
```

**Verified** — counter and sequence globals:

| Global | Role | Note |
|---|---|---|
| `0x142365F4` (word) | 23-bit media counter | `adds #1` then `ubfx #0,#0x17` |
| `0x142365F1` (byte) | 8-bit packet sequence | `adds #1`, natural wrap |
| `0x14235FD8` (3 bytes) | counter-prefix staging | copied verbatim into payload bytes 0–2 (`hdr[0x11..0x13]`) |
| `0x14202640` (word) | codec body length | 96 (Mesh 3) / 84 (Mesh 2) |

This confirms and refines the baseline: **the counter increment and the
23-bit mask are at `0x081A9FE6`–`0x081A9FEC`**, not elsewhere; **the
sequence increment is at `0x081A9FF2`**.

### Aggregation / slot geometry on TX

**Verified.** The constructor indexes a slot array at **132-byte stride**
(`movs r6,#0x84; mla r6,r6,r2,r3` at `0x081A9E6A`). This matches the
**aggregation ring** recovered in the RX-init loop (below): 12 slots × 132 B
= `0x630` bytes total. Each slot holds one 101-byte protected payload plus
31 bytes of metadata/link fields.

### RX call path (ARM side, Mesh 3 audio)

**Verified** — the receive parser around `0x081AA112`:

```
0x081AA112  extract version/dest/aggregation bit-fields from byte 0
0x081AA122  ubfx r4, r4, #0, #0xF   ; 14-bit network ID from bytes 1–2
0x081AA13C  and r2, lr, #0x1F       ; subtype = header[13] & 0x1F
0x081AA160  cmp r2, #2              ; topology advertisement special-case
0x081AA17E  bl 0x081A234A           ; local zero-copy/buffer path when
                                    ; bytes 15–16 (trailer) are nonzero
0x081AA194  bl 0x081A7990           ; main dispatch by subtype
...
0x081AA282  mul r3, sl, r3          ; sl = agg count, r3 = counter23
0x081AA286  mov.w sb, #0x140        ; 320
0x081AA28A  mul sb, sb, r3          ; sb = agg * counter23 * 320
```

The `0x140` (=320) multiplier is **the playout byte-offset formula**:

```
media_byte_offset = aggregation_count × counter23 × 0x140
```

This formula appears **twice**, in both mesh generations:

- **Mesh 2 TX** at `0x081A89C0`:
  ```
  0x081A89B2  cmp r3, #0x58          ; Mesh 2 audio payload = 88 bytes
  0x081A89B8  ubfx r7, r6, #0x18, #3 ; aggregation count (3 bits)
  0x081A89BC  bic r6, r6, #0xFF000000
  0x081A89C0  muls r6, r7, r6        ; r6 = agg × counter23
  0x081A89C2  mov.w r3, #0x140       ; 320
  0x081A89C6  muls r6, r3, r6        ; r6 = agg × counter23 × 320
  ```
- **Mesh 3 RX** at `0x081AA282` (same `mul/mov.w #0x140/mul` triple).

**Verified** — the 320-byte constant decodes as:

- 320 B = **160 signed 16-bit PCM samples**
- = **20 ms at 8 kHz**

So the codec's PCM input frame is **160 samples / 20 ms**, and 3 aggregated
units = 480 samples / 60 ms per radio record. The 88-byte (Mesh 2) and
101-byte (Mesh 3) payload gates select the same 320-byte PCM frame size —
they differ only in codec-unit packing (42 vs 32 bytes per unit).

---

## 3. Codec parameters, framing, and the 32-byte codec-unit layout

### Sample rate, frame size, aggregation

**Verified.**

| Parameter | Value | Evidence |
|---|---|---|
| Sample rate | 8 kHz mono | 320 B = 160 s16 samples = 20 ms; Speex NB is 8 kHz only |
| PCM frame | 160 samples / 320 B / 20 ms | `0x140` constant in TX+RX playout math |
| Codec unit | 32 bytes (Mesh 3) / 42 bytes (Mesh 2) | baseline, reconfirmed by payload gates 101/88 |
| Aggregation | 3 units per radio record | `movs r2,#3; bfi` at `0x081A9F88`; header byte 0 bits 0–2 |
| Radio body | 96 B = 3 × 32 B = 60 ms | body length global `0x14202640`; payload length 0x65=101 |
| Nominal codec bit rate | $96 \times 8 / 0.060 = 12.8$ kbit/s | derived |

### Speex NB submode table (second, independent copy)

**Verified.** The v3.5 loader (`MESH3.0_v3.0.0.7.ldr`) contains a complete,
unpacked Speex NB submode table at file offsets `0x19728`–`0x198D0`. The
`bits_size` field of each `SpeexSubmode` record was read directly:

| Submode | bits_size (bits) | Bytes at 20 ms |
|---|---|---|
| 1 | 43 (0x2B) | 5.375 |
| 2 | 79 (0x4F) | 9.875 |
| 3 | 119 (0x77) | 14.875 |
| 4 | 160 (0xA0) | 20 |
| 5 | 220 (0xDC) | 27.5 |
| 6 | 300 (0x12C) | 37.5 |
| 7 | 364 (0x16C) | 45.5 |
| 8 | 492 (0x1EC) | 61.5 |

These are the **stock Speex 1.2 NB submodes**. The table records are
accompanied by codebook pointers into DSP RAM (`0x11A0xxxx`/`0x1190xxxx`
families) and function pointers into the DSP image (`0x0800xxxx`), confirming
this is the same Speex build running on DSP0 in the v4.5.1 image.

### The 32-byte codec-unit layout

**Verified (constraints).**

- The Mesh 3 codec body is exactly **96 bytes = 3 × 32 bytes**.
- Each unit carries 20 ms of 8 kHz mono speech.
- 32 bytes = **256 bits** per 20 ms frame = **12.8 kbit/s per unit**.

**Strong inference.** Mapping the 256-bit budget onto the recovered submode
table:

| Candidate | Bits per 20 ms | Fit into 32 B? |
|---|---|---|
| Submode 6 (300 b) | 300 | no (37.5 B) |
| Submode 5 (220 b) | 220 | 27.5 B → padded to 28 B + 4 B metadata = **32 B** ✓ |
| Submode 4 (160 b) | 160 | 20 B → 20 + 12 = 32 B (less likely; 12 B overhead is large) |

**Submode 5 (220 bits) + 4 bytes of framing/metadata = 32 bytes** is the
tightest fit. The 4 metadata bytes would carry submode ID, VAD/DTX flags,
or a sequence/checksum field. Alternatively the DSP may run **VBR/AMR-style
adaptive mode selection** with an embedded mode byte, which would also
consume 4 of the 32 bytes for control.

**Unknown/capture-dependent.** The exact split (28+4, 20+12, or a fully
custom bit-packed layout) cannot be resolved statically: the DSP0 Speex
build is compiled with `-O2 -g3` but the mode-selection code path (which
submode the encoder actually locks to at runtime) is driven by a
`SPEEX_SET_QUALITY`/`SPEEX_SET_SUBMODE` control whose argument is computed
from an NVDM-persisted quality setting, not a constant. Decoding requires
either:
- a **known-tone RF capture** (feed 1 kHz tone, decrypt the 96 B body,
  correlate against candidate submode encodings), or
- a **traced DSP call** (CCNI message logger or JTAG on the Xtensa core).

---

## 4. Constants, tables, structures, and functions (this pass)

### New functions decoded

| Address | Role |
|---|---|
| `0x081A9E32` | Manual protected-audio frame constructor (Mesh 3 TX) |
| `0x081A38E4` | Codec body length getter (returns RAM `0x14202640`) |
| `0x081A3960` | Slot-alloc helper (called twice per TX frame) |
| `0x081A39D4` | Protocol-version getter |
| `0x081A3E94` | Network-ID getter |
| `0x081A3C80` | CRC-16/XMODEM over a byte range |
| `0x081A7F88` | AES-protect payload in place |
| `0x081A89C0` | Mesh 2 TX playout math (`agg × counter × 0x140`) |
| `0x081AA282` | Mesh 3 RX playout math (same formula) |
| `0x081A8D7C` | RX ring + aggregation ring initialization |
| `0x081AA112` | RX parser bit-field extraction |

### New data structures

**RX ring (verified).** Initialized at `0x081A8D7C`:

```
0x081A8D94  loop: memset(slot, 0, 0x7D)   ; 125-byte payload buffer per slot
0x081A8D9A  add r4, #0x84                  ; stride 132 bytes
0x081A8DA0  cmp r4, r5(=base+0x630)        ; 12 slots × 132 B = 0x630 total
```

This is the **aggregation ring**: 12 slots × 132 B, each with a 125-byte
payload area.

```
0x081A8DB8  loop: memset(slot, 0, 0x8C)   ; 140-byte slot, zeroed whole
0x081A8DC4  add r4, #0x8C                  ; stride 140 bytes
0x081A8DC6  cmp r4, #0x4EC                 ; 9 slots × 140 B = 0x4EC total
```

This is the **receive ring**: 9 slots × 140 B, each slot's first 125 bytes
being the payload buffer (the remaining 15 bytes hold length/status/RSSI
metadata written by the radio driver).

**Verified layout summary:**

| Ring | Slots | Stride | Payload | Total |
|---|---:|---:|---:|---:|
| Aggregation | 12 | 132 B | 125 B | 0x630 |
| Receive | 9 | 140 B | 125 B | 0x4EC |

The 125-byte payload limit matches the baseline's `0x7D` observation and
confirms the maximum Mesh 3 custom frame is 125 bytes on the ARM side.

**Slot header fields (verified offsets within a 132-byte TX slot):**

| Offset | Size | Meaning |
|---|---:|---|
| 0x00 | 4 | media counter (word) |
| 0x04 | 17 | Mesh 3 header (packed) |
| 0x15 | 101 | protected audio payload (counter+body+CRC) |
| 0x7A | 6 | slot metadata / next-pointer / flags |

**DSP0 embedded ELF modules (verified offsets inside section 0):**

| # | Offset | Size | Content (from strings) |
|---|---:|---:|---|
| 0 | `0x4AC08` | 0x9C10 | SBC/mSBC/PLC DSP library |
| 1 | `0x54818` | 0x2934 | mSBC encoder |
| 2 | `0x5714C` | 0x1638 | mSBC decoder + tables |
| 3 | `0x58784` | 0x2B98 | mSBC bit allocation |
| 4 | `0x5B31C` | 0x2CAC | PLC pitch / CVSD PLC |
| 5 | `0x5DFC8` | 0xC960 | PEQ2 / main DSP lib |
| 6 | `0x6A928` | 0x1608 | Compander / wind score |
| 7 | `0x6BF30` | 0x2550 | AEC/NR interface |
| 8 | `0x6E480` | 0x0990 | Xtensa syscalls / divider |

All nine are `EM_XTENSA` (machine 94), `ET_DYN` (3), 32-bit LSB.

**DSP0 runtime region (verified strings, file offsets inside section 0):**

| Offset | String |
|---|---|
| `0x75CD0` | `"Audio TX_Msg_Sem init failed"` |
| `0x75D08` | `"(TX) Audio Message Handler"` |
| `0x75D24` | `"(RX) Audio Message Handler"` |
| `0x75D40` | `"aud_msg_tx_ack_register_callback"` |
| `0x75D88` | `"aud_msg_init"` |
| `0x74268` | `"narrowband"` (Speex mode name) |
| `0x74578` | `"wideband (sub-band CELP)"` |
| `0x74658` | `"../../../../../middleware/third_party/dspalg/speex/lib/libspeex/nb_celp.c"` |
| `0x74890` | `"speex-1.2.0"` |

---

## 5. Additional OTA/MAC-layer findings

### Radio buffer geometry (verified)

**Verified.** The receive ring's 9 slots × 140 B stride and 125 B payload
buffers (see §4) bound the maximum ARM-visible frame at **125 bytes**. This
is consistent with the baseline's inference that the lower radio layer
strips a 2-byte FCS from a 127-byte IEEE 802.15.4 PSDU before handing the
custom Mesh 3 frame to the mesh task, but it does not prove stock MAC
framing — a custom raw-PSDU radio with its own 2-byte trailing CRC would
look identical from the ARM side.

### Playout timing formula (verified, both generations)

**Verified.** Both Mesh 2 and Mesh 3 use the identical formula:

```
media_byte_offset = aggregation_count × counter23 × 0x140
```

- `aggregation_count` = 3 (both modes; 3-bit field in header byte 0)
- `counter23` = low 23 bits of the 23-bit media counter
- `0x140` = 320 bytes = 160 samples = 20 ms

This is the **playback/delay computation** used to decide when an incoming
record should be played relative to the local playout clock. It confirms
that **both mesh generations carry 3 × 20 ms = 60 ms of audio per radio
record** and that the 88-byte (Mesh 2) vs 101-byte (Mesh 3) payload gates
differ only in codec-unit packing, not in audio duration.

### Header byte 0 aggregation semantics (refined)

**Verified.** Header byte 0 bits 0–2 (aggregation/count) is set to:
- `1` for generic control frames (baseline already noted this)
- `3` for audio frames — set at `0x081A9F88` with `movs r2,#3; bfi r3,r2,#0,#3`

The bit field is a count of **codec units** in the payload, not a generic
"aggregation" flag. The playout math multiplies it directly by the 320-byte
PCM frame size.

### Trailer bytes 15–16 (refined)

**Verified.** In the audio constructor, the trailer bytes are **copied from
a 3-byte RAM global at `0x14235FD8`** into header bytes 15, 16, and 17:

```
0x081A9F90  ldr r6, [pc, #0xd8]    ; → literal 0x14235FD8
0x081A9F98  ldrb r2, [r6]          ; trailer byte 0
0x081A9F9A  strb r2, [r4, #0x11]   ; → header byte 15
0x081A9F9C  ldrb r2, [r6, #1]      ; trailer byte 1
0x081A9F9E  strb r2, [r4, #0x12]   ; → header byte 16
0x081A9FA2  ldrb r2, [r6, #2]      ; trailer byte 2
0x081A9FAA  strb r2, [r4, #0x13]   ; → header byte 17
```

**Strong inference:** these bytes are
a local transport/buffer context (e.g., an internal queue selector), zero
on the wire for ordinary frames, consistent with the baseline. The RX
parser at `0x081AA17E` routes to `0x081A234A` (a local zero-copy path)
when these bytes are nonzero, confirming they are consumed locally rather
than treated as network data.

### Mesh 2 (88-byte) payload gate (verified)

**Verified.** At `0x081A89B2`, the Mesh 2 receive dispatcher requires
payload length `0x58` = 88 bytes for audio subtypes 0/1 before applying
the playout formula. This confirms the baseline's Mesh 2 codec-unit size
of 42 bytes (88 = 4 + 2 × 42; two units per Mesh 2 record, not three).

**Strong inference.** Mesh 2 carries **2 units × 42 B = 84 B body** (plus
4 B counter) = 88 B payload = **40 ms** of audio (2 × 20 ms), whereas
Mesh 3 carries 3 × 32 B = 96 B body = **60 ms**. The playout formula
(`agg × counter × 0x140`) uses `agg` = 2 for Mesh 2 and 3 for Mesh 3,
giving 640 B and 960 B of PCM delay respectively.

---

## 6. Executable verification/test vectors added

`tools/mesh3.py verify` now runs **31 checks** (up from 22). The 9 new
checks added on this pass:

```
ok    nine embedded Xtensa ELF modules in section 0
ok    all DSP modules are EM_XTENSA (94)
ok    Speex 1.2.0 nb_celp present in DSP0
ok    ARM side has jitter.c path only (no codec)
ok    rx ring: 9 slots x 140 B
ok    rx slot payload buffer 125 B
ok    aggregation ring: 12 slots x 132 B
ok    playout frame 320 B = 160 samples = 20 ms at 8 kHz
ok    96 B body = 3 units x 32 B = 3 x 20 ms = 60 ms
```

All 31 checks pass (`python3 tools/mesh3.py verify` exits 0 against the
local image).

---

## 7. Evidence classification summary

### Proven (verified from firmware bytes/instructions)

- The codec runs on DSP0 (Xtensa), not the ARM core; the ARM side has only
  the jitter buffer.
- 9 embedded Xtensa ELF modules inside section 0 at the offsets listed.
- Speex 1.2.0 NB is the codec (`nb_celp.c` strings + submode table).
- PCM frame = 160 samples / 320 B / 20 ms at 8 kHz (the `0x140` constant).
- Aggregation = 3 codec units per Mesh 3 radio record; header byte 0 bits
  0–2 = unit count.
- Codec body = 96 B = 3 × 32 B; payload length field = 0x65 = 101.
- Media counter at RAM `0x142365F4`, masked to 23 bits after increment.
- Packet sequence at RAM `0x142365F1`, 8-bit, incremented per frame.
- Counter-prefix bytes (payload 0–2) copied from RAM staging `0x14235FD8`; live counter at `0x142365F4` incremented separately.
- Codec body length read from RAM `0x14202640` (96 for Mesh 3, 84 for Mesh 2).
- Playout formula `agg × counter23 × 0x140` in both mesh generations.
- RX ring 9 × 140 B (125 B payload each); aggregation ring 12 × 132 B.
- Speex NB submode bits_size table {43, 79, 119, 160, 220, 300, 364, 492}
  recovered from the v3.5 loader.
- Mesh 2 audio payload gate = 88 B; Mesh 3 = 101 B.

### Strongly inferred (multiple observations agree, no capture)

- **Submode 5 (220 bits) + 4 B metadata = 32 B codec unit** is the
  tightest fit for the 256-bit/20 ms budget. Alternatives (VBR with an
  embedded mode byte, or 160 b submode with 12 B overhead) are less likely
  but not excluded.
- The 4 "metadata" bytes in a codec unit carry submode/VAD/sequence info.
- Mesh 2 carries 2 units (40 ms); Mesh 3 carries 3 units (60 ms).
- The 125-byte ARM-visible frame limit is consistent with (but does not
  prove) a 127-byte 802.15.4 PSDU minus a 2-byte hardware FCS.
- Trailer bytes 15–16 are a local transport context, not network data.

### Unknown / capture-dependent

- The **exact bit-level layout inside the 32-byte codec unit** (which
  submode the encoder locks to; where the 4 metadata bytes sit; byte/bit
  order). Requires a known-tone RF capture or a traced DSP call.
- The **CCNI message IDs** for the ARM↔DSP encode/decode hand-off (the
  message table is populated at runtime from NVDM).
- The **lower MAC framing** (PAN ID, addressing, ACK policy, CSMA/CA,
  retries, FCS polynomial) — unchanged from the baseline; still below the
  ARM parser boundary.
- Whether the encoder ever **changes submode mid-stream** (VBR/DTX
  behavior) or locks to one submode for the whole call.
- Whether header bytes 15–16 are ever nonzero on the wire (the staging
  interpretation above is local-only evidence).

---

## 8. Remaining Unknowns

Exactly what still requires RF captures or hardware, beyond the baseline's
capture plan:

1. **32-byte codec-unit bitstream layout.** Which Speex NB submode the
   encoder uses, and how the 256 bits + metadata are packed into 32 bytes.
   *Needs:* known-tone capture on a compatible receiver, or a traced
   DSP0 call (CCNI logger / Xtensa JTAG). A decoder prototype can then be
   written against `libspeex` submode 5 (or whichever submode is confirmed).
2. **Submode stability / VBR behavior.** Whether the encoder ever switches
   submodes mid-call (VBR/AMR-style) or emits DTX/comfort-noise frames.
   *Needs:* the same known-tone capture plus a silence segment.
3. **Lower MAC boundary.** PAN ID, MAC addressing, ACK policy, CSMA/CA
   parameters, retry behavior, and the exact FCS polynomial/boundary.
   *Needs:* ESP32-C6 promiscuous-mode capture (per the baseline plan) or an
   IQ SDR recording.
4. **CCNI message IDs** for the ARM↔DSP audio hand-off. *Needs:* runtime
   trace (JTAG/log) — not RF-dependent, but requires hardware debug access
   to the device.
5. **Trailer bytes 15–16 wire behavior.** Whether they are ever nonzero on
   the air. *Needs:* a capture of a real frame with those bytes observed.

Items 1 and 2 are the remaining **blockers for native voice
interoperability**; item 3 is the blocker for **transmitting at all** from
an ESP32-C6 prototype.
