. "$PSScriptRoot\TestKit.ps1"
. (Use-Plugin 'office-paths')

$fg = [pscustomobject]@{ hwnd = 42; exe = 'winword.exe' }

Test 'a saved document is a path' {
    Assert-Equal @{ hwnd = 42; exe = 'winword.exe'; path = 'D:\Klanten\Contoso\Offerte.docx' } (New-PathDetail $fg 'D:\Klanten\Contoso\Offerte.docx')
}

Test 'OneDrive and SharePoint documents are web addresses' {
    Assert-Equal 'https://contoso.sharepoint.com/sites/x/Plan.xlsx' (New-PathDetail $fg 'https://contoso.sharepoint.com/sites/x/Plan.xlsx').path
}

Test 'an unsaved document is only a name' {
    Assert-Equal @{ hwnd = 42; exe = 'winword.exe'; document = 'Document1' } (New-PathDetail $fg 'Document1')
}

Complete-Tests
