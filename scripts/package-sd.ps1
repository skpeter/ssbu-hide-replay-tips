$ErrorActionPreference = "Stop"
$Root = Split-Path -Parent $PSScriptRoot
$Mod = Join-Path $Root "dist/hide-replay-tips"
$Zip = Join-Path $Root "hide-replay-tips-sd.zip"
if (-not (Test-Path (Join-Path $Mod "info.toml"))) {
    Write-Error "missing $Mod; run python scripts/patch_layouts.py first"
}
$Stage = Join-Path $Root "dist/sd"
$Dest = Join-Path $Stage "ultimate/mods/hide-replay-tips"
if (Test-Path $Stage) { Remove-Item -Recurse -Force $Stage }
New-Item -ItemType Directory -Force -Path (Split-Path $Dest) | Out-Null
Copy-Item -Recurse $Mod $Dest
if (Test-Path $Zip) { Remove-Item $Zip }
Compress-Archive -Path (Join-Path $Stage "*") -DestinationPath $Zip
Write-Host "wrote $Zip"
