# Timekeeper plugin protocol helpers (plugin API 1).
#
# The host starts the plugin with stdin/stdout piped. Every message is one
# JSON-RPC 2.0 notification per line: {"jsonrpc":"2.0","method":...,"params":...}.
# See docs/protocol.md. This file is copied into every plugin folder by
# build-index.ps1; edit it here, in shared/.

$ErrorActionPreference = 'Stop'
$script:hostPid = 0
$script:settings = $null
$script:foreground = $null
$script:folders = @()

# Unit tests set TIMEKEEPER_PLUGIN_TEST=1 and dot-source the plugin: nothing
# is read from stdin, sent messages are collected in $script:sent, and the
# plugin returns before its main loop (see test/Run-Tests.ps1).
$script:testMode = [bool]$env:TIMEKEEPER_PLUGIN_TEST
if ($script:testMode) {
    $script:sent = New-Object System.Collections.ArrayList
} else {
    $utf8 = New-Object System.Text.UTF8Encoding($false)
    $script:out = New-Object System.IO.StreamWriter([Console]::OpenStandardOutput(), $utf8)
    $script:out.AutoFlush = $true
    $script:in = New-Object System.IO.StreamReader([Console]::OpenStandardInput(), $utf8)
    $script:pendingLine = $script:in.ReadLineAsync()
}

function Send-Message([string]$Method, $Params) {
    if ($script:testMode) { [void]$script:sent.Add(@{ method = $Method; params = $Params }); return }
    $message = [ordered]@{ jsonrpc = '2.0'; method = $Method; params = $Params }
    $script:out.WriteLine(($message | ConvertTo-Json -Compress -Depth 8))
}

# state: running, degraded or failing. The message is shown in the plugin store.
function Send-Status([string]$State, [string]$Message = '') {
    Send-Message 'status' @{ state = $State; message = $Message }
}

function Send-Log([string]$Message) {
    Send-Message 'log' @{ message = $Message }
}

# Reads what the host sent since the last call and keeps the latest
# settings, foreground window and customer folders in script variables.
# Returns the messages; exits when the host stops or is gone.
function Receive-Messages {
    $messages = @()
    while ($script:pendingLine.IsCompleted) {
        $line = $script:pendingLine.Result
        if ($null -eq $line) { exit 0 }
        $script:pendingLine = $script:in.ReadLineAsync()
        if (-not $line.Trim()) { continue }
        $message = $line | ConvertFrom-Json
        switch ($message.method) {
            'shutdown' { exit 0 }
            'initialize' {
                $script:hostPid = [int]$message.params.host_pid
                $script:settings = $message.params.settings
                $script:folders = @($message.params.customer_folders)
            }
            'settings' { $script:settings = $message.params }
            'foreground' { $script:foreground = $message.params }
            'folders' { $script:folders = @($message.params.customer_folders) }
        }
        $messages += $message
    }
    if ($script:hostPid -and -not (Get-Process -Id $script:hostPid -ErrorAction SilentlyContinue)) { exit 0 }
    return , $messages
}

# Waits up to $Milliseconds, returning early when the host sends something.
function Wait-ForHost([int]$Milliseconds) {
    $deadline = [DateTime]::UtcNow.AddMilliseconds($Milliseconds)
    while ([DateTime]::UtcNow -lt $deadline -and -not $script:pendingLine.IsCompleted) {
        Start-Sleep -Milliseconds 100
    }
}

# A setting from the host, or the default when it is missing.
function Get-Setting([string]$Key, $Default) {
    if ($null -ne $script:settings -and $null -ne $script:settings.$Key) { return $script:settings.$Key }
    return $Default
}

# True on Windows. Windows PowerShell 5.1 has no $IsWindows, so this reads
# the OS environment variable that Windows always sets.
$script:onWindows = $env:OS -eq 'Windows_NT'

# A program name in lower case without ".exe": "Code.exe" on Windows and
# "code" on Linux are the same program.
function Get-AppName([string]$Exe) {
    return ($Exe.Trim().ToLower() -replace '\.exe$', '')
}

# Whether a program is one of the names in a list, with or without ".exe".
function Test-App([string]$Exe, $Names) {
    if (-not $Exe) { return $false }
    $name = Get-AppName $Exe
    foreach ($n in @($Names)) { if ($n -and (Get-AppName ([string]$n)) -eq $name) { return $true } }
    return $false
}

# Waits for the initialize message before the plugin starts working.
function Wait-Initialize {
    $deadline = [DateTime]::UtcNow.AddSeconds(30)
    while (-not $script:hostPid) {
        if ([DateTime]::UtcNow -gt $deadline) { exit 1 }
        Wait-ForHost 500
        [void](Receive-Messages)
    }
}
