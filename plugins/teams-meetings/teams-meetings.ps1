# Teams meetings: a meeting runs while Teams holds the microphone.
#
# Windows records microphone use per app under the privacy registry key below.
# New Teams is a packaged app (MSTeams_8wekyb3d8bbwe) directly under the key;
# classic Teams appears under NonPackaged. LastUsedTimeStop = 0 means in use.

. "$PSScriptRoot\protocol.ps1"

Add-Type -TypeDefinition @"
using System;
using System.Collections.Generic;
using System.Runtime.InteropServices;
using System.Text;
public static class TkWindows {
    delegate bool EnumProc(IntPtr hwnd, IntPtr data);
    [DllImport("user32.dll")] static extern bool EnumWindows(EnumProc f, IntPtr data);
    [DllImport("user32.dll")] static extern bool IsWindowVisible(IntPtr hwnd);
    [DllImport("user32.dll")] static extern uint GetWindowThreadProcessId(IntPtr hwnd, out uint pid);
    [DllImport("user32.dll", CharSet = CharSet.Unicode)] static extern int GetWindowText(IntPtr hwnd, StringBuilder text, int max);
    public static List<string> Titles(uint[] pids) {
        var titles = new List<string>();
        var wanted = new HashSet<uint>(pids);
        EnumWindows((h, d) => {
            uint pid;
            GetWindowThreadProcessId(h, out pid);
            if (wanted.Contains(pid) && IsWindowVisible(h)) {
                var sb = new StringBuilder(512);
                GetWindowText(h, sb, 512);
                if (sb.Length > 0) titles.Add(sb.ToString());
            }
            return true;
        }, IntPtr.Zero);
        return titles;
    }
}
"@

$micKey = 'HKCU:\Software\Microsoft\Windows\CurrentVersion\CapabilityAccessManager\ConsentStore\microphone'
# Titles of the main Teams window sections; anything else while the mic is on is the meeting.
$hubPattern = '^(Chat|Activity|Calendar|Teams|Calls|Files|Apps|OneDrive|Assignments|Communities|Copilot|Microsoft Teams)\b'

function Test-TeamsMicrophone {
    if (-not (Test-Path $micKey)) { return $null }
    foreach ($key in Get-ChildItem -Path $micKey -Recurse -ErrorAction SilentlyContinue) {
        if ($key.PSChildName -notmatch 'teams') { continue }
        $stop = $key.GetValue('LastUsedTimeStop')
        $start = $key.GetValue('LastUsedTimeStart')
        if ($null -ne $stop -and $stop -eq 0 -and $start -gt 0) { return $true }
    }
    return $false
}

function Get-MeetingTitle {
    $pids = @(Get-Process -Name 'ms-teams', 'Teams' -ErrorAction SilentlyContinue | ForEach-Object { [uint32]$_.Id })
    if (-not $pids.Count) { return $null }
    foreach ($title in [TkWindows]::Titles($pids)) {
        $clean = ($title -replace '\s*\|\s*Microsoft Teams.*$', '').Trim()
        if ($clean -and $clean -notmatch $hubPattern) { return $clean }
    }
    return $null
}

Wait-Initialize
Send-Status 'running' 'Watching the microphone'
$inMeeting = $false

while ($true) {
    [void](Receive-Messages)
    try {
        $mic = Test-TeamsMicrophone
        if ($null -eq $mic) {
            Send-Status 'degraded' 'The microphone privacy registry key was not found'
        } elseif ($mic) {
            $title = Get-MeetingTitle
            if (-not $title -and (Get-Setting 'count_any_call' $true)) { $title = 'Teams call' }
            if ($title) {
                # Repeated while it lasts, so the host knows the meeting is still on.
                Send-Message 'meeting' @{ active = $true; title = $title }
                if (-not $inMeeting) { Send-Log "Meeting started: $title" }
                $inMeeting = $true
            }
        } elseif ($inMeeting) {
            Send-Message 'meeting' @{ active = $false }
            Send-Log 'Meeting ended'
            $inMeeting = $false
        }
    } catch {
        Send-Status 'degraded' "Check failed: $($_.Exception.Message)"
    }
    Wait-ForHost ([int](Get-Setting 'poll_seconds' 5) * 1000)
}
