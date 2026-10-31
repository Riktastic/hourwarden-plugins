# Ticket IDs in window titles: Jira (CON-123), TopDesk (I 2410 123),
# ServiceNow (INC0012345) and your own patterns.
#
# Only reads the title of the window in front. The ticket ID is offered in
# the top bar as the task; it never decides the customer and nothing is
# sent to Jira, TopDesk or anywhere else.

. "$PSScriptRoot/protocol.ps1"

# Names with or without .exe; on Linux the same browsers run as msedge,
# chrome, firefox and brave.
$defaultApps = @('msedge.exe', 'chrome.exe', 'firefox.exe', 'brave.exe', 'outlook.exe', 'olk.exe', 'ms-teams.exe', 'chromium', 'teams-for-linux')
$defaultPatterns = @(
    '\b[A-Z][A-Z0-9]{1,9}-\d{1,7}\b',                    # Jira, YouTrack, Linear: CON-123
    '\b[IMWC] ?\d{4}[ -]\d{3,6}\b',                      # TopDesk: I 2410 123, I2410-123
    '\b(?:INC|RITM|REQ|CHG|PRB|SCTASK|TASK)\d{7}\b'      # ServiceNow: INC0012345
)
# Things that look like ticket IDs but are not.
$defaultIgnore = '^(UTF|ISO|SHA|MD|COVID|WIN|TLS|SSL|IPV|HTTP|RFC|CVE|GPT|AES|RSA|USB|HDMI|PCI|DDR|LTE|IEEE|ASCII|ES|EN|NL|DE)-'

$replyPrefix = '^\s*((RE|FW|FWD|AW|WG|TR|ANTW|DOORST)\s*:\s*)+'
$trimChars = [char[]]' []():#-|'

# The first ticket ID in a title, with the words around it as the task:
# "[CON-123] Login fails - Jira - Microsoft Edge" gives CON-123 "Login fails".
function Get-TicketHint([string]$Title, [string[]]$Patterns, [string]$Ignore) {
    if (-not $Title) { return $null }
    $segments = @($Title -split '\s[-|]\s')
    foreach ($pattern in $Patterns) {
        for ($i = 0; $i -lt $segments.Count; $i++) {
            foreach ($m in [regex]::Matches($segments[$i], $pattern)) {
                $id = ($m.Value -replace '\s+', ' ').Trim()
                if ($Ignore -and $id -match $Ignore) { continue }
                $words = ($segments[$i].Remove($m.Index, $m.Length) -replace $replyPrefix, '').Trim($trimChars).Trim()
                # TopDesk puts the number in its own part: "I 2410 123 - Printer broken - TOPdesk".
                if (-not $words -and $i + 2 -lt $segments.Count) { $words = $segments[$i + 1].Trim() }
                return @{ ticket_id = $id; title = $words }
            }
        }
    }
    return $null
}

if ($script:testMode) { return }
Wait-Initialize
Send-Status 'running' 'Looking for ticket IDs in window titles'
$lastKey = $null
$lastSent = [DateTime]::MinValue

while ($true) {
    [void](Receive-Messages)
    $fg = $script:foreground
    $apps = @(Get-Setting 'apps' $defaultApps | ForEach-Object { $_.ToLower() })
    if ($fg -and (Test-App $fg.exe $apps)) {
        try {
            $hint = Get-TicketHint $fg.title @(Get-Setting 'patterns' $defaultPatterns) (Get-Setting 'ignore' $defaultIgnore)
            if ($hint) {
                $key = "$($fg.hwnd)|$($hint.ticket_id)"
                # Repeated while it stays in front, so the host keeps offering it.
                if ($key -ne $lastKey -or ([DateTime]::UtcNow - $lastSent).TotalSeconds -ge 15) {
                    Send-Message 'detail' @{ hwnd = $fg.hwnd; exe = $fg.exe; ticket_id = $hint.ticket_id; ticket_title = $hint.title }
                    if ($key -ne $lastKey) { Send-Log "Ticket $($hint.ticket_id) in $($fg.exe)" }
                    $lastKey = $key
                    $lastSent = [DateTime]::UtcNow
                }
            }
            Send-Status 'running' 'Looking for ticket IDs in window titles'
        } catch {
            Send-Status 'degraded' "A pattern does not work: $($_.Exception.Message)"
        }
    }
    Wait-ForHost ([int](Get-Setting 'poll_seconds' 2) * 1000)
}
