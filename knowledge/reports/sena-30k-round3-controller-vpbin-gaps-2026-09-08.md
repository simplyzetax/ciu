# Sena 30K Mesh — Round-3 Static Analysis (Controller Images, vp.bin, Protocol Gaps) — 2026-09-08

## Purpose and scope

Third static pass, delegated to four parallel analysis agents (DspSpeex,
ControllerMac, VpBin, MeshGaps) targeting the round-2 open items:

1. **Controller images** (`30K.img`, `30KBT.dfu`) — does the lower MAC live
   there?
2. **`vp.bin`** — unanalyzed 5 MB DSP0 audio payload; anything new?
3. **DSP0 Speex** — runtime submode selection and codec-unit layout.
4. **Protocol gaps** — Mesh 2 codec unit, probe-survey formula, topology
   threshold semantics, jitter/relay constants, subtype-11 handler.

Every load-bearing claim from the agents was **independently re-verified**
byte-for-byte by the coordinator before inclusion (see §6). One claim
failed verification and was corrected; two round-2 statements were
corrected. Claims are labeled **Verified** (raw bytes reproduced by both
agent and coordinator), **Strong inference** (multiple observations, no
OTA capture), **Unknown**.

Baseline references (unchanged):

- `reports/sena-30k-mesh-protocol-static-analysis-2026-09-08.md` (round 1)
- `reports/sena-30k-mesh3-audio-mac-deep-analysis-2026-09-08.md` (round 2)

---

## 1. Firmware inventory — what each image actually is

**Verified.** Complete identification of every file in `firmware/`:

| File | Format | Role | Mesh-relevant content |
|---|---|---|---|
| `Submesh_v0.5b0.bin` | Airoha TLV + LZMA | Cortex-M mesh runtime + Xtensa DSP0 | protocol, codec, crypto (rounds 1–2) |
| `MESH3.0_v3.0.0.7.ldr` | ADI LDR (`01 50 e8 ad`) | **Blackfin BF707** mesh-intercom DSP (v3.5 gen) | Speex NB submode tables, RF channel tables, CCITT-FALSE FCS table |
| `30K.img` (v4.5.1) | **CSR-dfu2** (`43 53 52 2d 64 66 75 32`) | Qualcomm/CSR Kalimba BT controller (BR/EDR + BLE), product `30K#3211` | none |
| `30KBT.dfu` (v3.5) | CSR-dfu2 | Same role, older build, product `30K#3110`; only 5.5 % byte-identical to `30K.img` | none |
| `vp.bin` (both gens) | Sena `"File"` container | Voice-prompt **audio assets** only (see §4) | none |

**Verified — the decisive negative result:** neither controller image
contains any 802.15.4/MAC material. Zero hits for
`PAN|CSMA|CCA|FCS|ACK|802.15|IEEE|O-QPSK|ZigBee|PSDU` across ASCII and
UTF-16LE string inventories (435 strings in `30K.img`, 604 in
`30KBT.dfu`); no ELF magic anywhere in either file; no 802.15.4 FCF
construction (`0x8841`/`0x8843` hits are Kalimba opcodes, confirmed invalid
as Thumb).

**Consequence:** the round-2 hypothesis "the lower MAC might live in the
controller images" is **dead**. The lower MAC is not in any of the five
analyzed files.

---

## 2. The radio boundary — new structural picture

**Verified.** `MESH3.0_v3.0.0.7.ldr` is an **ADI Blackfin BF707** image
(header `01 50 e8 ad`, DXE name `Mesh_Intercom_3.0.dxe` at file `0x3C`,
string `Closed Loop Design BF707 HID Library` at `0x2747A`, Winbond
`W25Q32BV` SPI flash driver at `0x1A5B4`). It contains:

- The Mesh 2 channel table at file `0x1A710`
  (`13 0d 0e 0f 10 11 16 17 18 15` = 19,13,14,15,16,17,22,23,24,21),
  byte-identical to the Submesh table at VA `0x0823700C`.
- The Mesh 3 channel table at file `0x1A71C`
  (`10 14 0e 16 0c 18 12` = 16,20,14,22,12,24,18), byte-identical to the
  Submesh table at VA `0x08236FC9`.
