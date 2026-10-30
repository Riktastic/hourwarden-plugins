. "$PSScriptRoot\TestKit.ps1"
. (Use-Plugin 'remote-desktop')

Test 'tab name with folders' {
    Assert-Equal @{ folder = 'Contoso\Production'; name = 'APP01' } (Split-RdmEntry 'Contoso\Production\APP01')
}

Test 'tab name without a folder' {
    Assert-Equal @{ folder = $null; name = 'db01.fabrikam.local' } (Split-RdmEntry 'db01.fabrikam.local')
}

Test 'server from a Remote Desktop Connection title' {
    Assert-Equal 'app01.contoso.local' (Get-MstscServer 'app01.contoso.local - Remote Desktop Connection')
    Assert-Equal '10.0.0.5' (Get-MstscServer '10.0.0.5 - Verbinding met extern bureaublad')
}

Test 'the connection dialog itself has no server' {
    Assert-Equal $null (Get-MstscServer 'Remote Desktop Connection')
    Assert-Equal $null (Get-MstscServer '')
}

Complete-Tests
