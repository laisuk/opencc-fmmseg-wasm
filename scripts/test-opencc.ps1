# test-opencc.ps1
# Local npm smoke test for opencc-fmmseg-wasm

$ErrorActionPreference = "Stop"

$ProjectName = "test-opencc"

# Detect caller current directory
$BaseDir = (Get-Location).Path

Write-Host "BaseDir: $BaseDir"

$ProjectPath = Join-Path $BaseDir $ProjectName

Write-Host "Creating project at:"
Write-Host $ProjectPath

# Create project folder
New-Item -ItemType Directory -Force -Path $ProjectPath | Out-Null

# Enter project folder
Set-Location $ProjectPath

# Init npm
npm init -y

# Install package
npm install opencc-fmmseg-wasm

# Test CLI
npx opencc-fmmseg -h

Write-Host ""
Write-Host "Done."
