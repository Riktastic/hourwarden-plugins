. "$PSScriptRoot/TestKit.ps1"
. (Use-Plugin 'remote-workspaces')

Test 'Citrix desktop and app' {
    Assert-Equal 'Contoso Beheerwerkplek' (Get-WorkspaceName 'CDViewer.exe' 'Contoso Beheerwerkplek - Desktop Viewer')
    Assert-Equal 'Fabrikam ERP' (Get-WorkspaceName 'wfica32.exe' 'Fabrikam ERP (Remote)')
}

Test 'Azure Virtual Desktop, Windows 365 and Horizon' {
    Assert-Equal 'CTS-AVD-Admin' (Get-WorkspaceName 'msrdc.exe' 'CTS-AVD-Admin - Remote Desktop')
    Assert-Equal 'Contoso Cloud PC' (Get-WorkspaceName 'windowsapp.exe' 'Contoso Cloud PC - Windows App')
    Assert-Equal 'FAB-VDI-01' (Get-WorkspaceName 'vmware-view.exe' 'FAB-VDI-01 - VMware Horizon Client')
}

Test 'the clients own windows are no workplace' {
    Assert-Equal $null (Get-WorkspaceName 'windowsapp.exe' 'Windows App')
    Assert-Equal $null (Get-WorkspaceName 'msrdc.exe' 'Remote Desktop')
    Assert-Equal $null (Get-WorkspaceName 'notepad.exe' 'Contoso - Desktop Viewer')
}

Test 'Citrix and Horizon on Linux' {
    Assert-Equal 'Fabrikam ERP' (Get-WorkspaceName 'wfica' 'Fabrikam ERP (Remote)')
    Assert-Equal 'FAB-VDI-01' (Get-WorkspaceName 'vmware-view' 'FAB-VDI-01 - VMware Horizon Client')
}

Complete-Tests
