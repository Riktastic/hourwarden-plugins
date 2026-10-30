# Builds index.json: copies shared/protocol.ps1 into every plugin that uses
# it, then records each plugin's id, version and SHA-256 per file. Run it
# after changing a plugin; the app refuses files that do not match.
#
# When the signing key exists, index.json is signed (index.json.sig) with
# timekeeper-sign from the Timekeeper repository next to this one. Make a
# key once with:
#   cargo run -q --manifest-path ..\Timekeeper\Cargo.toml -p timekeeper-core --bin timekeeper-sign -- keygen "$env:USERPROFILE\.timekeeper\store-signing.key"
#
#   powershell -ExecutionPolicy Bypass -File build-index.ps1

param(
    [string]$SigningKey = (Join-Path $env:USERPROFILE '.timekeeper\store-signing.key')
)

$ErrorActionPreference = 'Stop'
$root = $PSScriptRoot
$shared = Join-Path $root 'shared\protocol.ps1'
$entries = @()

foreach ($dir in Get-ChildItem (Join-Path $root 'plugins') -Directory | Sort-Object Name) {
    $manifestPath = Join-Path $dir.FullName 'plugin.toml'
    if (-not (Test-Path $manifestPath)) { Write-Warning "$($dir.Name): no plugin.toml, skipped"; continue }
    $manifest = Get-Content $manifestPath -Raw
    $id = [regex]::Match($manifest, '(?m)^id\s*=\s*"([^"]+)"').Groups[1].Value
    $version = [regex]::Match($manifest, '(?m)^version\s*=\s*"([^"]+)"').Groups[1].Value
    if ($id -ne $dir.Name) { throw "$($dir.Name): id '$id' must equal the folder name" }
    if (-not $version) { throw "$($dir.Name): no version" }

    if (Select-String -Path (Join-Path $dir.FullName '*.ps1') -Pattern 'protocol\.ps1' -Quiet) {
        Copy-Item $shared (Join-Path $dir.FullName 'protocol.ps1') -Force
    }

    $files = [ordered]@{}
    foreach ($file in Get-ChildItem $dir.FullName -File -Recurse | Sort-Object FullName) {
        $relative = $file.FullName.Substring($dir.FullName.Length + 1).Replace('\', '/')
        $files[$relative] = (Get-FileHash -Algorithm SHA256 -LiteralPath $file.FullName).Hash.ToLower()
    }
    $entries += [ordered]@{ id = $id; version = $version; path = "plugins/$($dir.Name)"; files = $files }
    Write-Host "$id $version ($($files.Count) files)"
}

$index = [ordered]@{ name = 'Timekeeper plugins'; api = 1; plugins = $entries }
$json = $index | ConvertTo-Json -Depth 6
[System.IO.File]::WriteAllText((Join-Path $root 'index.json'), $json + "`n", (New-Object System.Text.UTF8Encoding($false)))
Write-Host "index.json written with $($entries.Count) plugins"

$signature = Join-Path $root 'index.json.sig'
if (Test-Path $SigningKey) {
    $manifest = Join-Path $root '..\Timekeeper\Cargo.toml'
    & cargo run -q --manifest-path $manifest -p timekeeper-core --bin timekeeper-sign -- sign $SigningKey (Join-Path $root 'index.json')
    if ($LASTEXITCODE -ne 0) { throw 'signing failed' }
} else {
    if (Test-Path $signature) { Remove-Item $signature }
    Write-Warning "Not signed: no key at $SigningKey. Timekeeper accepts this store only while no plugin store key is set."
}
