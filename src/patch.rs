//! In-place hide of Smash Ultimate layout HUD panes (BFLYT) and visibility tracks (BFLAN).
//! Mutates a decompressed `layout.arc` SARC; size is unchanged.

const PANE_SECTIONS: [&[u8]; 6] = [b"pan1", b"pic1", b"txt1", b"wnd1", b"bnd1", b"prt1"];
const VIS_TAGS: [&[u8]; 2] = [b"FLVI", b"RLVI"];

fn be_bom(data: &[u8], at: usize) -> bool {
    data.get(at..at + 2) == Some(&b"\xfe\xff"[..])
}

fn u16_at(data: &[u8], off: usize, be: bool) -> Option<u16> {
    let b = data.get(off..off + 2)?;
    Some(if be {
        u16::from_be_bytes([b[0], b[1]])
    } else {
        u16::from_le_bytes([b[0], b[1]])
    })
}

fn u32_at(data: &[u8], off: usize, be: bool) -> Option<u32> {
    let b = data.get(off..off + 4)?;
    Some(if be {
        u32::from_be_bytes([b[0], b[1], b[2], b[3]])
    } else {
        u32::from_le_bytes([b[0], b[1], b[2], b[3]])
    })
}

fn patch_bflyt(data: &mut [u8]) -> u32 {
    if data.get(0..4) != Some(&b"FLYT"[..]) {
        return 0;
    }
    let be = be_bom(data, 4);
    let header_size = match u16_at(data, 6, be) {
        Some(v) => v as usize,
        None => return 0,
    };
    let nsec = match u16_at(data, 0x10, be) {
        Some(v) => v as usize,
        None => return 0,
    };
    let mut off = header_size;
    let mut hidden = 0u32;
    for _ in 0..nsec {
        if off + 8 > data.len() {
            break;
        }
        let magic = &data[off..off + 4];
        let size = match u32_at(data, off + 4, be) {
            Some(v) => v as usize,
            None => break,
        };
        if size < 8 || off + size > data.len() {
            break;
        }
        if PANE_SECTIONS.iter().any(|m| *m == magic) {
            data[off + 8] &= 0xFE;
            data[off + 10] = 0;
            hidden += 1;
        }
        off += size;
        if off % 4 != 0 {
            off += 4 - (off % 4);
        }
    }
    hidden
}

fn zero_flvi_keys(data: &mut [u8], tag_start: usize, tag_size: usize, be: bool) -> u32 {
    let end = tag_start + tag_size;
    let body = tag_start + 8;
    if body + 4 > end {
        return 0;
    }
    let mut n = u16_at(data, body, be).unwrap_or(0);
    if n == 0 || n > 64 {
        n = u16_at(data, body + 2, be).unwrap_or(0);
        if n == 0 || n > 64 {
            return 0;
        }
    }
    let mut pos = body + 4;
    let mut patched = 0u32;
    for _ in 0..n {
        if pos + 12 > end {
            break;
        }
        let curve = data[pos + 2];
        let key_count = u16_at(data, pos + 4, be).unwrap_or(0) as usize;
        let key_off = u32_at(data, pos + 8, be).unwrap_or(0) as usize;
        let mut keys = tag_start + key_off;
        if keys < pos || keys >= end {
            keys = pos + key_off;
        }
        for k in 0..key_count {
            if curve == 2 {
                let slot = keys + k * 12 + 4;
                if slot + 4 <= end {
                    data[slot..slot + 4].fill(0);
                    patched += 1;
                }
            } else {
                let slot = keys + k * 8 + 4;
                if slot + 2 <= end {
                    data[slot..slot + 2].fill(0);
                    patched += 1;
                }
            }
        }
        pos += 12;
    }
    patched
}

fn patch_bflan(data: &mut [u8]) -> u32 {
    if data.get(0..4) != Some(&b"FLAN"[..]) {
        return 0;
    }
    let be = be_bom(data, 4);
    let mut patched = 0u32;
    let mut i = 0usize;
    while i + 8 <= data.len() {
        let mag = &data[i..i + 4];
        if VIS_TAGS.iter().any(|t| *t == mag) {
            if let Some(size) = u32_at(data, i + 4, be) {
                let size = size as usize;
                if size >= 8 && i + size <= data.len() {
                    patched += zero_flvi_keys(data, i, size, be);
                    i += size;
                    continue;
                }
            }
        }
        i += 4;
    }
    patched
}

fn patch_inner(data: &mut [u8]) -> (u32, u32) {
    match data.get(0..4) {
        Some(b"FLYT") => (patch_bflyt(data), 0),
        Some(b"FLAN") => (0, patch_bflan(data)),
        _ => (0, 0),
    }
}

/// Patch a decompressed `layout.arc` (SARC) in place. Returns (panes hidden, vis keys zeroed).
pub fn patch_layout_arc(data: &mut [u8]) -> (u32, u32) {
    if data.get(0..4) != Some(&b"SARC"[..]) {
        return patch_inner(data);
    }
    let be = be_bom(data, 6);
    let header_size = match u16_at(data, 4, be) {
        Some(v) => v as usize,
        None => return (0, 0),
    };
    let data_offset = match u32_at(data, 0xC, be) {
        Some(v) => v as usize,
        None => return (0, 0),
    };
    let sfat = header_size;
    if data.get(sfat..sfat + 4) != Some(&b"SFAT"[..]) {
        return (0, 0);
    }
    let node_count = match u16_at(data, sfat + 6, be) {
        Some(v) => v as usize,
        None => return (0, 0),
    };
    let mut panes = 0u32;
    let mut vis = 0u32;
    let nodes = sfat + 0xC;
    for i in 0..node_count {
        let n = nodes + i * 16;
        let start = match u32_at(data, n + 8, be) {
            Some(v) => v as usize,
            None => break,
        };
        let end = match u32_at(data, n + 12, be) {
            Some(v) => v as usize,
            None => break,
        };
        let lo = data_offset + start;
        let hi = data_offset + end;
        if lo >= hi || hi > data.len() {
            continue;
        }
        let (p, v) = patch_inner(&mut data[lo..hi]);
        panes += p;
        vis += v;
    }
    (panes, vis)
}
