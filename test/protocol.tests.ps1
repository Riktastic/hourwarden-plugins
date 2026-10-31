# Unit tests of the protocol helpers every plugin shares (shared/protocol.ps1).
. "$PSScriptRoot/TestKit.ps1"
. (Join-Path (Split-Path $PSScriptRoot -Parent) 'shared\protocol.ps1')

# Feeds lines to Receive-Messages as if the host wrote them; after the last
# one, reading waits (as with a host that has nothing to say).
function Use-HostLines([string[]]$Lines) {
    $queue = New-Object System.Collections.Queue
    foreach ($l in $Lines) { $queue.Enqueue($l) }
    $script:in = [pscustomobject]@{ Queue = $queue }
    $script:in | Add-Member -MemberType ScriptMethod -Name ReadLineAsync -Value {
        if ($this.Queue.Count) { return [System.Threading.Tasks.Task]::FromResult([string]$this.Queue.Dequeue()) }
        return (New-Object 'System.Threading.Tasks.TaskCompletionSource[string]').Task
    }
    $script:pendingLine = $script:in.ReadLineAsync()
}

Test 'status and log messages are sent' {
    Send-Status 'degraded' 'Outlook is not running'
    Send-Log 'checked 3 folders'
    Assert-Equal 'status' $script:sent[0].method
    Assert-Equal @{ state = 'degraded'; message = 'Outlook is not running' } $script:sent[0].params
    Assert-Equal @{ message = 'checked 3 folders' } $script:sent[1].params
}

Test 'a status without a message sends an empty one' {
    Send-Status 'running'
    Assert-Equal '' $script:sent[0].params.message
}

Test 'settings fall back to their default' {
    $script:settings = $null
    Assert-Equal 5 (Get-Setting 'poll_seconds' 5)
    $script:settings = [pscustomobject]@{ poll_seconds = 30; empty = $null }
    Assert-Equal 30 (Get-Setting 'poll_seconds' 5)
    Assert-Equal 'x' (Get-Setting 'empty' 'x')
    Assert-Equal 'y' (Get-Setting 'missing' 'y')
}

Test 'host messages update what the plugin knows' {
    $init = @{ jsonrpc = '2.0'; method = 'initialize'; params = @{ host_pid = $PID; settings = @{ poll_seconds = 10 }; customer_folders = @(@{ path = 'D:\Klanten\Contoso' }) } } | ConvertTo-Json -Compress -Depth 5
    Use-HostLines @($init, '', '{"jsonrpc":"2.0","method":"foreground","params":{"title":"Mail - Outlook","app":"olk.exe"}}')
    $messages = Receive-Messages
    Assert-Equal 2 $messages.Count 'the blank line is skipped'
    Assert-Equal $PID $script:hostPid
    Assert-Equal 10 (Get-Setting 'poll_seconds' 5)
    Assert-Equal 'D:\Klanten\Contoso' $script:folders[0].path
    Assert-Equal 'olk.exe' $script:foreground.app

    Use-HostLines @('{"jsonrpc":"2.0","method":"settings","params":{"poll_seconds":60}}', '{"jsonrpc":"2.0","method":"folders","params":{"customer_folders":[]}}')
    [void](Receive-Messages)
    Assert-Equal 60 (Get-Setting 'poll_seconds' 5)
    Assert-Equal 0 @($script:folders).Count
}

Test 'nothing waiting gives no messages' {
    Use-HostLines @()
    Assert-Equal 0 (Receive-Messages).Count
}

Test 'waiting for the host returns early when it writes' {
    Use-HostLines @('{"jsonrpc":"2.0","method":"settings","params":{}}')
    $watch = [Diagnostics.Stopwatch]::StartNew()
    Wait-ForHost 3000
    Assert-True ($watch.ElapsedMilliseconds -lt 1000) "waited $($watch.ElapsedMilliseconds) ms"
}

Test 'initialize is waited for' {
    $script:hostPid = 0
    Use-HostLines @((@{ method = 'initialize'; params = @{ host_pid = $PID; settings = @{}; customer_folders = @() } } | ConvertTo-Json -Compress -Depth 4))
    Wait-Initialize
    Assert-Equal $PID $script:hostPid
}

Test 'program names match with or without .exe' {
    Assert-Equal 'code' (Get-AppName 'Code.exe')
    Assert-Equal 'konsole' (Get-AppName 'konsole')
    Assert-True (Test-App 'firefox' @('msedge.exe', 'firefox.exe')) 'Linux name against a Windows name'
    Assert-True (Test-App 'MSEDGE.EXE' @('msedge')) 'Windows name against a Linux name'
    Assert-True (-not (Test-App 'notepad.exe' @('code.exe', 'konsole')))
    Assert-True (-not (Test-App '' @('code.exe')))
}

Complete-Tests