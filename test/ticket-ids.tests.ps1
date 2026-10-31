. "$PSScriptRoot/TestKit.ps1"
. (Use-Plugin 'ticket-ids')

function Hint([string]$Title) { Get-TicketHint $Title $defaultPatterns $defaultIgnore }

Test 'Jira page in Edge' {
    Assert-Equal @{ ticket_id = 'CON-123'; title = 'Login fails' } (Hint "[CON-123] Login fails - Jira - Work - Microsoft$([char]0x200B) Edge")
}

Test 'ticket ID after the summary' {
    Assert-Equal @{ ticket_id = 'CON-123'; title = 'Login fails' } (Hint 'Login fails (CON-123) - Jira')
}

Test 'mail reply in Outlook' {
    Assert-Equal @{ ticket_id = 'FAB-7'; title = 'Export is slow' } (Hint 'RE: FW: [FAB-7] Export is slow - Message (HTML)')
}

Test 'TopDesk number in its own part of the title' {
    Assert-Equal @{ ticket_id = 'I 2410 123'; title = 'Printer broken' } (Hint 'I 2410 123 - Printer broken - TOPdesk - Microsoft Edge')
    Assert-Equal 'I2410-0042' (Hint 'Melding I2410-0042 bekijken - TOPdesk').ticket_id
}

Test 'ServiceNow incident' {
    Assert-Equal 'INC0012345' (Hint 'INC0012345 | Mail not delivered | ServiceNow').ticket_id
}

Test 'look-alikes are not tickets' {
    Assert-Equal $null (Hint 'Save as UTF-8 - Notepad')
    Assert-Equal $null (Hint 'COVID-19 measures - Intranet')
    Assert-Equal $null (Hint 'Inbox - jan@contoso.com - Outlook')
    Assert-Equal $null (Hint '')
}

Test 'your own pattern' {
    Assert-Equal 'T#4411' (Get-TicketHint 'Ticket T#4411: VPN down' @('T#\d+') '').ticket_id
}

Test 'browsers on Linux are recognised by their program name' {
    Assert-True (Test-App 'firefox' $defaultApps)
    Assert-True (Test-App 'msedge' $defaultApps)
    Assert-True (Test-App 'chromium' $defaultApps)
}

Complete-Tests
