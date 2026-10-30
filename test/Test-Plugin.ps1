# Runs a plugin the way Timekeeper does and prints what it reports.
#
#   powershell -ExecutionPolicy Bypass -File test\Test-Plugin.ps1 -Plugin networks -Seconds 10
#   ... -Plugin office-paths -Foreground '{"hwnd":0,"pid":0,"exe":"explorer.exe","title":"Documents"}'
#   ... -Plugin file-changes -Folders 'D:\Klanten\Contoso'

param(
    [Parameter(Mandatory)] [string]$Plugin,
    [int]$Seconds = 10,
    [string]$Foreground = '',
    [string[]]$Folders = @(),
    [string]$Settings = '{}'
)

$ErrorActionPreference = 'Stop'
$dir = Join-Path (Split-Path $PSScriptRoot -Parent) "plugins\$Plugin"
$manifest = Get-Content (Join-Path $dir 'plugin.toml') -Raw
$argsLine = [regex]::Match($manifest, '(?m)^args\s*=\s*\[(.*)\]').Groups[1].Value
$arguments = [regex]::Matches($argsLine, '"([^"]*)"') | ForEach-Object { '"' + $_.Groups[1].Value + '"' }

$psi = New-Object System.Diagnostics.ProcessStartInfo
$psi.FileName = "$env:SystemRoot\System32\WindowsPowerShell\v1.0\powershell.exe"
$psi.Arguments = $arguments -join ' '
$psi.WorkingDirectory = $dir
$psi.UseShellExecute = $false
$psi.RedirectStandardInput = $true
$psi.RedirectStandardOutput = $true
$psi.RedirectStandardError = $true
$psi.CreateNoWindow = $true
$process = [System.Diagnostics.Process]::Start($psi)

$init = @{ jsonrpc = '2.0'; method = 'initialize'; params = @{ api = 1; host_pid = $PID; settings = ($Settings | ConvertFrom-Json); customer_folders = $Folders } }
$process.StandardInput.WriteLine(($init | ConvertTo-Json -Compress -Depth 6))
if ($Foreground) {
    $process.StandardInput.WriteLine((@{ jsonrpc = '2.0'; method = 'foreground'; params = ($Foreground | ConvertFrom-Json) } | ConvertTo-Json -Compress -Depth 6))
}
$process.StandardInput.Flush()

$deadline = [DateTime]::UtcNow.AddSeconds($Seconds)
$pending = $process.StandardOutput.ReadLineAsync()
while ([DateTime]::UtcNow -lt $deadline -and -not $process.HasExited) {
    if ($pending.Wait(200)) {
        if ($null -eq $pending.Result) { break }
        Write-Host "<- $($pending.Result)"
        $pending = $process.StandardOutput.ReadLineAsync()
    }
}
$process.StandardInput.WriteLine('{"jsonrpc":"2.0","method":"shutdown"}')
$process.StandardInput.Close()
if (-not $process.WaitForExit(3000)) { $process.Kill(); Write-Host '(killed: did not stop on shutdown)' }
$errors = $process.StandardError.ReadToEnd()
if ($errors.Trim()) { Write-Host "stderr: $errors" }
Write-Host "exit code: $($process.ExitCode)"
