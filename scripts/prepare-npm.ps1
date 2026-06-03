wasm-pack build --target web --release

$src = ".\bin\opencc.js"
$dst = ".\pkg\bin\opencc.js"

New-Item -ItemType Directory -Force ".\pkg\bin" | Out-Null

if (!(Test-Path $dst) -or ((Get-Item $src).LastWriteTime -gt (Get-Item $dst).LastWriteTime)) {
    Copy-Item $src $dst -Force
    Write-Host "Updated pkg/bin/opencc.js" -ForegroundColor Green
}
else {
    Write-Host "No bin update needed." -ForegroundColor Blue
}

.\scripts\apply_package_template.ps1