- A **256-entry CRC-16/CCITT-FALSE lookup table** (poly `0x1021`) at file
  `0x1A728` (Blackfin VA `0x11802A88`), byte-verified as the exact
  byte-indexed table. This is the only FCS-type constant found in any
  mesh-side image. It matches the network CRC (CRC-16/XMODEM shares
  poly `0x1021`, init 0) already verified in rounds 1–2 — the Blackfin
  DSP computes the same CRC family.

**Strong inference (revised architecture):**

```
ARM Cortex-M (Submesh sec1)        Blackfin BF707 (MESH3.ldr)
  mesh protocol, crypto,             audio DSP + RF interface glue
  framing, channel tables,           Speex NB codec (v3.5 gen)
  CCNI bridge  <------CCNI/SPI---->  channel tables + CRC table
        |
        v  (RF control via m2/m3_rf_param)
  external 802.15.4 radio / sub-MCU  <-- PAN ID, ACK, CSMA/CA live HERE
```

The RF parameters (`m2_rf_param`, `m3_rf_param`, strings at VA
`0x08236FD0`/`0x08236FDC`, xrefs verified in sec1 code at `0x0814B6B8`
family) configure a radio that is **outside all five analyzed files**.
PAN ID, ACK policy, and CSMA/CA parameters were not found in any image.

**Still capture-dependent:** the entire 802.15.4-vs-custom-PSDU question.
The ESP32-C6 capture plan (round 1) remains the only path to it.

---

## 3. Codec unit layout — advanced but not closed

### sec0 Speex submode table (Verified, byte-exact)

The round-2 inference is now **confirmed by a second copy inside the
v4.5.1 image itself**: complete stock Speex 1.2 NB submode records at
sec0 `0x74018`–`0x741A8`, `bits_size` at fixed stride:

| sec0 offset | bits_size |
|---|---:|
| `0x7401C` | 43 |
| `0x74054` | 79 |
| `0x7408C` | 119 |
| `0x740C4` | 160 |
| `0x740FC` | 220 |
| `0x74134` | 300 |
| `0x7416C` | 364 |
| `0x741A4` | 492 |

Plus a second copy in the v3.5 loader at file `0x19728`–`0x198D0`
(round 2) — three independent copies of the same stock table.

**Verified.** DSP0 virtual-address mapping pinned:
`VA = 0x08040000 + (file_offset − 0x71968)` for the `0x0804xxxx` rodata
family (brute-forced: the `narrowband` string at `0x74268` maps to VA
`0x08042900`, referenced by 10 pointer sites, no competing base scores
above noise). Per-submode data pointers (`0x08086F4C`–`0x08087xxx`
family) are **not file-backed in sec0** — they point into a DSP data-RAM
segment filled at load time.

### Runtime submode selection — still UNKNOWN, now precisely bounded

**Unknown, with the blocker identified.** The submode cannot be resolved
statically this round because no Xtensa disassembler was available:
capstone 5.0.7 has no `CS_ARCH_XTENSA` in this build (verified:
`dir(capstone)` lists `CS_ARCH_XCORE`, not XTENSA), and llvm@21 has no
Xtensa target. The ctl handler cannot be pointer-scanned: the
`Unknown nb_ctl request: ` string at sec0 `0x745C0` has **zero** pointer
words resolving to it under the pinned base — the reference must be a
PC-relative/computed address, requiring real instruction-stream
disassembly. The round-2 "selection bottoms out in NVDM" hypothesis is
**downgraded to open** — neither confirmed nor refuted.

The 28+4 codec-unit model (220-bit submode + 4 B metadata) remains the
tightest fit but is unchanged in status: strong inference, capture or
Xtensa disassembly required.

### Counter-prefix staging: writer/reader pair closed (Verified)

**Verified — new.** The 3-byte staging global `0x14235FD8` (round 2 found
the reader; the writer was unknown) is written by the **Mesh 2 TX frame
builder at VA `0x081A7DC0`**:

```
0x081A7ED6  ldr r3, [pc, #0x28]     ; pool 0x081A7F00 = 0x14235FD8
0x081A7ED8  ldr r2, [r3]
0x081A7EDA  adds r2, #1
0x081A7EDC  bic r2, r2, #0xFF000000 ; keep low 24 bits
0x081A7EE0  str r2, [r3]
0x081A7EE2  ldrb r3, [r8]           ; r8 -> 0x142365F1 (packet sequence)
0x081A7EE6  adds r3, #1
0x081A7EE8  strb r3, [r8]
```

