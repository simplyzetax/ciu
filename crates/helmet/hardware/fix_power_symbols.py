#!/usr/bin/env python3
"""Fix copperhead's phantom copperhead_power:* refs -> real installed KiCad symbols.

- GND, 3V3, PWR_FLAG  -> replaced with the real symbols from KiCad's own
  power.kicad_sym (these are genuinely "installed" per copperhead's policy).
- AVDD                -> has no standard power-library equivalent, so each
  instance is converted into a plain net label "AVDD" at the same point.
  This is electrically identical for a single-sheet design (matches how
  MOSI/MISO/SCLK/etc. are already wired in this schematic via (label ...)).

Run this from inside crates/helmet/hardware, on the file that already has
content (i.e. AFTER `git stash apply`, not on the empty scaffold).
"""
import re
import sys
from pathlib import Path

SCH_PATH = Path(sys.argv[1] if len(sys.argv) > 1 else
                 "at86rf233-spi-carrier-board-for-esp32-c6-host.kicad_sch")
POWER_LIB = Path("/Applications/KiCad/KiCad.app/Contents/SharedSupport/symbols/power.kicad_sym")

# schematic-side name -> real symbol name inside power.kicad_sym
POWER_MAPPING = {
    "GND": "GND",
    "3V3": "+3V3",
    "PWR_FLAG": "PWR_FLAG",
}


def extract_balanced(text: str, start: int) -> str:
    depth = 0
    for i in range(start, len(text)):
        if text[i] == "(":
            depth += 1
        elif text[i] == ")":
            depth -= 1
            if depth == 0:
                return text[start:i + 1]
    raise ValueError("unbalanced parens")


def find_symbol_def_block(lib_text: str, name: str) -> str:
    pattern = re.compile(r'\(symbol\s+"' + re.escape(name) + r'"')
    m = pattern.search(lib_text)
    if not m:
        raise ValueError(f"symbol {name!r} not found in {POWER_LIB}")
    return extract_balanced(lib_text, m.start())


def rename_top_level(block: str, old_name: str, new_full_name: str) -> str:
    return block.replace(f'(symbol "{old_name}"', f'(symbol "{new_full_name}"', 1)


def strip_lib_symbol_def(sch_text: str, lib_id: str) -> str:
    """Remove a (symbol "copperhead_power:XXX" ...) block from lib_symbols."""
    pattern = re.compile(r'\n?\t*\(symbol\s+"' + re.escape(lib_id) + r'"')
    m = pattern.search(sch_text)
    if not m:
        return sch_text
    start = sch_text.index("(symbol", m.start())
    block = extract_balanced(sch_text, start)
    return sch_text[:m.start()] + sch_text[m.start() + len(sch_text[m.start():start]) + len(block):] \
        if False else (sch_text[:sch_text.index(block)] + sch_text[sch_text.index(block) + len(block):])


def find_all_instance_blocks(sch_text: str, lib_id: str):
    """Find every (symbol (lib_id "copperhead_power:AVDD") ...) instance block."""
    blocks = []
    pattern = re.compile(r'\(symbol\s*\n\t*\(lib_id\s+"' + re.escape(lib_id) + r'"\)')
    for m in pattern.finditer(sch_text):
        start = sch_text.rindex("(symbol", 0, m.start() + 8)
        block = extract_balanced(sch_text, start)
        blocks.append((start, block))
    return blocks


def get_at_and_uuid(block: str):
    at_m = re.search(r'\(at\s+([\-0-9.]+)\s+([\-0-9.]+)\s+([\-0-9.]+)\)', block)
    uuid_m = re.search(r'\(uuid\s+"([0-9a-f-]+)"\)', block)
    x, y, angle = at_m.group(1), at_m.group(2), at_m.group(3)
    uuid = uuid_m.group(1)
    return x, y, angle, uuid


def main():
    lib_text = POWER_LIB.read_text()
    sch_text = SCH_PATH.read_text()

    # --- 1. Inject real GND / +3V3 / PWR_FLAG symbol defs into lib_symbols ---
    new_blocks = []
    for schem_name, lib_name in POWER_MAPPING.items():
        block = find_symbol_def_block(lib_text, lib_name)
        block = rename_top_level(block, lib_name, f"power:{lib_name}")
        new_blocks.append(block)

    lib_symbols_open = sch_text.find("(lib_symbols")
    if lib_symbols_open == -1:
        raise ValueError("no (lib_symbols block found")
    insert_at = lib_symbols_open + len("(lib_symbols")
    injected = "\n\t\t" + "\n\t\t".join(new_blocks)
    sch_text = sch_text[:insert_at] + injected + sch_text[insert_at:]

    # --- 2. Remove the four phantom copperhead_power:* defs from lib_symbols ---
    for schem_name in list(POWER_MAPPING.keys()) + ["AVDD"]:
        sch_text = strip_lib_symbol_def(sch_text, f"copperhead_power:{schem_name}")

    # --- 3. Rewrite GND / 3V3 / PWR_FLAG instance lib_id references ---
    for schem_name, lib_name in POWER_MAPPING.items():
        sch_text = sch_text.replace(
            f'(lib_id "copperhead_power:{schem_name}")',
            f'(lib_id "power:{lib_name}")',
        )

    # --- 4. Convert every AVDD power-symbol instance into a local net label ---
    avdd_blocks = find_all_instance_blocks(sch_text, "copperhead_power:AVDD")
    # Replace from the end so earlier offsets stay valid
    for start, block in sorted(avdd_blocks, key=lambda b: b[0], reverse=True):
        x, y, angle, old_uuid = get_at_and_uuid(block)
        label = (
            f'(label "AVDD" (at {x} {y} {angle})\n'
            f'\t\t(effects (font (size 1.27 1.27)))\n'
            f'\t\t(uuid "{old_uuid}")\n'
            f'\t)'
        )
        sch_text = sch_text[:start] + label + sch_text[start + len(block):]

    # --- 5. Drop the now-unused copperhead_power entry from sym-lib-table ---
    sym_lib_table = SCH_PATH.parent / "sym-lib-table"
    if sym_lib_table.exists():
        t = sym_lib_table.read_text()
        t = re.sub(r'\n?\s*\(lib \(name "copperhead_power"\).*?\)\)\)', '', t, flags=re.S)
        sym_lib_table.write_text(t)
        print(f"cleaned {sym_lib_table}")

    SCH_PATH.write_text(sch_text)
    print(f"patched {SCH_PATH}: {len(avdd_blocks)} AVDD instance(s) -> labels, "
          f"{len(POWER_MAPPING)} power symbol types -> real power:* symbols")


if __name__ == "__main__":
    main()
