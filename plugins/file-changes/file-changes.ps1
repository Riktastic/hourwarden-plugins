# Saved files in customer folders.
#
# One FileSystemWatcher per customer folder (subfolders included). Hidden
# folders, temporary files and bursts (sync clients, git checkouts) are
# ignored. The host decides whether the change belongs to the user's work.

. "$PSScriptRoot\protocol.ps1"

$script:watchers = @()
$script:watched = @()
$script:recent = @{}
$script:burst = New-Object System.Collections.Generic.Queue[DateTime]
$script:quietUntil = [DateTime]::MinValue

function Reset-Watchers {
    foreach ($w in $script:watchers) {
        Get-EventSubscriber | Where-Object SourceObject -eq $w | Unregister-Event
        $w.EnableRaisingEvents = $false
        $w.Dispose()
    }
    $script:watchers = @()
    $script:watched = @($script:folders | Where-Object { $_ -and (Test-Path -LiteralPath $_ -PathType Container) })
    foreach ($folder in $script:watched) {
        $w = New-Object System.IO.FileSystemWatcher $folder
        $w.IncludeSubdirectories = $true
        $w.NotifyFilter = [System.IO.NotifyFilters]'FileName, LastWrite'
        foreach ($name in 'Changed', 'Created', 'Renamed') {
            Register-ObjectEvent -InputObject $w -EventName $name -SourceIdentifier "tk-$([guid]::NewGuid())" | Out-Null
        }
        $w.EnableRaisingEvents = $true
        $script:watchers += $w
    }
    $missing = @($script:folders).Count - $script:watched.Count
    if (-not $script:watched.Count) {
        Send-Status 'degraded' 'No customer folders to watch; add folders on the customer pages'
    } elseif ($missing -gt 0) {
        Send-Status 'degraded' "Watching $($script:watched.Count) folders; $missing do not exist"
    } else {
        Send-Status 'running' "Watching $($script:watched.Count) customer folders"
    }
}

function Test-Ignored([string]$path) {
    if ($path -match '\\\.[^\\]+\\' -or $path -match '\\(node_modules|\.git)\\') { return $true }
    $name = Split-Path -Leaf $path
    $defaults = @('~$*', '*.tmp', '*.temp', '*.lock', '*.crdownload', '*.partial')
    foreach ($pattern in @(Get-Setting 'ignore_patterns' $defaults)) {
        if ($name -like $pattern) { return $true }
    }
    if (Test-Path -LiteralPath $path -PathType Container) { return $true }
    return $false
}

function Test-Burst([DateTime]$Now = [DateTime]::UtcNow) {
    $now = $Now
    $script:burst.Enqueue($now)
    while ($script:burst.Count -and ($now - $script:burst.Peek()).TotalSeconds -gt 5) { [void]$script:burst.Dequeue() }
    if ($script:burst.Count -gt [int](Get-Setting 'max_burst' 20)) {
        $script:quietUntil = $now.AddSeconds(10)
    }
    return $now -lt $script:quietUntil
}

if ($script:testMode) { return }
Wait-Initialize
Reset-Watchers
$knownFolders = ($script:folders -join '|')
$pending = [ordered]@{}

while ($true) {
    [void](Receive-Messages)
    if (($script:folders -join '|') -ne $knownFolders) {
        $knownFolders = ($script:folders -join '|')
        Reset-Watchers
    }
    $now = [DateTime]::UtcNow
    foreach ($event in @(Get-Event -ErrorAction SilentlyContinue)) {
        $path = $event.SourceEventArgs.FullPath
        Remove-Event -EventIdentifier $event.EventIdentifier
        if (-not $path -or (Test-Ignored $path)) { continue }
        if ($pending.Contains($path)) { continue }
        # Count each file once towards a burst (a save gives several events).
        [void](Test-Burst)
        $pending[$path] = @{ at = $now; change = [string]$event.SourceEventArgs.ChangeType }
    }
    # Changes are held for two seconds; a burst in that time drops them all.
    if ($now -lt $script:quietUntil) { $pending.Clear() }
    foreach ($path in @($pending.Keys)) {
        $item = $pending[$path]
        if (($now - $item.at).TotalSeconds -lt 2) { continue }
        $pending.Remove($path)
        # The same file is reported at most every 10 seconds.
        $last = $script:recent[$path]
        if ($last -and ($now - $last).TotalSeconds -lt 10) { continue }
        $script:recent[$path] = $now
        Send-Message 'file_changed' @{ path = $path; change = $item.change }
    }
    if ($script:recent.Count -gt 500) { $script:recent.Clear() }
    Wait-ForHost 500
}
