#!/usr/bin/env python3
"""Patch dumped Smash Ultimate layout.arc files so replay/movie HUD panes stay hidden.

Does not ship Nintendo assets. You dump the vanilla archives; this writes an
ARCropolis mod folder. Live-match HUD is not in the target list.
"""

from __future__ import annotations

import argparse
import io
import struct
import sys
from pathlib import Path

PANE_SECTIONS = {b"pan1", b"pic1", b"txt1", b"wnd1", b"bnd1", b"prt1"}
VIS_TAGS = {b"FLVI", b"RLVI"}
LAYOUTS = (
    "info_movie_recording",
    "info_movie_screen",
)
GAME_PATH = "ui/layout/info/{name}/{name}/layout.arc"
SARC_HASH_MULT = 0x65


def die(msg: str, code: int = 1) -> None:
    print(f"error: {msg}", file=sys.stderr)
    sys.exit(code)


def u16(data: bytes, off: int, be: bool) -> int:
    return struct.unpack_from(">H" if be else "<H", data, off)[0]


def u32(data: bytes, off: int, be: bool) -> int:
    return struct.unpack_from(">I" if be else "<I", data, off)[0]


def p16(be: bool, v: int) -> bytes:
    return struct.pack(">H" if be else "<H", v & 0xFFFF)


def p32(be: bool, v: int) -> bytes:
    return struct.pack(">I" if be else "<I", v & 0xFFFFFFFF)


def yaz0_decompress(src: bytes) -> bytes:
    if src[:4] != b"Yaz0":
        return src
    dest_end = struct.unpack(">I", src[4:8])[0]
    dest = bytearray()
    i = 16
    group = 0
    bits = 0
    while len(dest) < dest_end:
        if bits == 0:
            if i >= len(src):
                break
            group = src[i]
            i += 1
            bits = 8
        if group & 0x80:
            dest.append(src[i])
            i += 1
        else:
            b1 = src[i]
            b2 = src[i + 1]
            i += 2
            dist = ((b1 & 0xF) << 8) | b2
            copy = (b1 >> 4) + 2
            if copy == 2:
                copy = src[i] + 18
                i += 1
            for _ in range(copy):
                dest.append(dest[-dist - 1])
        group = (group << 1) & 0xFF
        bits -= 1
    return bytes(dest[:dest_end])


def sarc_hash(name: str, multiplier: int = SARC_HASH_MULT) -> int:
    h = 0
    for ch in name:
        h = (h * multiplier + ord(ch)) & 0xFFFFFFFF
    return h


def read_sarc(blob: bytes) -> tuple[dict[str, bytes], bool, int]:
    blob = yaz0_decompress(blob)
    if blob[:4] != b"SARC":
        die("not a SARC (expected layout.arc)")
    be = blob[6:8] == b"\xfe\xff"
    header_size = u16(blob, 4, be)
    data_offset = u32(blob, 0xC, be)
    sfat = header_size
    if blob[sfat : sfat + 4] != b"SFAT":
        die("SARC missing SFAT")
    node_count = u16(blob, sfat + 6, be)
    multiplier = u32(blob, sfat + 8, be)
    nodes = sfat + 0xC
    sfnt = nodes + node_count * 16
    if blob[sfnt : sfnt + 4] != b"SFNT":
        die("SARC missing SFNT")
    str_base = sfnt + 8
    files: dict[str, bytes] = {}
    for i in range(node_count):
        n = nodes + i * 16
        file_hash = u32(blob, n, be)
        attr = u32(blob, n + 4, be)
        start = u32(blob, n + 8, be)
        end = u32(blob, n + 12, be)
        if attr & 0x01000000:
            off = str_base + ((attr & 0xFFFF) * 4)
            z = blob.index(b"\x00", off)
            name = blob[off:z].decode("utf-8")
        else:
            name = f"{file_hash:08x}"
        files[name.replace("\\", "/")] = bytes(blob[data_offset + start : data_offset + end])
    return files, be, multiplier


