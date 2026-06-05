param(
    [ValidateSet("", "scoped", "unscoped")]
    [string]$PackageNameMode = ""
)

$pkgPath = ".\pkg\package.json"
$templatePath = ".\templates\package.template.json"
$scopePrefix = "@laisuk/"

$pkg = Get-Content $pkgPath -Raw | ConvertFrom-Json
$template = Get-Content $templatePath -Raw | ConvertFrom-Json

# wasm-pack owns the generated package version/name source.
# This script owns the final package.json metadata policy.
$name = [string]$pkg.name
$version = $pkg.version

# Normalize to the unscoped base name first so repeated runs are idempotent.
if ( $name.StartsWith($scopePrefix))
{
    $baseName = $name.Substring($scopePrefix.Length)
}
else
{
    $baseName = $name
}

switch ($PackageNameMode)
{
    "scoped" {
        $name = "$scopePrefix$baseName"
    }
    "unscoped" {
        $name = $baseName
    }
    default {
        # Preserve current wasm-pack behavior unless the caller explicitly asks
        # for scoped/unscoped package naming.
        $name = $pkg.name
    }
}

$out = $template | ConvertTo-Json -Depth 20 | ConvertFrom-Json

$out.name = $name
$out.version = $version

$out | ConvertTo-Json -Depth 20 | Set-Content $pkgPath -Encoding UTF8

Write-Host "Updated $pkgPath"
Write-Host "Package name: $name"
Write-Host "Preserved version: $version"
