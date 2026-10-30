. "$PSScriptRoot\TestKit.ps1"
. (Use-Plugin 'vscode-git')

$dot = [char]0x25CF

Test 'file URIs become folders' {
    Assert-Equal 'd:\Projects\Timekeeper' (ConvertFrom-FileUri 'file:///d%3A/Projects/Timekeeper')
    Assert-Equal 'C:\Klanten\Contoso Portal' (ConvertFrom-FileUri 'file:///C:/Klanten/Contoso%20Portal/')
    Assert-Equal $null (ConvertFrom-FileUri 'vscode-remote://ssh-remote%2Bweb01/srv/app')
}

Test 'workspace name from the title' {
    Assert-Equal 'Timekeeper' (Get-WorkspaceFromTitle 'main.rs - Timekeeper - Visual Studio Code').name
    Assert-Equal 'Timekeeper' (Get-WorkspaceFromTitle "$dot main.rs - Timekeeper - Visual Studio Code").name
    Assert-Equal 'portal' (Get-WorkspaceFromTitle 'portal - Visual Studio Code - Insiders').name
    Assert-Equal 'Contoso' (Get-WorkspaceFromTitle 'Contoso (Workspace) - Cursor').name
}

Test 'a known workspace wins over the file and profile names' {
    Assert-Equal 'portal' (Get-WorkspaceFromTitle 'README.md - portal - Work - Visual Studio Code' @('portal', 'timekeeper')).name
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
    $storage = Join-Path $env:TEMP "tk-vscode-$PID"
    foreach ($n in 'a', 'b') { New-Item -ItemType Directory -Force (Join-Path $storage $n) | Out-Null }
    Set-Content (Join-Path $storage 'a\workspace.json') '{"folder":"file:///d%3A/Old/portal"}'
    Set-Content (Join-Path $storage 'b\workspace.json') '{"folder":"file:///d%3A/Klanten/Contoso/portal"}'
    (Get-Item (Join-Path $storage 'a\workspace.json')).LastWriteTime = (Get-Date).AddDays(-3)
    $map = Get-WorkspaceFolders @($storage, (Join-Path $storage 'missing'))
    Assert-Equal 'd:\Klanten\Contoso\portal' $map['portal']
    Remove-Item -Recurse -Force $storage
}

Test 'repository root is found from a subfolder' {
    $repo = Join-Path $env:TEMP "tk-repo-$PID"
    New-Item -ItemType Directory -Force (Join-Path $repo '.git'), (Join-Path $repo 'src\app') | Out-Null
    Assert-Equal $repo (Get-RepositoryRoot (Join-Path $repo 'src\app'))
    Remove-Item -Recurse -Force $repo
}

Complete-Tests