def write_sarc(files: dict[str, bytes], be: bool, multiplier: int) -> bytes:
    names = sorted(files, key=lambda n: sarc_hash(n, multiplier))
    sfat_nodes = bytearray()
    sfnt = bytearray()
    data = bytearray()
    align = 4
    for name in names:
        while len(sfnt) % 4:
            sfnt.append(0)
        str_index = len(sfnt) // 4
        sfnt.extend(name.encode("utf-8") + b"\x00")
        while len(data) % align:
            data.append(0)
        start = len(data)
        payload = files[name]
        data.extend(payload)
        end = len(data)
        sfat_nodes.extend(p32(be, sarc_hash(name, multiplier)))
        sfat_nodes.extend(p32(be, 0x01000000 | (str_index & 0xFFFF)))
        sfat_nodes.extend(p32(be, start))
        sfat_nodes.extend(p32(be, end))

    while len(sfnt) % 4:
        sfnt.append(0)
    sfat = b"SFAT" + p16(be, 0xC) + p16(be, len(names)) + p32(be, multiplier) + bytes(sfat_nodes)
    sfnt_sec = b"SFNT" + p16(be, 8) + p16(be, 0) + bytes(sfnt)
    data_offset = 0x14 + len(sfat) + len(sfnt_sec)
    pad = (4 - (data_offset % 4)) % 4
    data_offset += pad
    file_size = data_offset + len(data)
    bom = b"\xfe\xff" if be else b"\xff\xfe"
    header = b"SARC" + p16(be, 0x14) + bom + p32(be, file_size) + p32(be, data_offset)
    header += p16(be, 0x100) + p16(be, 0)
    return header + sfat + sfnt_sec + (b"\x00" * pad) + bytes(data)


def patch_bflyt(data: bytearray) -> int:
    if data[:4] != b"FLYT":
        return 0
    be = data[4:6] == b"\xfe\xff"
    header_size = u16(data, 6, be)
    nsec = u16(data, 0x10, be)
    off = header_size
    hidden = 0
    for _ in range(nsec):
        if off + 8 > len(data):
            break
        magic = bytes(data[off : off + 4])
        size = u32(data, off + 4, be)
        if size < 8 or off + size > len(data):
            break
        if magic in PANE_SECTIONS:
            data[off + 8] &= 0xFE
            data[off + 10] = 0
            hidden += 1
        off += size
        if off % 4:
            off += 4 - (off % 4)
    return hidden


def _zero_flvi_keys(data: bytearray, tag_start: int, tag_size: int, be: bool) -> int:
    """Zero visibility keyframes inside one FLVI/RLVI tag."""
    end = tag_start + tag_size
    body = tag_start + 8
    if body + 4 > end:
        return 0
    n = u16(data, body, be)
    if n == 0 or n > 64:
        n = u16(data, body + 2, be)
        entry_base = body + 4
        if n == 0 or n > 64:
            return 0
    else:
        entry_base = body + 4
    patched = 0
    pos = entry_base
    for _ in range(n):
        if pos + 12 > end:
            break
        curve = data[pos + 2]
        key_count = u16(data, pos + 4, be)
        key_off = u32(data, pos + 8, be)
        keys = tag_start + key_off
        if keys < pos or keys >= end:
            keys = pos + key_off
        for k in range(key_count):
            if curve == 2:
                slot = keys + k * 12 + 4
                if slot + 4 <= end:
                    data[slot : slot + 4] = b"\x00\x00\x00\x00"
                    patched += 1
            else:
                slot = keys + k * 8 + 4
                if slot + 2 <= end:
                    data[slot : slot + 2] = b"\x00\x00"
                    patched += 1
        pos += 12
    return patched


def patch_bflan(data: bytearray) -> int:
    if data[:4] != b"FLAN":
        return 0
    be = data[4:6] == b"\xfe\xff"
    patched = 0
    i = 0
    while i + 8 <= len(data):
        mag = bytes(data[i : i + 4])
        if mag in VIS_TAGS:
            size = u32(data, i + 4, be)
            if 8 <= size <= len(data) - i:
                patched += _zero_flvi_keys(data, i, size, be)
                i += size
                continue
        i += 4
    return patched


def patch_layout_arc(blob: bytes) -> tuple[bytes, int, int]:
    files, be, multiplier = read_sarc(blob)
    panes = 0
    vis = 0
    out: dict[str, bytes] = {}
    for name, raw in files.items():
        buf = bytearray(raw)
        lower = name.lower()
        if lower.endswith(".bflyt"):
            panes += patch_bflyt(buf)
        elif lower.endswith(".bflan"):
            vis += patch_bflan(buf)
        out[name] = bytes(buf)
    return write_sarc(out, be, multiplier), panes, vis


