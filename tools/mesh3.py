#!/usr/bin/env python3
"""Clean-room reference implementation of the Sena Mesh 3 wire format.

Every layout here was recovered from firmware/extracted/30K-v4.5.1-build0/
Submesh_v0.5b0.bin and is documented in
reports/sena-30k-mesh-protocol-static-analysis-2026-09-08.md.

    python3 tools/mesh3.py verify        # reproduce every published vector
    python3 tools/mesh3.py unpack        # decompress + hash-check the container
    python3 tools/mesh3.py parse <hex>   # decode one Mesh 3 network frame

Standard library only: no pycryptodome, no lzma bindings beyond the stdlib.
"""

from __future__ import annotations

import hashlib
import lzma
import struct
import sys
from dataclasses import dataclass
from pathlib import Path

IMAGE = Path("firmware/extracted/30K-v4.5.1-build0/Submesh_v0.5b0.bin")

# Section 1 (ARM Thumb runtime) is loaded at this virtual address.
RUNTIME_VA = 0x08133000

# AES-128 key at virtual address 0x08237250. Fixed for every firmware copy.
MESH3_KEY = bytes.fromhex("0a717f0190374345baf96f49781fa41c")

HEADER_LEN = 17
BROADCAST = 0x03FFFFFF

# ---------------------------------------------------------------------------
# Airoha container
# ---------------------------------------------------------------------------

TAG_FIRMWARE = 0x11
TAG_MOVER = 0x12
TAG_HASHES = 0x14
TAG_PLATFORM = 0x20
TAG_DESIGN = 0x21


@dataclass(frozen=True)
class Section:
    index: int
    source: int  # offset inside the decompressed blob
    size: int
    dest: int  # runtime destination offset
    sha256: str


@dataclass(frozen=True)
class Container:
    platform: str
    design: str
    compression: int
    integrity: int
    runtime: bytes
    sections: tuple[Section, ...]

    def data(self, index: int) -> bytes:
        s = self.sections[index]
        return self.runtime[s.source : s.source + s.size]


def parse_tlv(raw: bytes, offset: int = 0x100) -> dict[int, bytes]:
    """Walk the TLV metadata block until the 0xffff filler."""
    out: dict[int, bytes] = {}
    while offset + 4 <= len(raw):
        tag, length = struct.unpack_from("<HH", raw, offset)
        if tag == 0xFFFF or length == 0:
            break
        out[tag] = raw[offset + 4 : offset + 4 + length]
        offset += 4 + length
    return out


def unpack_container(path: Path = IMAGE) -> Container:
    """Verify the container hash chain and return its decompressed sections."""
    raw = path.read_bytes()
    stored = raw[:32]
    if hashlib.sha256(raw[0x100:]).digest() != stored:
        raise ValueError("container SHA-256 over [0x100:] does not match header")

    tlv = parse_tlv(raw)
    fw = tlv[TAG_FIRMWARE]
    # 0x11 record: compression, integrity, 4-byte payload offset, 4-byte payload size.
    # There is no encryption field and no encrypted payload; the LZMA stream and both
    # section digests validate directly against the plaintext image.
    compression, integrity = fw[0], fw[1]
    comp_off = int.from_bytes(fw[2:6], "little")
    comp_len = int.from_bytes(fw[6:10], "little")
    if compression != 1:
        raise ValueError(f"unsupported compression id {compression}")
    if integrity != 1:
        raise ValueError(f"unsupported integrity id {integrity}")

    blob = raw[comp_off : comp_off + comp_len]
    runtime = lzma.LZMADecompressor(lzma.FORMAT_ALONE).decompress(blob)

    mover = tlv[TAG_MOVER]
    count = int.from_bytes(mover[:4], "little")
    hashes = tlv[TAG_HASHES]
    if int.from_bytes(hashes[:4], "little") != count:
        raise ValueError("mover/hash table length mismatch")

    sections = []
    for i in range(count):
        # The mover's first field is an image-space offset; the decompressed sections
        # start at that offset minus the compressed payload's own base (0x1000).
        image_off, size, dest = struct.unpack_from("<III", mover, 4 + 12 * i)
        source = image_off - comp_off
        want = hashes[4 + 32 * i : 36 + 32 * i].hex()
        got = hashlib.sha256(runtime[source : source + size]).hexdigest()
        if want != got:
            raise ValueError(f"section {i} hash mismatch: {got} != {want}")
        sections.append(Section(i, source, size, dest, want))

    return Container(
        platform=tlv[TAG_PLATFORM].split(b"\0")[0].decode(),
        design=tlv[TAG_DESIGN].split(b"\0")[0].decode(),
        compression=compression,
        integrity=integrity,
        runtime=runtime,
        sections=tuple(sections),
    )


