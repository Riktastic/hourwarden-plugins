# A small test kit for plugin unit tests, without dependencies.
#
#   . "$PSScriptRoot\TestKit.ps1"
#   . (Use-Plugin 'networks')          # loads the plugin's functions, not its main loop
#   Test 'picks the Wi-Fi network' { Assert-Equal 'Home' (Select-Network ...).name }
#   Complete-Tests
#
# HOURWARDEN_PLUGIN_TEST makes shared/protocol.ps1 collect sent messages in
# $script:sent instead of writing them, and the plugin return before its loop.

$env:HOURWARDEN_PLUGIN_TEST = '1'
$ErrorActionPreference = 'Stop'
$script:passed = 0
$script:failures = New-Object System.Collections.Generic.List[string]

function Use-Plugin([string]$Id) {
    return Join-Path (Split-Path $PSScriptRoot -Parent) "plugins/$Id/$Id.ps1"
}

# A fresh folder in the temporary folder, on Windows and Linux.
function New-TestFolder([string]$Name) {
    $path = Join-Path ([IO.Path]::GetTempPath()) "$Name-$PID"
    New-Item -ItemType Directory -Force $path | Out-Null
    return $path
}

# Hashtables are compared by key, whatever their order.
function ConvertTo-Comparable($Value) {
    if ($null -eq $Value) { return $null }
    if ($Value -is [System.Collections.IDictionary]) {
        $out = [ordered]@{}
        foreach ($key in @($Value.Keys | Sort-Object)) { $out[[string]$key] = ConvertTo-Comparable $Value[$key] }
        return $out
    }
    if ($Value -is [System.Collections.IEnumerable] -and $Value -isnot [string]) {
        return , @($Value | ForEach-Object { ConvertTo-Comparable $_ })
    }
    return $Value
}

function Assert-Equal($Expected, $Actual, [string]$Because = '') {
    $e = ConvertTo-Json -InputObject (ConvertTo-Comparable $Expected) -Compress -Depth 8
    $a = ConvertTo-Json -InputObject (ConvertTo-Comparable $Actual) -Compress -Depth 8
    if ($e -cne $a) { throw "expected $e but got $a $Because" }
}

function Assert-True($Value, [string]$Because = 'expected true') {
    if (-not $Value) { throw $Because }
}

function Test([string]$Name, [scriptblock]$Body) {
    if ($script:sent) { $script:sent.Clear() }
    try {
        . $Body
        $script:passed++
        Write-Host "  ok    $Name"
    } catch {
        $script:failures.Add($Name)
        Write-Host "  FAIL  $Name`n        $($_.Exception.Message)" -ForegroundColor Red
    }
}

function Complete-Tests {
    Write-Host "  $($script:passed) passed, $($script:failures.Count) failed"
    if ($script:failures.Count) { exit 1 }
    exit 0
}
