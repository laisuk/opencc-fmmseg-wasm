param(
    [string]$Package = "opencc-fmmseg-wasm"
)

$ProjectName = "test-opencc"

if (Test-Path $ProjectName) {
    Remove-Item $ProjectName -Recurse -Force
}

New-Item -ItemType Directory $ProjectName | Out-Null

if (Test-Path $Package) {
    $Package = (Resolve-Path $Package).Path
}

Push-Location $ProjectName

try {
    npm init -y
    if ($LASTEXITCODE -ne 0) { throw "npm init failed" }

    npm install $Package
    if ($LASTEXITCODE -ne 0) { throw "npm install failed" }

    npx opencc-fmmseg -h
    if ($LASTEXITCODE -ne 0) { throw "opencc-fmmseg CLI smoke test failed" }

    Write-Host "npm package smoke test passed." -ForegroundColor Green
}
finally {
    Pop-Location
}