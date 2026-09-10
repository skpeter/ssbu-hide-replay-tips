#![feature(proc_macro_hygiene)]

mod patch;

use arcropolis_api::{arc_callback, load_original_file};

const MAX_LAYOUT: usize = 0x100000;

const LAYOUTS: &[&str] = &[
    "ui/layout/info/info_movie_recording/info_movie_recording/layout.arc",
    "ui/layout/info/info_movie_screen/info_movie_screen/layout.arc",
];

#[arc_callback]
fn on_layout(hash: u64, data: &mut [u8]) -> Option<usize> {
    let size = load_original_file(hash, &mut *data)?;
    let (panes, vis) = patch::patch_layout_arc(&mut data[..size]);
    println!(
        "[hide_replay_tips] {:#x} hidden panes={} vis_keys={}",
        hash, panes, vis
    );
    Some(size)
}

#[skyline::main(name = "hide_replay_tips")]
pub fn main() {
    for path in LAYOUTS {
        on_layout::install(*path, MAX_LAYOUT);
        println!("[hide_replay_tips] callback {}", path);
    }
}