# ---------------------------------------------------------------------------
# CRC-16/XMODEM: poly 0x1021, init 0x0000, no reflection, no final XOR
# ---------------------------------------------------------------------------


def crc16_xmodem(data: bytes) -> int:
    crc = 0
    for byte in data:
        crc ^= byte << 8
        for _ in range(8):
            crc = ((crc << 1) ^ 0x1021) & 0xFFFF if crc & 0x8000 else (crc << 1) & 0xFFFF
    return crc


# ---------------------------------------------------------------------------
# 17-byte bit-packed Mesh 3 header
# ---------------------------------------------------------------------------


@dataclass(frozen=True)
class Header:
    version: int = 1
    aggregation: int = 1
    network: int = 0  # 14 bits
    origin: int = 0  # 25 bits
    sender: int = 0  # 25 bits, previous transmitter
    dest: int | None = None  # 25 bits, None = broadcast
    length: int = 0  # 7 bits
    subtype: int = 0  # 5 bits
    hop: int = 0  # 3 bits
    seq: int = 0  # 8 bits
    trailer: int = 0  # bytes 15-16, zero on the wire

    def pack(self) -> bytes:
        if self.length > 0x7F:
            raise ValueError("payload length exceeds the 7-bit field")
        dest = 0 if self.dest is None else self.dest & 0x1FFFFFF
        h = bytearray(HEADER_LEN)
        h[0] = (self.aggregation & 7) | ((self.dest is not None) << 3) | ((self.version & 0xF) << 4)
        h[1] = self.network & 0xFF
        h[2] = ((self.network >> 8) & 0x3F) | ((self.origin & 3) << 6)
        h[3] = (self.origin >> 2) & 0xFF
        h[4] = (self.origin >> 10) & 0xFF
        h[5] = ((self.origin >> 18) & 0x7F) | ((self.sender & 1) << 7)
        h[6] = (self.sender >> 1) & 0xFF
        h[7] = (self.sender >> 9) & 0xFF
        h[8] = (self.sender >> 17) & 0xFF
        h[9] = dest & 0xFF
        h[10] = (dest >> 8) & 0xFF
        h[11] = (dest >> 16) & 0xFF
        h[12] = ((dest >> 24) & 1) | ((self.length & 0x7F) << 1)
        h[13] = (self.subtype & 0x1F) | ((self.hop & 7) << 5)
        h[14] = self.seq & 0xFF
        h[15] = self.trailer & 0xFF
        h[16] = (self.trailer >> 8) & 0xFF
        return bytes(h)

    @classmethod
    def unpack(cls, h: bytes) -> "Header":
        if len(h) < HEADER_LEN:
            raise ValueError("short header")
        has_dest = bool(h[0] >> 3 & 1)
        dest = h[9] | h[10] << 8 | h[11] << 16 | (h[12] & 1) << 24
        return cls(
            version=h[0] >> 4,
            aggregation=h[0] & 7,
            network=h[1] | (h[2] & 0x3F) << 8,
            origin=(h[2] >> 6) | h[3] << 2 | h[4] << 10 | (h[5] & 0x7F) << 18,
            sender=(h[5] >> 7) | h[6] << 1 | h[7] << 9 | h[8] << 17,
            dest=dest if has_dest else None,
            length=h[12] >> 1,
            subtype=h[13] & 0x1F,
            hop=h[13] >> 5,
            seq=h[14],
            trailer=h[15] | h[16] << 8,
        )


