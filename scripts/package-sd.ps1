$ErrorActionPreference = "Stop"
$Root = Split-Path -Parent $PSScriptRoot
$Nro = Join-Path $Root "target/aarch64-skyline-switch/release/libhide_replay_tips.nro"
$Zip = Join-Path $Root "hide-replay-tips-sd.zip"
if (-not (Test-Path $Nro)) {
    Write-Error "missing $Nro; cargo skyline build --release first"
}
$Stage = Join-Path $Root "dist/sd"
$Dest = Join-Path $Stage "atmosphere/contents/01006A800016E000/romfs/skyline/plugins"
if (Test-Path $Stage) { Remove-Item -Recurse -Force $Stage }
New-Item -ItemType Directory -Force -Path $Dest | Out-Null
Copy-Item $Nro (Join-Path $Dest "libhide_replay_tips.nro")
if (Test-Path $Zip) { Remove-Item $Zip }
Compress-Archive -Path (Join-Path $Stage "*") -DestinationPath $Zip
Write-Host "wrote $Zip"
