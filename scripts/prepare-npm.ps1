param(
    [ValidateSet("", "scoped", "unscoped")]
    [string]$PackageNameMode = "",

    [switch]$Ruzstd,

    [switch]$NoModules
)

$wasmTarget = if ($NoModules) { "no-modules" } else { "web" }
$outDir = if ($NoModules) { "pkg-no-modules" } else { "pkg" }

$wasmPackArgs = @(
    "build",
    "--target", $wasmTarget,
    "--out-dir", $outDir,
    "--release"
)

if ($Ruzstd) {
    $wasmPackArgs += @("--", "--features", "ruzstd")
    Write-Host "Building with pure-Rust ZSTD dictionary decoding (ruzstd)." -ForegroundColor Cyan
} else {
    Write-Host "Building with CBOR dictionaries." -ForegroundColor Cyan
}

wasm-pack @wasmPackArgs

if ($LASTEXITCODE -ne 0) {
    exit $LASTEXITCODE
}

if ($NoModules) {
    Write-Host "Standalone WASM build completed: $outDir" -ForegroundColor Green
    return
}

$src = ".\bin\opencc.js"
$dst = ".\pkg\bin\opencc.js"

New-Item -ItemType Directory -Force ".\pkg\bin" | Out-Null

if (!(Test-Path $dst) -or ((Get-Item $src).LastWriteTime -gt (Get-Item $dst).LastWriteTime))
{
    Copy-Item $src $dst -Force
    Write-Host "Updated pkg/bin/opencc.js" -ForegroundColor Green
}
else
{
    Write-Host "No bin update needed." -ForegroundColor Blue
}

.\scripts\apply_package_template.ps1 $PackageNameMode