So one ARM function increments the 24-bit staging counter and the packet
sequence together, and the Mesh 3 audio ctor (`0x081A9F90`–`0x081A9FA4`)
copies the 3 staging bytes into the frame. Single-writer/single-reader
confirmed by exhaustive literal scan (exactly 4 loader sites for
`0x14235FD8`, only this function stores).

**Consequence:** the 3-byte counter prefix is an **ARM-side transport
counter shared by both mesh generations** — incremented on the Mesh 2
builder and consumed by the Mesh 3 constructor. It is a mesh-family
field, not DSP-generated.

### Media counter semantics refined (Verified)

**Verified — new.** The media counter `0x142365F4` is not a free-running
23-bit counter: the updater at VA `0x081A8458` (called from the audio
ctor path at `0x081A9E48`) implements **modulo-60000 sample arithmetic**
— constants `movw r2, #0xEA60` (60000) at `0x081A84C8` (bytes
`4e f6 60 22`), `udiv`/`mls` by 0xEA60, guard constants 6499 and 19999,
result stored to `0x14235FC4`. 60000 samples = 7.5 s at 8 kHz. The
23-bit mask in the constructor is a serialization bound, not a wrap
point; the true wrap is mod 60000 samples.

---

## 4. vp.bin — voice prompts, fully parsed (Verified)

**Verified.** `vp.bin` (v4.5.1: 5,211,712 B, magic `"File"` at 0x0) is a
**voice-prompt asset container**, fully reverse-engineered:

- **Record table** at file `0x10`–`0x3197`: 1056 records × 12 B,
  layout `(u16 index, u16 lang, u24 VA, u24 offset, u16 len)`;
  `file_offset = offset × 64` into the audio pool at `0x4285`+.
  Verified by walking all 1056 records with zero out-of-range failures
  and by confirming prompt-name ordering.
- **Name table** at `0x3197`–`0x38A4`: 116 UTF-16LE strings —
  `CN/DC/EN/FN/FR/GM/IT/JP/KR/RS/SP_TARGET` language targets,
  `0.prm`…`219.prm` prompt files, and `NowSpeak ASR` resources
  (`NowSpeakASR.kap`, `data_us/fr/ge/it/jp/ru/uk/es/mc`) — a
  voice-assistant/ASR subsystem previously unknown.
- **Audio pool** `0x4285`–`0x4F8640`: nibble-entropy 4.5–6.9 with long
  zero runs (silence), **no container magics** (no OggS/RIFF/AMR/Speex
  headers) → raw codec bitstream. ~0.15 B/sample ≈ 1.2 bits/sample
  effective — consistent with Speex NB at low rate with VAD silences,
  i.e. **the same Speex NB codec as the mesh** renders the prompts.
- v3.5 `vp.bin`: same structure, 89 prompts.

**Verdict (Verified negative):** no MAC strings, no Speex source strings,
no codec codebooks, no second submode interpretation. vp.bin does not
change rounds 1–2 conclusions, but its prompts are an **encrypted-free,
known-content Speex NB corpus** — see §6 (this is more useful than it
looks).

---

## 5. Protocol gaps closed (Verified unless noted)

### 5.1 Mesh 2 codec unit = 42 bytes, 2 units per record

**Verified.** Constant `0x2A` (42) appears as the Mesh 2 unit size:
`movs r3, #0x2A` at `0x081A4FFC` (payload sizing), and stride-42 indexing
loops at `0x081A8B00`/`0x081A8654` (`movs r0,#0x2A; mla r0,r0,r5,r3`),
each running exactly **6 iterations** — the two 42-byte units are
processed in 6 sub-block passes. `movs r3, #0x54` (84) at `0x081A89F8`
passes the 84-byte body (= 2 × 42) into the jitter-cache writer
`0x081A5158`.