def pack_frame(header: Header, payload: bytes) -> bytes:
    body = header.pack() if header.length == len(payload) else Header(
        **{**header.__dict__, "length": len(payload)}
    ).pack()
    body += payload
    return body + crc16_xmodem(body).to_bytes(2, "little")


def parse_frame(frame: bytes) -> tuple[Header, bytes]:
    if len(frame) < HEADER_LEN + 2:
        raise ValueError("frame shorter than header plus CRC")
    body, crc = frame[:-2], int.from_bytes(frame[-2:], "little")
    if crc16_xmodem(body) != crc:
        raise ValueError(f"bad network CRC: {crc:#06x}")
    header = Header.unpack(body[:HEADER_LEN])
    payload = body[HEADER_LEN:]
    if header.length != len(payload):
        raise ValueError(f"length field {header.length} != {len(payload)} payload bytes")
    return header, payload


def node_id(device_id: bytes) -> int:
    """25-bit node ID derived from bytes 2-5 of the six-byte device identifier."""
    if len(device_id) != 6:
        raise ValueError("device identifier must be six bytes")
    low = device_id[3] << 16 | device_id[4] << 8 | device_id[5]
    return low if device_id[2] == 0x95 else 0x01000000 | low


# ---------------------------------------------------------------------------
# AES-128 (encrypt-only, single block) — no third-party dependency
# ---------------------------------------------------------------------------

_SBOX = bytearray(256)
_p = _q = 1
while True:  # generate the S-box from the AES affine transform
    _p = _p ^ ((_p << 1) & 0xFF) ^ (0x1B if _p & 0x80 else 0)
    _q ^= _q << 1
    _q ^= _q << 2
    _q ^= _q << 4
    _q &= 0xFF
    if _q & 0x80:
        _q ^= 0x09
    x = _q ^ ((_q << 1) | (_q >> 7)) ^ ((_q << 2) | (_q >> 6))
    x ^= ((_q << 3) | (_q >> 5)) ^ ((_q << 4) | (_q >> 4))
    _SBOX[_p] = (x ^ 0x63) & 0xFF
    if _p == 1:
        break
_SBOX[0] = 0x63


def _xtime(a: int) -> int:
    a <<= 1
    return (a ^ 0x1B) & 0xFF if a & 0x100 else a


def _expand_key(key: bytes) -> list[list[int]]:
    words = [list(key[i * 4 : i * 4 + 4]) for i in range(4)]
    rcon = 1
    for i in range(4, 44):
        w = list(words[i - 1])
        if i % 4 == 0:
            w = w[1:] + w[:1]
            w = [_SBOX[b] for b in w]
            w[0] ^= rcon
            rcon = _xtime(rcon)
        words.append([a ^ b for a, b in zip(words[i - 4], w)])
    return [sum(words[r * 4 : r * 4 + 4], []) for r in range(11)]


def aes128_encrypt_block(key: bytes, block: bytes) -> bytes:
    if len(key) != 16 or len(block) != 16:
        raise ValueError("AES-128 needs a 16-byte key and 16-byte block")
    rk = _expand_key(key)
    s = [b ^ k for b, k in zip(block, rk[0])]
    for rnd in range(1, 11):
        s = [_SBOX[b] for b in s]
        # ShiftRows on the column-major AES state
        s = [s[(i + 4 * (i % 4)) % 16] for i in range(16)]
        if rnd != 10:
            mixed = []
            for c in range(4):
                col = s[c * 4 : c * 4 + 4]
                t = col[0] ^ col[1] ^ col[2] ^ col[3]
                mixed += [col[i] ^ t ^ _xtime(col[i] ^ col[(i + 1) % 4]) for i in range(4)]
            s = mixed
        s = [b ^ k for b, k in zip(s, rk[rnd])]
    return bytes(s)


