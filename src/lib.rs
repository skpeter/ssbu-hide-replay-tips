#![feature(proc_macro_hygiene)]

mod patch;

use crc32fast::Hasher;
use std::io::Write;

const MAX_LAYOUT: usize = 0x100000;
const LOG_PATH: &str = "sd:/hide_replay_tips.log";

const LAYOUTS: &[&str] = &[
    "ui/layout/info/info_movie_recording/info_movie_recording/layout.arc",
    "ui/layout/info/info_movie_screen/info_movie_screen/layout.arc",
    "ui/layout/info/info_movie_result/info_movie_result/layout.arc",
    "ui/layout/info/info_pause_camera/info_pause_camera/layout.arc",
];

type CallbackFn = extern "C" fn(u64, *mut u8, usize, &mut usize) -> bool;

extern "C" {
    fn arcrop_register_callback(hash: u64, length: usize, cb: CallbackFn);
    fn arcrop_load_file(hash: u64, buffer: *mut u8, length: usize, out_size: &mut usize) -> bool;
}

fn hash40(path: &str) -> u64 {
    let bytes = path.as_bytes();
    let mut hasher = Hasher::new();
    hasher.update(bytes);
    ((bytes.len() as u64) << 32) + u64::from(hasher.finalize())
}

fn log_line(line: &str) {
    let mut file = match std::fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(LOG_PATH)
    {
        Ok(f) => f,
        Err(_) => return,
    };
    let _ = writeln!(file, "{}", line);
}

fn load_original_file(hash: u64, buf: &mut [u8]) -> Option<usize> {
    let mut out = 0usize;
    let ok = unsafe { arcrop_load_file(hash, buf.as_mut_ptr(), buf.len(), &mut out) };
    if ok {
        Some(out)
    } else {
        None
    }
}

extern "C" fn on_layout(hash: u64, data: *mut u8, size: usize, out_size: &mut usize) -> bool {
    let data = unsafe { std::slice::from_raw_parts_mut(data, size) };
    let loaded = match load_original_file(hash, data) {
        Some(n) => n,
        None => {
            log_line(&format!("cb hash={:#x} size={:#x} load=fail", hash, size));
            return false;
        }
    };
    let stats = patch::patch_layout_arc(&mut data[..loaded]);
    *out_size = loaded;
    log_line(&format!(
        "cb hash={:#x} loaded={:#x} panes={} anims={} flyt={} flan={} other={} inner={}",
        hash, loaded, stats.panes, stats.anims, stats.flyt, stats.flan, stats.other, stats.inners
    ));
    true
}

#[skyline::main(name = "hide_replay_tips")]
pub fn main() {
    let _ = std::fs::write(LOG_PATH, "install begin\n");
    unsafe {
        for path in LAYOUTS {
            let hash = hash40(path);
            log_line(&format!("register {} {:#x}", path, hash));
            arcrop_register_callback(hash, MAX_LAYOUT, on_layout);
        }
    }
    log_line("install done");
}
