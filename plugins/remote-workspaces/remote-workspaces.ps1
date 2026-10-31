# Remote workplaces: the desktop or app you work in through Citrix, Azure
# Virtual Desktop and Windows 365 (Windows App / Remote Desktop client) and
# VMware Horizon. Customers often give you such a workplace; its name
# usually carries the customer's name.
#
# The name is reported as the document, which is matched against customer
# names and aliases like a file name; add an alias if the name differs.

. "$PSScriptRoot/protocol.ps1"

# Program (without .exe) -> title suffixes to drop. Citrix Workspace
# (wfica) and VMware Horizon also run on Linux.
$clients = @{
    'cdviewer'         = ' - Desktop Viewer$'
    'wfica32'          = ' \((Remote|Extern)\)$| - Citrix Workspace$'
    'wfica'            = ' \((Remote|Extern)\)$| - Citrix Workspace$'
    'msrdc'            = ' - (Remote Desktop|Extern bureaublad)$'
    'windows365'       = ' - Windows 365$'
    'windowsapp'       = ' - Windows App$'
    'vmware-view'      = ' - VMware Horizon( Client)?$'
    'vmware-remotemks' = ' - VMware Horizon( Client)?$'
}
$clientWindows = '^(Citrix Workspace|Windows App|Windows 365|Remote Desktop|VMware Horizon( Client)?|Microsoft Remote Desktop)$'

# The workplace name from a client's window title, or $null for the
# client's own windows (its start screen or settings).
function Get-WorkspaceName([string]$Exe, [string]$Title) {
    $suffix = $clients[(Get-AppName $Exe)]
    if ($null -eq $suffix -or -not $Title) { return $null }
    $name = ($Title -replace $suffix, '').Trim()
    if (-not $name -or $name -match $clientWindows) { return $null }
    return $name
}

if ($script:testMode) { return }
Wait-Initialize
Send-Status 'running' 'Waiting for a remote workplace to be in front'
$lastKey = $null
$lastSent = [DateTime]::MinValue

while ($true) {
    [void](Receive-Messages)
    $fg = $script:foreground
    if ($fg) {
        $name = Get-WorkspaceName $fg.exe $fg.title
        if ($name) {
            $key = "$($fg.hwnd)|$name"
            if ($key -ne $lastKey -or ([DateTime]::UtcNow - $lastSent).TotalSeconds -ge 15) {
                Send-Message 'detail' @{ hwnd = $fg.hwnd; exe = $fg.exe; document = $name; remote_folder = $name }
                if ($key -ne $lastKey) { Send-Log "Remote workplace: $name" }
                $lastKey = $key
                $lastSent = [DateTime]::UtcNow
            }
        }
    }
    Wait-ForHost ([int](Get-Setting 'poll_seconds' 2) * 1000)
}
