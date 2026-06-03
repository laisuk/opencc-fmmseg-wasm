$pkgPath = ".\pkg\package.json"
$templatePath = ".\templates\package.template.json"

$pkg = Get-Content $pkgPath -Raw | ConvertFrom-Json
$template = Get-Content $templatePath -Raw | ConvertFrom-Json

$name = $pkg.name
$version = $pkg.version

$out = $template | ConvertTo-Json -Depth 20 | ConvertFrom-Json

$out.name = $name
$out.version = $version

$out | ConvertTo-Json -Depth 20 | Set-Content $pkgPath -Encoding UTF8

Write-Host "Updated $pkgPath"
Write-Host "Preserved name: $name"
Write-Host "Preserved version: $version"