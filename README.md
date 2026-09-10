# Hide Replay Tips

ARCropolis UI mod that hides Super Smash Bros. Ultimate's replay / movie control overlay — the same HUD that **X + D-pad Down** toggles during playback.

Live matches are unchanged. This repo does **not** ship Nintendo layout files. You dump two `layout.arc` archives from your copy of the game, run the patcher, then drop the output on the SD card.

## What it patches

| Game path | Why |
|---|---|
| `ui/layout/info/info_movie_recording/info_movie_recording/layout.arc` | Recording / convert-to-video guide (REC bar, prompts) |
| `ui/layout/info/info_movie_screen/info_movie_screen/layout.arc` | Replay playback control overlay |

`info_pause` is intentionally left alone so pausing a real match still works.

The patcher turns every BFLYT pane invisible (and alpha 0) and zeroes BFLAN visibility tracks so the "show tips at the start of each replay" animation cannot bring them back.

## Download

[Releases](https://github.com/skpeter/ssbu-hide-replay-tips/releases) attach a patcher zip (scripts + `info.toml`). That zip still needs your dumped `layout.arc` files; it is not a drop-in SD mod.

## Requirements

- [ARCropolis](https://github.com/Raytwo/ARCropolis/releases)
- Python 3.10+ (stdlib only)
- [ArcExplorer](https://github.com/ScanMountGoat/ArcExplorer/releases) to dump the two archives from `data.arc`

## Dump

1. Open Smash Ultimate's `data.arc` in ArcExplorer.
2. Extract:
   - `ui/layout/info/info_movie_recording/info_movie_recording/layout.arc`
   - `ui/layout/info/info_movie_screen/info_movie_screen/layout.arc`
3. Put them under `vanilla/` using either the game folders or flat names:

```
vanilla/ui/layout/info/info_movie_recording/info_movie_recording/layout.arc
vanilla/ui/layout/info/info_movie_screen/info_movie_screen/layout.arc
```

or:

```
vanilla/info_movie_recording.arc
vanilla/info_movie_screen.arc
```

## Build the mod

```sh
python scripts/patch_layouts.py
# optional: python scripts/patch_layouts.py --self-test
```

Output:

```
dist/hide-replay-tips/info.toml
dist/hide-replay-tips/ui/layout/info/...
```

Copy `dist/hide-replay-tips` to `sd:/ultimate/mods/hide-replay-tips`. Enable it in ARCropolis's mod manager (Smash eShop icon).

To zip for SD-root extract:

```sh
# Windows
powershell -File scripts/package-sd.ps1
# Unix
bash scripts/package-sd.sh
```

## Why not Skyline?

Skyline / `skyline-smash` has no "hide replay tips" API. Injecting X + Down would work only if gated to replay playback; this layout replace is replay-only by file path and needs no HID.

## License

MIT for the patcher and packaging. Dumped / patched `layout.arc` files are Nintendo's and must not be committed or redistributed.
