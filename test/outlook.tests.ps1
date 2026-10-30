. "$PSScriptRoot\TestKit.ps1"
. (Use-Plugin 'outlook')

# A stand-in for an Outlook mail item: sender and recipients with plain
# SMTP addresses (no Exchange lookups).
function New-Mail([string]$Sender, [string[]]$Recipients) {
    $list = [pscustomobject]@{ Count = $Recipients.Count; Items = @($Recipients | ForEach-Object { [pscustomobject]@{ Address = $_; AddressEntry = $null } }) }
    $list | Add-Member ScriptMethod Item { param($i) $this.Items[$i - 1] }
    [pscustomobject]@{ SenderEmailType = 'SMTP'; SenderEmailAddress = $Sender; Organizer = $null; Recipients = $list }
}

Test 'domain of an address' {
    Assert-Equal 'contoso.com' (Get-Domain 'Jan.Jansen@Contoso.COM')
    Assert-Equal $null (Get-Domain 'not an address')
    Assert-Equal $null (Get-Domain '/O=EXCHANGELABS/OU=EXCHANGE ADMINISTRATIVE GROUP/CN=RECIPIENTS/CN=JAN')
}

Test 'sender and recipient domains, unique and without your own' {
    $script:settings = [pscustomobject]@{ ignore_domains = @('@rik.blue') }
    $mail = New-Mail 'me@rik.blue' @('a@contoso.com', 'b@Contoso.com', 'c@fabrikam.nl')
    Assert-Equal @('contoso.com', 'fabrikam.nl') @(Get-ItemDomains $mail $true $true)
    $script:settings = $null
}

Test 'received mail: only the sender counts' {
    $mail = New-Mail 'service@contoso.com' @('me@rik.blue', 'colleague@fabrikam.nl')
    Assert-Equal @('contoso.com') @(Get-ItemDomains $mail $true $false)
}

Test 'sent mail: only the recipients count, up to the maximum' {
    $script:settings = [pscustomobject]@{ max_recipients = 1 }
    $mail = New-Mail 'me@rik.blue' @('a@contoso.com', 'b@fabrikam.nl')
    Assert-Equal @('contoso.com') @(Get-ItemDomains $mail $false $true)
    $script:settings = $null
}

Complete-Tests
