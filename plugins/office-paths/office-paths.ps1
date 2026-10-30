# Office and Explorer: the full path of what is in front.
#
# Word, Excel and PowerPoint through COM (ActiveDocument / ActiveWorkbook /
# ActivePresentation); File Explorer through Shell.Application windows.
# Files in OneDrive or SharePoint give their web address.

. "$PSScriptRoot\protocol.ps1"

$apps = @{
    'winword.exe'  = @{ progId = 'Word.Application'; get = { param($a) $a.ActiveDocument.FullName } }
    'excel.exe'    = @{ progId = 'Excel.Application'; get = { param($a) $a.ActiveWorkbook.FullName } }
    'powerpnt.exe' = @{ progId = 'PowerPoint.Application'; get = { param($a) $a.ActivePresentation.FullName } }
}

function Get-OfficePath([string]$exe) {
    $app = $apps[$exe]
    try { $com = [Runtime.InteropServices.Marshal]::GetActiveObject($app.progId) } catch { return $null }
    try { return [string](& $app.get $com) } catch { return $null }
}

function Get-ExplorerPath([long]$hwnd) {
    $shell = New-Object -ComObject Shell.Application
    foreach ($window in $shell.Windows()) {
        try {
            if ([long]$window.HWND -eq $hwnd) { return [string]$window.Document.Folder.Self.Path }
        } catch { }
    }
    return $null
}

Wait-Initialize
Send-Status 'running' 'Waiting for Office or Explorer to be in front'
$lastKey = $null
$lastSent = [DateTime]::MinValue

while ($true) {
    [void](Receive-Messages)
    $fg = $script:foreground
    if ($fg) {
        $path = $null
        try {
            if ($apps.ContainsKey($fg.exe)) { $path = Get-OfficePath $fg.exe }
            elseif ($fg.exe -eq 'explorer.exe') { $path = Get-ExplorerPath $fg.hwnd }
        } catch {
            Send-Status 'degraded' "Could not read the path: $($_.Exception.Message)"
        }
        if ($path) {
            $key = "$($fg.hwnd)|$path"
            if ($key -ne $lastKey -or ([DateTime]::UtcNow - $lastSent).TotalSeconds -ge 15) {
                # An unsaved document ("Document1") has a name but no path yet.
                $isPath = $path.Contains('\') -or $path.Contains('://')
                $detail = if ($isPath) { @{ hwnd = $fg.hwnd; exe = $fg.exe; path = $path } } else { @{ hwnd = $fg.hwnd; exe = $fg.exe; document = $path } }
                Send-Message 'detail' $detail
                Send-Status 'running' 'Reading document paths'
                $lastKey = $key
                $lastSent = [DateTime]::UtcNow
            }
        }
    }
    Wait-ForHost ([int](Get-Setting 'poll_seconds' 3) * 1000)
}
