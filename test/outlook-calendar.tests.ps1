. "$PSScriptRoot\TestKit.ps1"
. (Use-Plugin 'outlook-calendar')

# A stand-in for an Outlook appointment.
function New-Appointment([string]$Subject, [DateTime]$Start, [int]$Minutes, [string[]]$Attendees = @(), [hashtable]$Extra = @{}) {
    $list = [pscustomobject]@{ Count = $Attendees.Count; Items = @($Attendees | ForEach-Object { [pscustomobject]@{ Address = $_; AddressEntry = $null } }) }
    $list | Add-Member ScriptMethod Item { param($i) $this.Items[$i - 1] }
    $item = [ordered]@{
        Subject = $Subject; Location = 'Utrecht'; Start = $Start; End = $Start.AddMinutes($Minutes)
        AllDayEvent = $false; BusyStatus = 2; MeetingStatus = 1; GlobalAppointmentID = 'ABC'; Recipients = $list
    }
    foreach ($k in $Extra.Keys) { $item[$k] = $Extra[$k] }
    [pscustomobject]$item
}

$start = [DateTime]::new(2026, 10, 29, 9, 0, 0, [DateTimeKind]::Local)

Test 'an appointment with attendees' {
    $c = ConvertTo-CalendarItem (New-Appointment 'Contoso steering' $start 60 @('me@rik.blue', 'jan@contoso.com', 'piet@Contoso.com')) @('rik.blue')
    Assert-Equal 'Contoso steering' $c.subject
    Assert-Equal @('contoso.com') $c.domains
    Assert-Equal 3600000 ($c.end - $c.start)
    Assert-Equal ([DateTimeOffset]$start).ToUnixTimeMilliseconds() $c.start
    Assert-Equal "ABC|2026-10-29T09:00:00" $c.key
}

Test 'each occurrence of a recurring meeting has its own key' {
    $a = ConvertTo-CalendarItem (New-Appointment 'Standup' $start 15) @()
    $b = ConvertTo-CalendarItem (New-Appointment 'Standup' $start.AddDays(1) 15) @()
    Assert-True ($a.key -ne $b.key)
}

Test 'all-day, free and cancelled appointments are skipped' {
    Assert-Equal $null (ConvertTo-CalendarItem (New-Appointment 'Holiday' $start 1440 @() @{ AllDayEvent = $true }) @())
    Assert-Equal $null (ConvertTo-CalendarItem (New-Appointment 'Travel' $start 30 @() @{ BusyStatus = 0 }) @())
    Assert-Equal $null (ConvertTo-CalendarItem (New-Appointment 'Cancelled' $start 30 @() @{ MeetingStatus = 5 }) @())
}

Test 'free appointments are kept when asked' {
    $script:settings = [pscustomobject]@{ skip_free = $false }
    Assert-True (ConvertTo-CalendarItem (New-Appointment 'Travel' $start 30 @() @{ BusyStatus = 0 }) @())
    $script:settings = $null
}

Test 'attendees beyond the maximum are not read' {
    $c = ConvertTo-CalendarItem (New-Appointment 'All hands' $start 60 @('a@contoso.com', 'b@fabrikam.nl')) @() 1
    Assert-Equal @('contoso.com') $c.domains
}

Complete-Tests