def find_vanilla(root: Path) -> dict[str, Path]:
    found: dict[str, Path] = {}
    for name in LAYOUTS:
        candidates = [
            root / "ui" / "layout" / "info" / name / name / "layout.arc",
            root / name / "layout.arc",
            root / f"{name}.arc",
            root / name / name / "layout.arc",
        ]
        for path in candidates:
            if path.is_file():
                found[name] = path
                break
        if name not in found:
            for path in root.rglob("layout.arc"):
                if name in path.as_posix():
                    found[name] = path
                    break
    return found


def write_mod(out_dir: Path, patched: dict[str, bytes]) -> None:
    mod = out_dir / "hide-replay-tips"
    if mod.exists():
        for old in sorted(mod.rglob("*"), reverse=True):
            if old.is_file():
                old.unlink()
            elif old.is_dir():
                old.rmdir()
    info_src = Path(__file__).resolve().parents[1] / "mod" / "info.toml"
    dest_info = mod / "info.toml"
    dest_info.parent.mkdir(parents=True, exist_ok=True)
    dest_info.write_bytes(info_src.read_bytes())
    for name, blob in patched.items():
        dest = mod / GAME_PATH.format(name=name)
        dest.parent.mkdir(parents=True, exist_ok=True)
        dest.write_bytes(blob)
        print(f"wrote {dest.relative_to(out_dir).as_posix()} ({len(blob)} bytes)")


def _minimal_bflyt() -> bytes:
    pane = bytearray(0x4C)
    pane[0] = 0x01
    pane[2] = 0xFF
    pane[4:12] = b"RootPane"
    scale = struct.pack("<ff", 1.0, 1.0)
    pane[0x3C:0x44] = scale
    pane[0x44:0x4C] = struct.pack("<ff", 100.0, 100.0)
    section = b"pan1" + struct.pack("<I", 8 + len(pane)) + bytes(pane)
    header = bytearray(0x14)
    header[0:4] = b"FLYT"
    header[4:6] = b"\xff\xfe"
    header[6:8] = struct.pack("<H", 0x14)
    header[8:12] = struct.pack("<I", 0x08000000)
    header[12:16] = struct.pack("<I", 0x14 + len(section))
    header[16:18] = struct.pack("<H", 1)
    return bytes(header) + section


def self_test() -> None:
    flyt = _minimal_bflyt()
    assert flyt[0x14:0x18] == b"pan1"
    assert flyt[0x1C] & 1
    packed = write_sarc({"blyt/test.bflyt": flyt}, be=False, multiplier=SARC_HASH_MULT)
    files, be, _ = read_sarc(packed)
    assert not be
    assert files["blyt/test.bflyt"] == flyt
    out, panes, _vis = patch_layout_arc(packed)
    again, _, _ = read_sarc(out)
    patched = again["blyt/test.bflyt"]
    if panes != 1 or (patched[0x1C] & 1) or patched[0x1E] != 0:
        die(f"self-test failed panes={panes} flags={patched[0x1C]:02x} alpha={patched[0x1E]}")
    print("self-test ok")


def main() -> None:
    here = Path(__file__).resolve().parents[1]
    p = argparse.ArgumentParser(description=__doc__)
    p.add_argument(
        "vanilla",
        nargs="?",
        type=Path,
        default=here / "vanilla",
        help="folder of dumped layout.arc files",
    )
    p.add_argument(
        "-o",
        "--out",
        type=Path,
        default=here / "dist",
        help="output folder (gets hide-replay-tips/)",
    )
    p.add_argument("--self-test", action="store_true", help="run a SARC/BFLYT round-trip check")
    args = p.parse_args()
    if args.self_test:
        self_test()
        return
    vanilla = args.vanilla
    if not vanilla.is_dir():
        die(f"vanilla folder not found: {vanilla}")
    found = find_vanilla(vanilla)
    if not found:
        die(
            "no target layout.arc files. Dump info_movie_recording and/or "
            "info_movie_screen with ArcExplorer into vanilla/ (see README)."
        )
    patched: dict[str, bytes] = {}
    for name, path in found.items():
        blob = path.read_bytes()
        out, panes, vis = patch_layout_arc(blob)
        print(f"{name}: hidden {panes} panes, zeroed {vis} visibility keys ({path})")
        patched[name] = out
    missing = [n for n in LAYOUTS if n not in patched]
    if missing:
        print("warning: not dumped: " + ", ".join(missing), file=sys.stderr)
    write_mod(args.out, patched)
    print(f"\ncopy {args.out / 'hide-replay-tips'} to sd:/ultimate/mods/")


if __name__ == "__main__":
    main()
