. "$PSScriptRoot\TestKit.ps1"
. (Use-Plugin 'web-addresses')

Test 'domain of an address' {
    Assert-Equal 'portal.contoso.com' (Get-WebHost 'https://portal.contoso.com/tickets?id=4')
    Assert-Equal 'contoso.sharepoint.com' (Get-WebHost 'contoso.sharepoint.com/sites/Project/Shared Documents')
    Assert-Equal 'fabrikam.nl' (Get-WebHost 'http://www.Fabrikam.nl:8080/')
    Assert-Equal 'intranet.local' (Get-WebHost 'https://user@intranet.local/')
}

Test 'browser pages, files and search words have no domain' {
    foreach ($a in @('edge://settings/profiles', 'chrome://newtab', 'about:blank', 'file:///C:/x.html', 'contoso offerte', 'localhost', '', 'chrome-extension://abc/popup.html')) {
        Assert-Equal $null (Get-WebHost $a) "for '$a'"
    }
}

Test 'ignored domains include their subdomains' {
    Assert-True (Test-IgnoredDomain 'mail.rik.blue' @('rik.blue'))
    Assert-True (Test-IgnoredDomain 'rik.blue' @('@rik.blue'))
    Assert-True (-not (Test-IgnoredDomain 'notrik.blue' @('rik.blue')))
    Assert-True (-not (Test-IgnoredDomain 'contoso.com' @()))
}

Complete-Tests
