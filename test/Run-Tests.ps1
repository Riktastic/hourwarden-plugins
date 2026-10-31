# Runs the unit tests of every plugin (test\<plugin>.tests.ps1) and of the
# shared protocol helpers (test\protocol.tests.ps1), each in its
# own PowerShell process so plugins cannot see each other's functions.
#
#   powershell -ExecutionPolicy Bypass -File test\Run-Tests.ps1
#   powershell -ExecutionPolicy Bypass -File test\Run-Tests.ps1 -Plugin networks

param([string]$Plugin = '*')

$root = Split-Path $PSScriptRoot -Parent
# Plugins load their own copy of the protocol helpers; keep them current
# (build-index.ps1 does the same before publishing).
$shared = Join-Path $root 'shared\protocol.ps1'
foreach ($dir in Get-ChildItem (Join-Path $root 'plugins') -Directory) {
    if (Select-String -Path (Join-Path $dir.FullName '*.ps1') -Pattern 'protocol\.ps1' -Quiet) {
        Copy-Item $shared (Join-Path $dir.FullName 'protocol.ps1') -Force
    }
}

$powershell = Join-Path $env:SystemRoot 'System32\WindowsPowerShell\v1.0\powershell.exe'
$failed = @()
$files = @(Get-ChildItem $PSScriptRoot -Filter "$Plugin.tests.ps1" | Sort-Object Name)
foreach ($file in $files) {
    Write-Host $file.BaseName.Replace('.tests', '')
    & $powershell -NoProfile -NonInteractive -ExecutionPolicy Bypass -File $file.FullName
    if ($LASTEXITCODE -ne 0) { $failed += $file.Name }
}

$untested = @(Get-ChildItem (Join-Path $root 'plugins') -Directory | Where-Object { -not (Test-Path (Join-Path $PSScriptRoot "$($_.Name).tests.ps1")) })
if ($Plugin -eq '*' -and $untested.Count) { Write-Host "Plugins without tests: $($untested.Name -join ', ')" -ForegroundColor Yellow; $failed += 'missing tests' }

# Every plugin documents itself: plugin.toml names a Markdown file (shown in
# the app's help) that exists, starts with a title and is plain ASCII.
foreach ($dir in Get-ChildItem (Join-Path $root 'plugins') -Directory -Filter $Plugin) {
    $manifest = Get-Content (Join-Path $dir.FullName 'plugin.toml') -Raw
    $docs = if ($manifest -match '(?m)^docs\s*=\s*"([^"]+)"') { Join-Path $dir.FullName $Matches[1] } else { $null }
    $problem = if (-not $docs) { 'no docs in plugin.toml' }
        elseif (-not (Test-Path $docs)) { "$docs is missing" }
        elseif ((Get-Content $docs -TotalCount 1) -notmatch '^# \S') { "$docs does not start with a # title" }
        elseif ([IO.File]::ReadAllBytes($docs) | Where-Object { $_ -gt 127 }) { "$docs is not plain ASCII" }
    if ($problem) { Write-Host "$($dir.Name): $problem" -ForegroundColor Yellow; $failed += "$($dir.Name) docs" }
}
if ($failed.Count) { Write-Host "Failed: $($failed -join ', ')" -ForegroundColor Red; exit 1 }
Write-Host "All tests passed ($($files.Count) test files: the plugins and their shared protocol helpers)" -ForegroundColor Green