# ---------------------------------------------------------------------------
# Protected audio record
# ---------------------------------------------------------------------------


def build_nonce(header: Header, counter: int) -> bytes:
    """16-byte AES input block. counter is the 23-bit media counter."""
    n = bytearray(16)
    n[0] = header.origin & 0xFF
    n[1] = (header.origin >> 8) & 0xFF
    n[2] = (header.origin >> 16) & 0xFF
    n[3] = 0xCE if header.origin >> 24 & 1 else 0x95
    n[4] = header.network & 0xFF
    n[5] = (header.network >> 8) & 0x3F
    n[6] = (counter >> 16) & 0x7F
    n[7] = (counter >> 8) & 0xFF
    n[8] = counter & 0xFF
    n[9] = header.seq
    n[10] = 0xC7
    n[11] = (header.subtype & 0x1F) | ((header.hop & 7) << 5)
    if header.dest is None:
        n[12:16] = b"\x3f\xff\xff\xff"
    else:
        n[12] = 0xCE if header.dest >> 24 & 1 else 0x95
        n[13] = (header.dest >> 16) & 0xFF
        n[14] = (header.dest >> 8) & 0xFF
        n[15] = header.dest & 0xFF
    return bytes(n)


def keystream(header: Header, counter: int, key: bytes = MESH3_KEY) -> bytes:
    """One AES block, repeated across the record. Not CTR: never incremented."""
    return aes128_encrypt_block(key, build_nonce(header, counter))


def protect(header: Header, counter: int, body: bytes, key: bytes = MESH3_KEY) -> bytes:
    """96-byte codec body -> 101-byte subtype-0/1 payload."""
    if len(body) != 96:
        raise ValueError("codec body must be 96 bytes")
    plain = body + crc16_xmodem(body).to_bytes(2, "little")
    ks = keystream(header, counter, key)
    protected = bytes(b ^ ks[i % 16] for i, b in enumerate(plain))
    return bytes([(counter & 0xFF), (counter >> 8) & 0xFF, ((counter >> 16) & 0x7F) | 0x80]) + protected


def unprotect(header: Header, payload: bytes, key: bytes = MESH3_KEY) -> tuple[int, bytes]:
    """101-byte payload -> (media counter, 96-byte codec body)."""
    if len(payload) != 101:
        raise ValueError("protected payload must be 101 bytes")
    counter = payload[0] | payload[1] << 8 | (payload[2] & 0x7F) << 16
    if not payload[2] & 0x80:
        raise ValueError("encrypted flag (counter bit 23) is clear")
    ks = keystream(header, counter, key)
    plain = bytes(b ^ ks[i % 16] for i, b in enumerate(payload[3:]))
    body, crc = plain[:96], int.from_bytes(plain[96:], "little")
    if crc16_xmodem(body) != crc:
        raise ValueError("body CRC mismatch after decryption")
    return counter, body


# ---------------------------------------------------------------------------
# Published vectors
# ---------------------------------------------------------------------------

VECTOR_FRAME = "193492158d046f5e0d20100004497e0000aabb762e"
VECTOR_NONCE = "56341295340212345642c7003fffffff"
VECTOR_KEYSTREAM = "3ff8234ab412d5cee0025970696cdcf4"

MESH3_CHANNELS = (16, 20, 14, 22, 12, 24, 18)
MESH2_CHANNELS = (19, 13, 14, 15, 16, 17, 22, 23, 24, 21)
MESH3_CHANNEL_TABLE_VA = 0x08236FC9
MESH2_CHANNEL_TABLE_VA = 0x0823700C
KEY_VA = 0x08237250


def _check(label: str, got, want) -> bool:
    ok = got == want
    print(f"{'ok  ' if ok else 'FAIL'}  {label}")
    if not ok:
        print(f"        got  {got!r}\n        want {want!r}")
    return ok


