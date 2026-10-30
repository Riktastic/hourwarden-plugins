# Outlook (classic): subject and mail domains of the item in front.
#
# Uses the running Outlook through COM. Addresses of Exchange users are
# resolved to their SMTP address. Only metadata is read, never the body.

. "$PSScriptRoot\protocol.ps1"

$olInspector = 35
$olExchangeUser = 0
$olExchangeRemoteUser = 5

function Get-Outlook {
    try { return [Runtime.InteropServices.Marshal]::GetActiveObject('Outlook.Application') } catch { return $null }
}

function Get-Smtp($addressEntry, [string]$fallback) {
    try {
        if ($addressEntry -and ($addressEntry.AddressEntryUserType -eq $olExchangeUser -or $addressEntry.AddressEntryUserType -eq $olExchangeRemoteUser)) {
            $user = $addressEntry.GetExchangeUser()
            if ($user) { return $user.PrimarySmtpAddress }
        }
    } catch { }
    return $fallback
}

function Get-Domain([string]$address) {
    if ($address -and $address -match '@([A-Za-z0-9.-]+\.[A-Za-z]{2,})$') { return $Matches[1].ToLower() }
    return $null
}

function Get-CurrentItem($outlook) {
    $window = $outlook.ActiveWindow()
    if ($null -eq $window) { return $null }
    if ($window.Class -eq $olInspector) { return $window.CurrentItem }
    $selection = $outlook.ActiveExplorer().Selection
    if ($selection.Count -ge 1) { return $selection.Item(1) }
    return $null
}

function Get-ItemInfo($item) {
    $addresses = New-Object System.Collections.Generic.List[string]
    $max = [int](Get-Setting 'max_recipients' 30)
    try {
        if ($item.SenderEmailType -eq 'EX') { $addresses.Add((Get-Smtp $item.Sender $item.SenderEmailAddress)) }
        elseif ($item.SenderEmailAddress) { $addresses.Add($item.SenderEmailAddress) }
    } catch { }
    try {
        if ($item.Organizer) { $addresses.Add($item.GetOrganizer().Address) }
    } catch { }
    try {
        $count = [Math]::Min($item.Recipients.Count, $max)
        for ($i = 1; $i -le $count; $i++) {
            $r = $item.Recipients.Item($i)
            $addresses.Add((Get-Smtp $r.AddressEntry $r.Address))
        }
    } catch { }
    $ignore = @(Get-Setting 'ignore_domains' @()) | ForEach-Object { $_.ToLower().TrimStart('@') }
    $domains = @($addresses | ForEach-Object { Get-Domain $_ } | Where-Object { $_ -and $ignore -notcontains $_ } | Sort-Object -Unique)
    return @{ subject = [string]$item.Subject; domains = $domains }
}

Wait-Initialize
Send-Status 'running' 'Waiting for Outlook to be in front'
$lastKey = $null
$lastSent = [DateTime]::MinValue

while ($true) {
    [void](Receive-Messages)
    $fg = $script:foreground
    if ($fg -and $fg.exe -eq 'outlook.exe') {
        try {
            $outlook = Get-Outlook
            if ($null -eq $outlook) {
                Send-Status 'degraded' 'Outlook is running but cannot be reached; is one of them started as administrator?'
            } else {
                $item = Get-CurrentItem $outlook
                if ($item) {
                    $info = Get-ItemInfo $item
                    $key = "$($fg.hwnd)|$($info.subject)|$($info.domains -join ',')"
                    # Repeated now and then so the host keeps it while you read.
                    if ($key -ne $lastKey -or ([DateTime]::UtcNow - $lastSent).TotalSeconds -ge 15) {
                        Send-Message 'detail' @{ hwnd = $fg.hwnd; exe = 'outlook.exe'; document = $info.subject; mail_domains = $info.domains }
                        $lastKey = $key
                        $lastSent = [DateTime]::UtcNow
                    }
                }
                Send-Status 'running' 'Reading the selected Outlook item'
            }
        } catch {
            Send-Status 'degraded' "Outlook could not be read: $($_.Exception.Message)"
        }
    }
    Wait-ForHost ([int](Get-Setting 'poll_seconds' 3) * 1000)
}
