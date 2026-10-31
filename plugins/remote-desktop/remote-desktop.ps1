# Remote Desktop Manager and Remote Desktop Connection: entry, folder, server.
#
# RDM: the selected tab's name (UI Automation) is the entry; folder and host
# come from the Devolutions.PowerShell module when available. Without the
# module, the server comes from the RDP connection the process holds open.
# Remote Desktop Connection (mstsc): the server is the first part of the title.

. "$PSScriptRoot/protocol.ps1"

Add-Type -AssemblyName UIAutomationClient, UIAutomationTypes

$rdmExes = @('remotedesktopmanager.exe', 'remotedesktopmanager64.exe', 'remotedesktopmanagerfree.exe')
$ignoredTabs = '^(Dashboard|Home|Start|Overview|Welcome|Navigation)$'
$script:entries = @{}
$script:entriesAt = [DateTime]::MinValue
$script:moduleState = 'unknown'

function Update-RdmEntries {
    if (-not (Get-Setting 'use_rdm_module' $true)) { $script:moduleState = 'off'; return }
    $minutes = [int](Get-Setting 'refresh_minutes' 15)
    if (([DateTime]::UtcNow - $script:entriesAt).TotalMinutes -lt $minutes) { return }
    $script:entriesAt = [DateTime]::UtcNow
    if (-not (Get-Module -ListAvailable -Name Devolutions.PowerShell)) { $script:moduleState = 'missing'; return }
    try {
        Import-Module Devolutions.PowerShell -ErrorAction Stop
        $list = if (Get-Command Get-RDMEntry -ErrorAction SilentlyContinue) { Get-RDMEntry } else { Get-RDMSession }
        $map = @{}
        foreach ($e in $list) {
            $map[[string]$e.Name] = @{ folder = [string]$e.Group; host = [string]$e.Host }
        }
        $script:entries = $map
        $script:moduleState = 'ok'
    } catch {
        $script:moduleState = "error: $($_.Exception.Message)"
    }
}

function Get-SelectedTab([long]$hwnd) {
    $root = [System.Windows.Automation.AutomationElement]::FromHandle([IntPtr]$hwnd)
    $condition = New-Object System.Windows.Automation.PropertyCondition(
        [System.Windows.Automation.AutomationElement]::ControlTypeProperty,
        [System.Windows.Automation.ControlType]::TabItem)
    foreach ($tab in $root.FindAll([System.Windows.Automation.TreeScope]::Descendants, $condition)) {
        try {
            $pattern = $tab.GetCurrentPattern([System.Windows.Automation.SelectionItemPattern]::Pattern)
            $name = $tab.Current.Name.Trim()
            if ($pattern.Current.IsSelected -and $name -and $name -notmatch $ignoredTabs) { return $name }
        } catch { }
    }
    return $null
}

# A tab name may carry the folder: "Contoso\Production\APP01".
function Split-RdmEntry([string]$Entry) {
    $i = $Entry.LastIndexOf('\')
    if ($i -lt 0) { return @{ folder = $null; name = $Entry } }
    return @{ folder = $Entry.Substring(0, $i); name = $Entry.Substring($i + 1) }
}

# Remote Desktop Connection titles start with the server: "app01.contoso.local - Remote Desktop Connection".
function Get-MstscServer([string]$Title) {
    $server = ($Title -split ' - ')[0].Trim()
    if (-not $server -or $server -match '^Remote Desktop Connection$|^Verbinding met extern bureaublad$') { return $null }
    return $server
}

function Get-RdpServer([int]$processId) {
    $connections = @(Get-NetTCPConnection -OwningProcess $processId -State Established -RemotePort 3389 -ErrorAction SilentlyContinue)
    if ($connections.Count -ne 1) { return $null }
    $address = $connections[0].RemoteAddress
    try { return [System.Net.Dns]::GetHostEntry($address).HostName } catch { return $address }
}

if ($script:testMode) { return }
Wait-Initialize
Send-Status 'running' 'Waiting for a remote desktop window'
$lastKey = $null
$lastSent = [DateTime]::MinValue

while ($true) {
    [void](Receive-Messages)
    $fg = $script:foreground
    $detail = $null
    try {
        if ($fg -and $rdmExes -contains $fg.exe) {
            Update-RdmEntries
            $entry = Get-SelectedTab $fg.hwnd
            if ($entry) {
                $parts = Split-RdmEntry $entry
                $folder = $parts.folder
                $name = $parts.name
                $known = $script:entries[$name]
                if ($known) {
                    if ($known.folder) { $folder = $known.folder }
                    $server = $known.host
                } else {
                    $server = Get-RdpServer $fg.pid
                }
                $detail = @{ hwnd = $fg.hwnd; exe = $fg.exe; document = $name; remote_folder = $folder; remote_server = $server }
            }
            switch -Wildcard ($script:moduleState) {
                'ok' { Send-Status 'running' "Reading Remote Desktop Manager ($($script:entries.Count) entries)" }
                'missing' { Send-Status 'degraded' 'Devolutions.PowerShell is not installed: folders only come from tab names' }
                'error*' { Send-Status 'degraded' "Devolutions.PowerShell could not read entries ($($script:moduleState))" }
                default { Send-Status 'running' 'Reading Remote Desktop Manager tabs' }
            }
        } elseif ($fg -and $fg.exe -eq 'mstsc.exe' -and $fg.title) {
            $server = Get-MstscServer $fg.title
            if ($server) { $detail = @{ hwnd = $fg.hwnd; exe = $fg.exe; document = $server; remote_server = $server } }
        }
    } catch {
        Send-Status 'degraded' "Could not read the session: $($_.Exception.Message)"
    }
    if ($detail) {
        $key = "$($detail.hwnd)|$($detail.document)|$($detail.remote_folder)|$($detail.remote_server)"
        if ($key -ne $lastKey -or ([DateTime]::UtcNow - $lastSent).TotalSeconds -ge 15) {
            Send-Message 'detail' $detail
            $lastKey = $key
            $lastSent = [DateTime]::UtcNow
        }
    }
    Wait-ForHost ([int](Get-Setting 'poll_seconds' 3) * 1000)
}
