. "$PSScriptRoot/TestKit.ps1"
. (Use-Plugin 'vscode-git')

$dot = [char]0x25CF

Test 'file URIs become folders' {
    Assert-Equal 'd:\Projects\HourWarden' (ConvertFrom-FileUri 'file:///d%3A/Projects/HourWarden')
    Assert-Equal 'C:\Klanten\Contoso Portal' (ConvertFrom-FileUri 'file:///C:/Klanten/Contoso%20Portal/')
    Assert-Equal $null (ConvertFrom-FileUri 'vscode-remote://ssh-remote%2Bweb01/srv/app')
    Assert-Equal '/home/rik/Klanten/Contoso Portal' (ConvertFrom-FileUri 'file:///home/rik/Klanten/Contoso%20Portal/')
}

Test 'workspace name from the title' {
    Assert-Equal 'HourWarden' (Get-WorkspaceFromTitle 'main.rs - HourWarden - Visual Studio Code').name
    Assert-Equal 'HourWarden' (Get-WorkspaceFromTitle "$dot main.rs - HourWarden - Visual Studio Code").name
    Assert-Equal 'portal' (Get-WorkspaceFromTitle 'portal - Visual Studio Code - Insiders').name
    Assert-Equal 'Contoso' (Get-WorkspaceFromTitle 'Contoso (Workspace) - Cursor').name
}

Test 'a known workspace wins over the file and profile names' {
    Assert-Equal 'portal' (Get-WorkspaceFromTitle 'README.md - portal - Work - Visual Studio Code' @('portal', 'hourwarden')).name
}

Test 'remote SSH windows give their server' {
    $ws = Get-WorkspaceFromTitle 'app.py - api [SSH: web01.contoso.local] - Visual Studio Code'
    Assert-Equal @{ name = 'api'; remote_server = 'web01.contoso.local' } $ws
}

Test 'repository from git remotes' {
    Assert-Equal 'Contoso/portal' (Get-GitRemote "[core]`n`tbare = false`n[remote `"origin`"]`n`turl = git@github.com:Contoso/portal.git`n`tfetch = +refs/heads/*`n")
    Assert-Equal 'contoso/Web/portal' (Get-GitRemote "[remote `"origin`"]`n`turl = https://contoso@dev.azure.com/contoso/Web/_git/portal`n")
    Assert-Equal 'team/repo' (Get-GitRemote "[remote `"origin`"]`n`turl = ssh://git@bitbucket.fabrikam.nl:7999/team/repo.git`n")
    Assert-Equal 'fabrikam/infra' (Get-GitRemote "[remote `"upstream`"]`n`turl = https://gitlab.com/fabrikam/infra`n")
    Assert-Equal $null (Get-GitRemote "[core]`n`tbare = false`n")
}

Test 'workspace folders from VS Code storage, newest wins' {
    $storage = New-TestFolder 'tk-vscode'
    foreach ($n in 'a', 'b', 'c') { New-Item -ItemType Directory -Force (Join-Path $storage $n) | Out-Null }
    Set-Content (Join-Path $storage 'a/workspace.json') '{"folder":"file:///d%3A/Old/portal"}'
    Set-Content (Join-Path $storage 'b/workspace.json') '{"folder":"file:///d%3A/Klanten/Contoso/portal"}'
    Set-Content (Join-Path $storage 'c/workspace.json') '{"folder":"file:///home/rik/infra"}'
    (Get-Item (Join-Path $storage 'a/workspace.json')).LastWriteTime = (Get-Date).AddDays(-3)
    $map = Get-WorkspaceFolders @($storage, (Join-Path $storage 'missing'))
    Assert-Equal 'd:\Klanten\Contoso\portal' $map['portal']
    Assert-Equal '/home/rik/infra' $map['infra']
    Remove-Item -Recurse -Force $storage
}

Test 'repository root is found from a subfolder' {
    $repo = New-TestFolder 'tk-repo'
    $app = Join-Path (Join-Path $repo 'src') 'app'
    New-Item -ItemType Directory -Force (Join-Path $repo '.git'), $app | Out-Null
    Assert-Equal $repo (Get-RepositoryRoot $app)
    Remove-Item -Recurse -Force $repo
}

Test 'VS Code and its forks on Linux' {
    Assert-True ($editors.Contains((Get-AppName 'code')))
    Assert-True ($editors.Contains((Get-AppName 'Code.exe')))
    Assert-Equal 'VSCodium' $editors[(Get-AppName 'codium')]
    Assert-Equal 'Code - Insiders' $editors[(Get-AppName 'code-insiders')]
}

Complete-Tests
