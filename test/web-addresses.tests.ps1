. "$PSScriptRoot/TestKit.ps1"
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

Test 'Azure portal addresses give the customer tenant' {
    Assert-Equal @('portal.azure.com', 'contoso.onmicrosoft.com') @(Get-WebDomains 'https://portal.azure.com/#@contoso.onmicrosoft.com/resource/subscriptions/x')
    Assert-Equal @('entra.microsoft.com', 'fabrikam.nl') @(Get-WebDomains 'https://entra.microsoft.com/#@fabrikam.nl/view/Users')
    Assert-Equal @('portal.azure.com') @(Get-WebDomains 'https://portal.azure.com/#home')
}

Test 'Power BI and Fabric addresses give the tenant and workspace' {
    $t = '72f988bf-86f1-41af-91ab-2d7cd011db47'
    $w = 'a1b2c3d4-0000-4000-8000-1234567890ab'
    Assert-Equal @('app.powerbi.com', $t, $w) @(Get-WebDomains "https://app.powerbi.com/groups/$w/reports/0f0e/ReportSection?ctid=$($t.ToUpper())&experience=power-bi")
    Assert-Equal @('app.fabric.microsoft.com', $t, $w) @(Get-WebDomains "app.fabric.microsoft.com/groups/$w/lakehouses/x?experience=fabric-developer&ctid=$t")
    Assert-Equal @('app.powerbi.com') @(Get-WebDomains 'https://app.powerbi.com/groups/me/list?experience=power-bi')
    $ids = @(Get-MicrosoftIds "https://app.powerbi.com/groups/$w/reports?ctid=$t")
    Assert-Equal @('tenant', 'workspace') @($ids | ForEach-Object { $_.Kind })
}

Test 'Azure portal addresses give the subscription and a tenant ID' {
    $t = '72f988bf-86f1-41af-91ab-2d7cd011db47'
    $s = '11111111-2222-3333-4444-555555555555'
    Assert-Equal @('portal.azure.com', $t, $s) @(Get-WebDomains "https://portal.azure.com/#@$t/resource/subscriptions/$s/resourceGroups/rg-web/overview")
    Assert-Equal @('portal.azure.com', $s) @(Get-WebDomains "portal.azure.com/#view/HubsExtension/BrowseResource/subscriptions/$s")
    Assert-Equal @('entra.microsoft.com') @(Get-WebDomains "https://entra.microsoft.com/#view/Microsoft_AAD_IAM/GroupDetailsMenuBlade/~/Overview/groupId/$s")
}

Test 'each Microsoft ID is logged once' {
    $script:sent.Clear()
    $t = '72f988bf-86f1-41af-91ab-2d7cd011db47'
    Write-NewIds "https://app.powerbi.com/home?ctid=$t"
    Write-NewIds "https://app.fabric.microsoft.com/home?ctid=$t"
    Assert-Equal 1 $script:sent.Count
    Assert-Equal "Seen tenant $t on app.powerbi.com" $script:sent[0].params.message
}

Test 'SharePoint admin and OneDrive count as the SharePoint host' {
    Assert-Equal @('contoso.sharepoint.com') @(Get-WebDomains 'https://contoso-admin.sharepoint.com/_layouts/15/online/AdminHome.aspx')
    Assert-Equal @('contoso.sharepoint.com') @(Get-WebDomains 'https://contoso-my.sharepoint.com/personal/jan')
    Assert-Equal @() @(Get-WebDomains 'edge://settings')
}

Test 'ignored domains include their subdomains' {
    Assert-True (Test-IgnoredDomain 'mail.rik.blue' @('rik.blue'))
    Assert-True (Test-IgnoredDomain 'rik.blue' @('@rik.blue'))
    Assert-True (-not (Test-IgnoredDomain 'notrik.blue' @('rik.blue')))
    Assert-True (-not (Test-IgnoredDomain 'contoso.com' @()))
}

Complete-Tests
