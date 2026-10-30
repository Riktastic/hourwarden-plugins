# Outlook (classic): subject and mail domains of the item in front, and
# counts of mail sent and received per mail domain.
#
# Uses the running Outlook through COM. Addresses of Exchange users are
# resolved to their SMTP address. Only metadata is read, never the body.

. "$PSScriptRoot\protocol.ps1"

$olInspector = 35
$olExchangeUser = 0
$olExchangeRemoteUser = 5
$olMail = 43
$olFolderSentMail = 5
$olFolderInbox = 6

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

# Mail domains of the sender (and organiser) and/or the recipients, without
# your own domains.
function Get-ItemDomains($item, [bool]$sender, [bool]$recipients) {
    $addresses = New-Object System.Collections.Generic.List[string]
    if ($sender) {
        try {
            if ($item.SenderEmailType -eq 'EX') { $addresses.Add((Get-Smtp $item.Sender $item.SenderEmailAddress)) }
            elseif ($item.SenderEmailAddress) { $addresses.Add($item.SenderEmailAddress) }
        } catch { }
        try {
            if ($item.Organizer) { $addresses.Add($item.GetOrganizer().Address) }
        } catch { }
    }
    if ($recipients) {
        $max = [int](Get-Setting 'max_recipients' 30)
        try {
            $count = [Math]::Min($item.Recipients.Count, $max)
            for ($i = 1; $i -le $count; $i++) {
                $r = $item.Recipients.Item($i)
                $addresses.Add((Get-Smtp $r.AddressEntry $r.Address))
            }
        } catch { }
    }
    $ignore = @(Get-Setting 'ignore_domains' @()) | ForEach-Object { $_.ToLower().TrimStart('@') }
    return @($addresses | ForEach-Object { Get-Domain $_ } | Where-Object { $_ -and $ignore -notcontains $_ } | Sort-Object -Unique)
}

# Reports every mail in Sent Items and the Inbox newer than the last check
# (at the start: the last few days). The host counts each key once.
$script:countedUntil = @{}
function Send-MailCounts($outlook) {
    $ns = $outlook.GetNamespace('MAPI')
    $start = (Get-Date).AddDays(-[int](Get-Setting 'count_days' 7))
    $kinds = @(
        @{ name = 'sent'; folder = $olFolderSentMail; date = 'SentOn'; metric = 'mail_sent'; sender = $false; recipients = $true },
        @{ name = 'received'; folder = $olFolderInbox; date = 'ReceivedTime'; metric = 'mail_received'; sender = $true; recipients = $false }
    )
    $total = 0
    foreach ($kind in $kinds) {
        $from = $script:countedUntil[$kind.name]
        if (-not $from -or $from -lt $start) { $from = $start }
        $items = $ns.GetDefaultFolder($kind.folder).Items
        $items.Sort("[$($kind.date)]", $true)
        $newest = $from
        $seen = 0
        $item = $items.GetFirst()
        while ($item -and $seen -lt 5000) {
            $seen++
            $when = $item.($kind.date)
            if ($when -lt $from) { break }
            if ($when -gt $newest) { $newest = $when }
            if ($item.Class -eq $olMail) {
                $domains = Get-ItemDomains $item $kind.sender $kind.recipients
                if ($domains.Count -gt 0) {
                    $at = ([DateTimeOffset]$when).ToUnixTimeMilliseconds()
                    Send-Message 'metric' @{ metric = $kind.metric; key = "$($kind.name):$($item.EntryID)"; at = $at; domains = $domains }
                    $total++
                }
            }
            $item = $items.GetNext()
        }
        $script:countedUntil[$kind.name] = $newest
    }
    return $total
}

if ($script:testMode) { return }
Wait-Initialize
Send-Status 'running' 'Waiting for Outlook to be in front'
$lastKey = $null
$lastSent = [DateTime]::MinValue
$lastCount = [DateTime]::MinValue

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
                    $info = @{ subject = [string]$item.Subject; domains = (Get-ItemDomains $item $true $true) }
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
    if ((Get-Setting 'count_mails' $true) -and ([DateTime]::UtcNow - $lastCount).TotalMinutes -ge [int](Get-Setting 'count_every_minutes' 5)) {
        $lastCount = [DateTime]::UtcNow
        $outlook = Get-Outlook
        if ($outlook) {
            try {
                $n = Send-MailCounts $outlook
                if ($n -gt 0) { Send-Log "Counted $n new mails" }
            } catch {
                Send-Log "Mails could not be counted: $($_.Exception.Message)"
            }
        }
    }
    Wait-ForHost ([int](Get-Setting 'poll_seconds' 3) * 1000)
}
