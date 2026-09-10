//! In-place hide of Smash Ultimate layout HUD panes (BFLYT) and BFLAN tracks.

const PANE_SECTIONS: [&[u8]; 8] = [
    b"pan1", b"pic1", b"txt1", b"wnd1", b"bnd1", b"prt1", b"cnt1", b"scr1",
];

#[derive(Default)]
pub struct PatchStats {
    pub panes: u32,
    pub anims: u32,
    pub flyt: u32,
    pub flan: u32,
    pub other: u32,
    pub inners: String,
}

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

fn set_u16(data: &mut [u8], off: usize, be: bool, v: u16) {
    if off + 2 > data.len() {
        return;
    }
    let b = if be { v.to_be_bytes() } else { v.to_le_bytes() };
    data[off..off + 2].copy_from_slice(&b);
}

fn set_f32(data: &mut [u8], off: usize, be: bool, v: f32) {
    if off + 4 > data.len() {
        return;
    }
    let b = if be { v.to_be_bytes() } else { v.to_le_bytes() };
    data[off..off + 4].copy_from_slice(&b);
}

fn magic_label(m: &[u8]) -> String {
    if m.len() >= 4 && m.iter().take(4).all(|b| *b >= 0x20 && *b < 0x7f) {
        String::from_utf8_lossy(&m[..4]).into_owned()
    } else if m.len() >= 4 {
        format!("{:02x}{:02x}{:02x}{:02x}", m[0], m[1], m[2], m[3])
    } else {
        "?".into()
    }
}

fn yaz0_decompress(src: &[u8]) -> Option<Vec<u8>> {
    if src.len() < 16 || src.get(0..4) != Some(&b"Yaz0"[..]) {
        return None;
    }
    let dest_end = u32::from_be_bytes([src[4], src[5], src[6], src[7]]) as usize;
    if dest_end == 0 || dest_end > 0x800000 {
        return None;
    }
    let mut dest = Vec::with_capacity(dest_end);
    let mut i = 16usize;
    let mut group = 0u8;
    let mut bits = 0u32;
    while dest.len() < dest_end {
        if bits == 0 {
            if i >= src.len() {
                break;
            }
            group = src[i];
            i += 1;
            bits = 8;
        }
        if group & 0x80 != 0 {
            if i >= src.len() {
                break;
            }
            dest.push(src[i]);
            i += 1;
        } else {
            if i + 1 >= src.len() {
                break;
            }
            let b1 = src[i];
            let b2 = src[i + 1];
            i += 2;
            let dist = (((b1 as usize) & 0xF) << 8) | (b2 as usize);
            let mut copy = ((b1 >> 4) as usize) + 2;
            if copy == 2 {
                if i >= src.len() {
                    break;
                }
                copy = src[i] as usize + 18;
                i += 1;
            }
            for _ in 0..copy {
                if dest.len() <= dist {
                    break;
                }
                let b = dest[dest.len() - dist - 1];
                dest.push(b);
            }
        }
        group = group.wrapping_shl(1);
        bits -= 1;
    }
    dest.truncate(dest_end);
    Some(dest)
}

fn walk_sections(data: &[u8], be: bool, mut on_sec: impl FnMut(usize, &[u8], usize)) {
    let header_size = match u16_at(data, 6, be) {
        Some(v) => v as usize,
        None => return,
    };
    let nsec = match u16_at(data, 0x10, be) {
        Some(v) => v as usize,
        None => return,
    };
    let mut off = header_size;
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
        on_sec(off, magic, size);
        off += size;
        if off % 4 != 0 {
            off += 4 - (off % 4);
        }
    }
}

fn patch_bflyt(data: &mut [u8]) -> u32 {
    if data.get(0..4) != Some(&b"FLYT"[..]) {
        return 0;
    }
    let be = be_bom(data, 4);
    let mut secs: Vec<(usize, usize)> = Vec::new();
    walk_sections(data, be, |off, magic, size| {
        if PANE_SECTIONS.iter().any(|m| *m == magic) {
            secs.push((off, size));
        }
    });
    let hidden = secs.len() as u32;
    for (off, size) in secs {
        data[off + 8] = 0;
        data[off + 10] = 0;
        let body = off + 8;
        if size >= 0x4C {
            set_f32(data, body + 0x24, be, -8000.0);
            set_f32(data, body + 0x3C, be, 0.0);
            set_f32(data, body + 0x40, be, 0.0);
            set_f32(data, body + 0x44, be, 0.0);
            set_f32(data, body + 0x48, be, 0.0);
        }
    }
    hidden
}

fn patch_bflan(data: &mut [u8]) -> u32 {
    if data.get(0..4) != Some(&b"FLAN"[..]) {
        return 0;
    }
    let be = be_bom(data, 4);
    let mut pai: Vec<(usize, usize)> = Vec::new();
    walk_sections(data, be, |off, magic, size| {
        if magic == b"pai1" {
            pai.push((off, size));
        }
    });
    if pai.is_empty() {
        let mut i = 0usize;
        while i + 16 <= data.len() {
            if &data[i..i + 4] == b"pai1" {
                if let Some(size) = u32_at(data, i + 4, be) {
                    let size = size as usize;
                    if size >= 0x12 && i + size <= data.len() {
                        pai.push((i, size));
                        i += size;
                        continue;
                    }
                }
            }
            i += 4;
        }
    }
    for (off, size) in &pai {
        if *size >= 0x10 {
            set_u16(data, off + 0x0C, be, 0);
            set_u16(data, off + 0x0E, be, 0);
        }
    }
    pai.len() as u32
}

fn patch_bytes(data: &mut [u8], stats: &mut PatchStats) {
    let mag = magic_label(data.get(..4).unwrap_or(&[]));
    if !stats.inners.is_empty() {
        stats.inners.push(',');
    }
    stats.inners.push_str(&mag);
    match data.get(0..4) {
        Some(b"FLYT") => {
            stats.flyt += 1;
            stats.panes += patch_bflyt(data);
        }
        Some(b"FLAN") => {
            stats.flan += 1;
            stats.anims += patch_bflan(data);
        }
        Some(b"Yaz0") => {
            if let Some(mut inner) = yaz0_decompress(data) {
                patch_bytes(&mut inner, stats);
            } else {
                stats.other += 1;
            }
        }
        _ => stats.other += 1,
    }
}

/// Patch a decompressed `layout.arc` (SARC) in place.
pub fn patch_layout_arc(data: &mut [u8]) -> PatchStats {
    let mut stats = PatchStats::default();
    if data.get(0..4) != Some(&b"SARC"[..]) {
        patch_bytes(data, &mut stats);
        return stats;
    }
    let be = be_bom(data, 6);
    let header_size = match u16_at(data, 4, be) {
        Some(v) => v as usize,
        None => return stats,
    };
    let data_offset = match u32_at(data, 0xC, be) {
        Some(v) => v as usize,
        None => return stats,
    };
    let sfat = header_size;
    if data.get(sfat..sfat + 4) != Some(&b"SFAT"[..]) {
        return stats;
    }
    let node_count = match u16_at(data, sfat + 6, be) {
        Some(v) => v as usize,
        None => return stats,
    };
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
        patch_bytes(&mut data[lo..hi], &mut stats);
    }
    stats
}
