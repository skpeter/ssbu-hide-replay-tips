# Hide Replay Tips

Skyline plugin that hides Super Smash Bros. Ultimate's replay playback control overlay — the same HUD that **X + D-pad Down** toggles during playback.

It does **not** ship Nintendo files. ARCropolis loads the vanilla `layout.arc` from `data.arc`; this plugin patches that buffer in place (hide BFLYT panes, disable BFLAN tracks) and hands it back.

The replay controls share `info_pause_camera` with the in-match pause-camera overlay, so that HUD is hidden too. The pause **menu** (`info_pause`) is not hooked.

## Install

Requires [ARCropolis](https://github.com/Raytwo/ARCropolis/releases) (and Skyline, which it already uses).

Extract a [release](https://github.com/skpeter/ssbu-hide-replay-tips/releases) zip onto the SD root, or copy:

```
atmosphere/contents/01006A800016E000/romfs/skyline/plugins/libhide_replay_tips.nro
```

Hold **L** on boot if you need to skip plugins.

## What it patches

| Game path | Why |
|---|---|
| `ui/layout/info/info_movie_recording/.../layout.arc` | Recording / convert-to-video guide |
| `ui/layout/info/info_movie_screen/.../layout.arc` | Replay movie overlay |
| `ui/layout/info/info_movie_result/.../layout.arc` | Replay / convert result overlay |
| `ui/layout/info/info_pause_camera/.../layout.arc` | Replay playback controls (also the pause-camera HUD) |

## Build

```sh
cargo install cargo-skyline
cargo skyline build --release
```

NRO:

```
target/aarch64-skyline-switch/release/libhide_replay_tips.nro
```

SD zip:

```sh
bash scripts/package-sd.sh
# Windows: powershell -File scripts/package-sd.ps1
```

## License

MIT for this plugin. Vanilla layouts stay in the game; they are never redistributed.