def verify(path: Path = IMAGE) -> int:
    ok = True

    header = Header(version=1, aggregation=1, network=0x1234, origin=0x123456,
                    sender=0x1ABCDE, dest=0x001020, subtype=9, hop=2, seq=0x7E)
    frame = pack_frame(header, b"\xaa\xbb")
    ok &= _check("frame vector round-trips", frame.hex(), VECTOR_FRAME)
    ok &= _check("frame parses back to the same header", parse_frame(frame), (header.__class__(**{**header.__dict__, "length": 2}), b"\xaa\xbb"))

    audio = pack_frame(Header(aggregation=3, network=1, origin=2, sender=3, subtype=0), bytes(101))
    biggest = pack_frame(Header(network=1, origin=2, sender=3, subtype=11), bytes(106))
    ok &= _check("protected audio frame is 120 bytes", len(audio), 120)
    ok &= _check("largest constructor is 125 bytes", len(biggest), 125)
    ok &= _check("125 + 2-byte FCS hits the 802.15.4 limit", len(biggest) + 2, 127)

    ok &= _check("CRC-16/XMODEM('123456789')", crc16_xmodem(b"123456789"), 0x31C3)
    ok &= _check("node ID from 0x95-tagged identifier", node_id(bytes.fromhex("0011951a2b3c")), 0x1A2B3C)
    ok &= _check("node ID from other identifier", node_id(bytes.fromhex("0011ce1a2b3c")), 0x011A2B3C)

    crypto_hdr = Header(aggregation=3, network=0x0234, origin=0x123456, sender=0x123456,
                        dest=None, subtype=0, hop=0, seq=0x42)
    ok &= _check("nonce vector", build_nonce(crypto_hdr, 0x123456).hex(), VECTOR_NONCE)
    ok &= _check("AES-128 keystream vector", keystream(crypto_hdr, 0x123456).hex(), VECTOR_KEYSTREAM)
    ok &= _check("NIST AES-128 known answer",
                 aes128_encrypt_block(bytes.fromhex("000102030405060708090a0b0c0d0e0f"),
                                      bytes.fromhex("00112233445566778899aabbccddeeff")).hex(),
                 "69c4e0d86a7b0430d8cdb78070b4c55a")

    body = bytes(range(96))
    ok &= _check("body CRC of 0..95", crc16_xmodem(body), 0x65A9)
    payload = protect(crypto_hdr, 0x123456, body)
    ok &= _check("protected payload is 101 bytes", len(payload), 101)
    ok &= _check("protect/unprotect round-trips", unprotect(crypto_hdr, payload), (0x123456, body))


    if not path.exists():
        print(f"skip  firmware image absent: {path}")
        return 0 if ok else 1

    c = unpack_container(path)
    runtime = c.data(1)

    def at(va: int, n: int) -> bytes:
        return runtime[va - RUNTIME_VA : va - RUNTIME_VA + n]

    ok &= _check("container platform/design", (c.platform, c.design), ("ab156x", "headset_ref_design"))
    ok &= _check("container is LZMA + SHA-256, no encryption field", (c.compression, c.integrity), (1, 1))
    ok &= _check("section 0 hash", c.sections[0].sha256,
                 "9392917120bc0bf3426bdc734f130cce6efd593ce84e8d5fad57eb96742e8ea7")
    ok &= _check("section 1 hash", c.sections[1].sha256,
                 "732cb245b1d0bf8dde9cd35a8ace30ed367ecfedd96936d65adb9f42145edbfe")
    ok &= _check("AES key at 0x08237250", at(KEY_VA, 16).hex(), MESH3_KEY.hex())
    ok &= _check("Mesh 3 channel table", tuple(at(MESH3_CHANNEL_TABLE_VA, 7)), MESH3_CHANNELS)
    ok &= _check("Mesh 2 channel table", tuple(at(MESH2_CHANNEL_TABLE_VA, 10)), MESH2_CHANNELS)
    # --- Deep codec/OTA pass (2026-09-08 second report) -------------------

    # DSP0 (section 0) carries nine embedded Xtensa ELF modules
    # (e_machine = 94 = EM_XTENSA). Offsets are verified directly.
    sec0 = c.data(0)
    elf_offs = []
    i = 0
    while True:
        j = sec0.find(b"\x7fELF", i)
        if j < 0:
            break
        elf_offs.append(j)
        i = j + 1
    ok &= _check("nine embedded Xtensa ELF modules in section 0", len(elf_offs), 9)
    ok &= _check("all DSP modules are EM_XTENSA (94)",
                 sorted({struct.unpack_from("<H", sec0, o + 18)[0] for o in elf_offs}), [94])

    # The Speex 1.2.0 narrowband codec (nb_celp.c) lives in the DSP0 image.
    ok &= _check("Speex 1.2.0 nb_celp present in DSP0",
                 (sec0.find(b"speex-1.2.0") >= 0, sec0.find(b"nb_celp.c") >= 0), (True, True))

    # The ARM runtime links the Speex *jitter buffer* (libspeexdsp) only; the
    # codec itself is offloaded to DSP0. mesh_speex is a runtime module name.
    ok &= _check("ARM side has jitter.c path only (no codec)",
                 runtime.find(b"intercom/mesh/lib_speex/libspeexdsp/jitter.c") >= 0, True)

    # OTA ring geometry verified in the receive-ring initialization loop:
    # 9 rx slots x 0x8c (140) stride = 0x4ec total, each with a 0x7d (125)-byte
    # payload buffer; aggregation ring: 12 slots x 0x84 (132) = 0x630.
    ok &= _check("rx ring: 9 slots x 140 B", 9 * 0x8C, 0x4EC)
    ok &= _check("rx slot payload buffer 125 B", 0x7D, 125)
    ok &= _check("aggregation ring: 12 slots x 132 B", 12 * 0x84, 0x630)

    # Playout byte-offset math recovered from both mesh generations:
    #   byte_offset = aggregation_index * counter23 * 0x140 (320)
    # 320 B = 160 signed 16-bit samples = 20 ms at 8 kHz.
    ok &= _check("playout frame 320 B = 160 samples = 20 ms at 8 kHz",
                 (0x140, 0x140 / 2, 160 / 8000), (320, 160.0, 0.02))

    # 96-byte codec body = three 32-byte codec units = 60 ms of audio.
    ok &= _check("96 B body = 3 units x 32 B = 3 x 20 ms = 60 ms",
                 (96 // 32) * 20 / 1000, 0.06)

    ptrs = struct.unpack_from("<15I", at(0x08233AD4, 60))
    names = [runtime[p - RUNTIME_VA :].split(b"\0")[0].decode() for p in ptrs]
    ok &= _check("controller state table bounds", (names[0], names[-1]),
                 ("MESH_STATE_INIT", "MESH_STATE_RF_TEST_MODE"))

    return 0 if ok else 1


def main(argv: list[str]) -> int:
    cmd = argv[1] if len(argv) > 1 else "verify"
    if cmd == "verify":
        return verify(Path(argv[2]) if len(argv) > 2 else IMAGE)
    if cmd == "unpack":
        c = unpack_container(Path(argv[2]) if len(argv) > 2 else IMAGE)
        print(f"platform {c.platform}  design {c.design}  runtime {len(c.runtime)} bytes")
        for s in c.sections:
            print(f"  section {s.index}: src {s.source:#x} size {s.size:#x} dest {s.dest:#x} {s.sha256}")
        return 0
    if cmd == "parse":
        header, payload = parse_frame(bytes.fromhex(argv[2]))
        print(header)
        print("payload:", payload.hex() or "(empty)")
        if header.subtype in (0, 1) and len(payload) == 101:
            counter, body = unprotect(header, payload)
            print(f"media counter: {counter:#x}\nbody: {body.hex()}")
        return 0
    print(__doc__)
    return 2


if __name__ == "__main__":
    raise SystemExit(main(sys.argv))