Bit-budget (strong inference, unchanged status): 42 B = 336 bits;
300-bit submode-6 frame + 4 B control ≈ 42 B. Mesh 2 = 2 × 40 ms =
80 ms of audio? No — recheck: round 2 established the playout formula
uses the same 320 B (20 ms) PCM frame for both generations; with
`aggregation = 2`, Mesh 2 carries 2 × 20 ms = **40 ms** per record
(round 2 said 40 ms — consistent; the earlier draft's "2.5×" line-rate
ratio is a radio-timing question, not an audio-duration one).

### 5.2 Probe survey (subtype 9/10): formula and role (Verified)

**Verified — new.** The subtype-9 handler at `0x081A8A0C`
(`cmp r4, #9`) echoes a 1-byte CRC and replies subtype `0x0A` via the
frame-send helper `0x081A7DC0`. The survey consumer at `0x081AAB50`
accumulates per-target: response count, echoed-request CRC byte,
response-CRC byte. **The caller at `0x081A33B2` is the Group Mesh
creator's collision survey**: immediately before the group-ID draw at
`0x081AAABC`–`0x081AAB2C` (`bl 0x082249FC`, `movw r3,#0x270F` = 9999,
`sdiv`, `adds #1`, `uxth`), and **if response count > 0 the candidate ID
is redrawn** (verified at `0x081AAB18`–`0x081AAB2A`).

