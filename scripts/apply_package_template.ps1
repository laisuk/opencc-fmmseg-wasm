$pkgPath = ".\pkg\package.json"
$templatePath = ".\templates\package.template.json"

$pkg = Get-Content $pkgPath -Raw | ConvertFrom-Json
$template = Get-Content $templatePath -Raw | ConvertFrom-Json

# Preserve wasm-pack generated identity/version
$name = $pkg.name
$version = $pkg.version

# Start from template
$out = $template

$out.name = $name
$out.version = $version

$out | ConvertTo-Json -Depth 20 | Set-Content $pkgPath -Encoding UTF8

Write-Host "Updated $pkgPath"
Write-Host "Preserved name: $name"
Write-Host "Preserved version: $version"