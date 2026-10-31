. "$PSScriptRoot/TestKit.ps1"
. (Use-Plugin 'admin-tools')

Test 'SQL Server Management Studio: server and database' {
    Assert-Equal @{ remote_server = 'sql01\prod'; document = 'Sales' } (Get-AdminTarget 'ssms.exe' 'Query.sql - SQL01\PROD.Sales (CONTOSO\rik (53)) - Microsoft SQL Server Management Studio')
    Assert-Equal @{ remote_server = 'sql01.contoso.local'; document = 'master' } (Get-AdminTarget 'Ssms.exe' 'SQLQuery1.sql - sql01.contoso.local.master (sa (61))* - Microsoft SQL Server Management Studio')
    Assert-Equal $null (Get-AdminTarget 'ssms.exe' 'Microsoft SQL Server Management Studio')
}

Test 'WinSCP and FileZilla' {
    Assert-Equal @{ remote_server = 'web01.contoso.nl' } (Get-AdminTarget 'winscp.exe' '/var/www - deploy@web01.contoso.nl - WinSCP')
    Assert-Equal @{ remote_server = 'ftp.fabrikam.nl' } (Get-AdminTarget 'filezilla.exe' 'Fabrikam - sftp://deploy@ftp.fabrikam.nl - FileZilla')
    Assert-Equal $null (Get-AdminTarget 'filezilla.exe' 'FileZilla')
}

Test 'Hyper-V and VMware consoles' {
    Assert-Equal @{ remote_server = 'APP01'; remote_folder = 'HV-HOST01' } (Get-AdminTarget 'vmconnect.exe' 'APP01 on HV-HOST01 - Virtual Machine Connection')
    Assert-Equal @{ remote_server = 'Contoso DC02' } (Get-AdminTarget 'vmrc.exe' 'Contoso DC02 - VMware Remote Console')
}

Test 'management consoles give the domain or server' {
    Assert-Equal @{ remote_server = 'dc01.contoso.local' } (Get-AdminTarget 'mmc.exe' 'Active Directory Users and Computers - [Active Directory Users and Computers [dc01.contoso.local]]')
    Assert-Equal @{ remote_server = 'dc01.fabrikam.nl' } (Get-AdminTarget 'mmc.exe' 'DNS Manager - [DNS\dc01.fabrikam.nl]')
    Assert-Equal @{ remote_server = 'contoso.local' } (Get-AdminTarget 'mmc.exe' 'Group Policy Management - [Group Policy Management\Forest: contoso.local]')
    Assert-Equal $null (Get-AdminTarget 'mmc.exe' 'Computer Management - [Computer Management (Local)]')
}

Test 'other programs are left alone' {
    Assert-Equal $null (Get-AdminTarget 'notepad.exe' 'sql01.contoso.local.txt - Notepad')
}

Test 'FileZilla on Linux' {
    Assert-Equal @{ remote_server = 'ftp.contoso.nl' } (Get-AdminTarget 'filezilla' 'Contoso site - sftp://deploy@ftp.contoso.nl - FileZilla')
}

Complete-Tests