**Consequence:** subtype 9/10 is the **group-ID collision probe** —
the survey isn't a generic link-quality tool; it directly gates
Group Mesh ID selection. (Baseline assigned it "link/channel-quality
probing" — refined.)

### 5.3 Topology threshold semantics (Verified mechanics)

**Verified — new.** The `[record+4]` field is a **decaying liveness
credit**: reset to `0x5DC` (1500) on every accepted frame
(`movw r3,#0x5DC; str r3,[r0,#4]` at `0x081A7906`, also on hit-path at
`0x081A7872`), thresholded `cmp.w r1, #0x560` (1376) at `0x081AA862`.
A record qualifies only if refreshed within 1500−1376 = **124 timer
ticks** of its last accepted frame. The `[record+0xC]` bitmap is a
**rolling 32-slot heard window** (`lsls r3,#1; orr r3,#1` at
`0x081A787C`), and the topology builder requires **> 22** (`cmp r4,#0x16`
at `0x081AA880`) set bits — i.e. heard in ≥ 23 of the last 32 slots.
Semantics = recency/liveness, **not** LQI. Exact tick rate remains
unknown.

### 5.4 Relay backoff constants (Verified, corrected order)

**Verified.** Per-child relay backoffs at `0x081A7CA6`–`0x081A7CB0`:
`0x3C` (60), `0x46` (70), `0x5A` (90), `0x50` (80) — selected by child
position in a `0x78`-strided child list. Round 1's 60/70/80/90 set is
confirmed; the selection is per-child-index, not per-topology-size (a
round-1 refinement).

### 5.5 Jitter cache structure (Verified, corrects round 2)

**Verified.** The Mesh 2 RX jitter cache is a **6-record × 0x24 (36) B
table at `0x142351D8`** (literal pool verified at `0x081A516C`), not the
round-2 "12-byte ring slot." Mesh 2 RX record stride is `0x54` (84).
The `0x081A5158` signature is `(id, timestamp, body_ptr, body_len=84)`,
with `sdiv` computing 84 / record-payload — i.e. 84-byte bodies are
sliced into 6 × 14 B? No: 6 records × 0x24 B table stride; the 14 B
figure is not supported — the table stores 6 records of 36 B each and
the 84-byte body is distributed across them. Exact per-record payload
split (36 B table stride vs 84 B body) remains **Unknown**.

### 5.6 Subtype 11 — handler found, role partially resolved (Verified mechanics / Strong inference role)

**Verified.** A subtype-11 branch exists at `0x081AA390`
(`cmp r2, #0xB`) in the receive dispatcher: it scans a 19-record table
(`cmp r3, #0x13` = 19 at `0x081AA3D4`), matches a key derived from the
payload against per-record range checks, and echoes a 1-byte reply as
subtype 10.

**Coordinator correction:** the agent claimed the table is at pool
`0x081AA644 = 0x14202640` ("the topology cache"). Verification shows
the subtype-11 handler's literal load at `0x081AA39A` resolves to pool
`0x081AA644` = **`0x14236C3C`**, not `0x14202640`. The 19×4-byte table
lives at `0x14236C3C` (a RAM table not file-backed), so the "subtype 11
reads the topology cache" identification is **retracted**. Role remains
**Strong inference**: a 19-entry poll/echo table consistent with a
gain/volume or group-settings query set (`mesh_intercom_vol` string
family is adjacent in the image). Exact record semantics: Unknown.

### 5.7 Reserved subtypes 3, 4, 12–31

**Verified negative (within analyzed images).** No handler branches for
these subtypes exist in the v4.5.1 receive dispatcher (round 1), and the
v3.5 loader/controller images contain no additional subtype strings or
dispatcher code. Reserved/ignored in both generations. No further static
work possible.

---

## 6. Verification record (coordinator)

Every load-bearing agent claim was re-derived byte-for-byte before
acceptance. Results:

| # | Claim | Independent result |
|---|---|---|
| 1 | `0x14235FD8` writer pool word | ✅ `0x14235FD8` at pool `0x081A7F00` |
| 2 | `movs r3,#0x58` @ `0x081A7E50` | ✅ bytes `58 23` |
| 3 | `movs r0,#0x2A` @ `0x081A8B00` | ✅ bytes `2a 20` |
| 4 | `cmp.w r1,#0x560` @ `0x081AA862` | ✅ bytes `b1 f5 ac 6f` |
| 5 | `movw r3,#0x5DC` @ `0x081A7906` | ✅ bytes `40 f2 dc 53` |
| 6 | `movw r2,#0xEA60` @ `0x081A84C8` | ✅ bytes `4e f6 60 22` (agent's hex was wrong, address right; corrected here) |
| 7 | CCITT-FALSE table @ ldr `0x1A728` | ✅ all 256 entries match poly `0x1021` table exactly |
| 8 | channel tables @ `0x1A710`/`0x1A71C` | ✅ byte-identical to sec1 tables |
| 9 | ldr = ADI BF707 | ✅ magic `01 50 e8 ad`, `BF707` string present |
| 10 | controllers = CSR-dfu2 | ✅ both magic-verified |
| 11 | vp.bin `File` magic + `.prm` names | ✅ |
| 12 | sec0 submode `bits_size` table | ✅ all 8 values byte-exact |
| — | MeshGaps Q9 pool = `0x14202640` | ❌ actual `0x14236C3C` — claim retracted, report corrected |

Two additional round-2 statements were corrected during this pass
(recorded in §5.1 and §5.5); one round-2 hypothesis (NVDM submode) was
downgraded to open (§3).

---

## 7. Remaining Unknowns (updated)

Still requiring RF captures or hardware:

1. **Lower MAC boundary** — now known to live **outside all analyzed
   firmware** (external 802.15.4 radio/sub-MCU, or undumped Blackfin
   regions). Unchanged as a capture task; no remaining static target.
2. **32-byte codec-unit bitstream** — submode selection requires an
   Xtensa disassembler (tooling gap, now precisely characterized) or a
   known-tone capture. **New shortcut:** vp.bin contains 219 named,
   known-content Speex NB prompts encoded by the same DSP; a decoder
   prototype can be validated against vp.bin audio *offline*, without RF
   — cracking the codec-unit layout with known plaintext pairs.
3. **CCNI message IDs** — needs Xtensa disassembly of `aud_msg_init`
   (tooling gap) or a runtime trace.
4. **Topology tick rate** (`0x5DC`→`0x560` decay) — needs a capture or
   timer-clock datasheet.
5. **Subtype-11 table semantics** (`0x14236C3C`, 19×4 B) — runtime RAM,
   needs a trace or capture.
6. **Trailer bytes 15–16 wire behavior** — unchanged.

New tooling recommendation: an Xtensa-capable disassembler
(`xtensa-esp32-elf-objdump` from Espressif's toolchain, or Ghidra's
Xtensa module) unlocks items 2 and 3 statically.

---

## 8. Reproduction

All verification snippets run against the local images with
`knowledge/tools/mesh3.py` (`unpack_container`) plus byte-level
assertions listed in §6. The working copies and agent scratch files are
in `/tmp/sena_r3/` (ephemeral). No repo files were modified by the
agents; this report and `knowledge/` additions are the only outputs.
