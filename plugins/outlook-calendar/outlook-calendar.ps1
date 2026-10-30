# Outlook calendar: your appointments of the last days and today, so the
# daily review can add meetings you attended away from the computer.
#
# Uses the running Outlook (classic) through COM. Reads the time, subject,
# location and the attendees' mail domains; never the body.

. "$PSScriptRoot\protocol.ps1"

$olFolderCalendar = 9
$olFree = 0
$olMeetingCanceled = 5
$olMeetingReceivedAndCanceled = 7
$olExchangeUser = 0
$olExchangeRemoteUser = 5

function Get-Outlook {
    try { return [Runtime.InteropServices.Marshal]::GetActiveObject('Outlook.Application') } catch { return $null }
}

function Get-Domain([string]$Address) {
    if ($Address -and $Address -match '@([A-Za-z0-9.-]+\.[A-Za-z]{2,})$') { return $Matches[1].ToLower() }
    return $null
}

function Get-Smtp($AddressEntry, [string]$Fallback) {
    try {
        if ($AddressEntry -and ($AddressEntry.AddressEntryUserType -eq $olExchangeUser -or $AddressEntry.AddressEntryUserType -eq $olExchangeRemoteUser)) {
            $user = $AddressEntry.GetExchangeUser()
            if ($user) { return $user.PrimarySmtpAddress }
        }
    } catch { }
    return $Fallback
}

function ConvertTo-UnixMs([DateTime]$Time) {
    return ([DateTimeOffset]$Time).ToUnixTimeMilliseconds()
}

# One appointment as the host wants it, or $null for what is not time
# spent: all-day items, free time and cancelled meetings.
function ConvertTo-CalendarItem($Item, $Ignore, [int]$MaxAttendees = 50) {
    if ($Item.AllDayEvent) { return $null }
    if ($Item.BusyStatus -eq $olFree -and (Get-Setting 'skip_free' $true)) { return $null }
    if ($Item.MeetingStatus -eq $olMeetingCanceled -or $Item.MeetingStatus -eq $olMeetingReceivedAndCanceled) { return $null }
    $Ignore = @($Ignore | ForEach-Object { $_.ToLower().TrimStart('@') })
    $addresses = New-Object System.Collections.Generic.List[string]
    try {
        $count = [Math]::Min([int]$Item.Recipients.Count, $MaxAttendees)
        for ($i = 1; $i -le $count; $i++) {
            $r = $Item.Recipients.Item($i)
            $addresses.Add((Get-Smtp $r.AddressEntry $r.Address))
        }
    } catch { }
    $domains = @($addresses | ForEach-Object { Get-Domain $_ } | Where-Object { $_ -and $Ignore -notcontains $_ } | Sort-Object -Unique)
    $start = [DateTime]$Item.Start
    return @{
        # Recurring meetings share their ID; the start makes each one unique.
        key      = "$($Item.GlobalAppointmentID)|$($start.ToString('s'))"
        start    = ConvertTo-UnixMs $start
        end      = ConvertTo-UnixMs ([DateTime]$Item.End)
        subject  = [string]$Item.Subject
        location = [string]$Item.Location
        domains  = $domains
    }
}

function Get-Appointments($Outlook, [DateTime]$From, [DateTime]$To) {
    $items = $Outlook.GetNamespace('MAPI').GetDefaultFolder($olFolderCalendar).Items
    $items.Sort('[Start]')
    $items.IncludeRecurrences = $true
    # Outlook wants dates in the format of the current culture here.
    $filter = "[Start] < '$($To.ToString('g'))' AND [End] > '$($From.ToString('g'))'"
    $ignore = @(Get-Setting 'ignore_domains' @())
    $max = [int](Get-Setting 'max_attendees' 50)
    $out = @()
    foreach ($item in $items.Restrict($filter)) {
        if ($out.Count -ge 500) { break }
        $converted = ConvertTo-CalendarItem $item $ignore $max
        if ($converted) { $out += $converted }
    }
    return , $out
}

if ($script:testMode) { return }
Wait-Initialize
Send-Status 'running' 'Waiting for Outlook'
$lastRead = [DateTime]::MinValue

while ($true) {
    [void](Receive-Messages)
    if (([DateTime]::UtcNow - $lastRead).TotalMinutes -ge [int](Get-Setting 'refresh_minutes' 15)) {
        $outlook = Get-Outlook
        if ($null -eq $outlook) {
            Send-Status 'degraded' 'Outlook is not running (or runs as administrator); appointments are read when it is open'
        } else {
            try {
                $today = (Get-Date).Date
                $items = Get-Appointments $outlook $today.AddDays(-[int](Get-Setting 'days_back' 7)) $today.AddDays(1)
                Send-Message 'calendar' @{ items = $items }
                Send-Status 'running' "Read $($items.Count) appointments"
                $lastRead = [DateTime]::UtcNow
            } catch {
                Send-Status 'degraded' "The calendar could not be read: $($_.Exception.Message)"
            }
        }
    }
    Wait-ForHost 5000
}
