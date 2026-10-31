. "$PSScriptRoot/TestKit.ps1"
. (Use-Plugin 'terminal-ssh')

$local = @('LAPTOP-RIK')

Test 'ssh command with options' {
    Assert-Equal 'web01.contoso.local' (Get-SshHost 'ssh -p 2222 -i C:\keys\id admin@web01.contoso.local' $local)
    Assert-Equal 'db01' (Get-SshHost 'ssh.exe -A db01 uptime' $local)
    Assert-Equal 'app01' (Get-SshHost 'Administrator: ssh ssh://root@app01:22' $local)
}

Test 'shell prompts' {
    Assert-Equal 'web01' (Get-SshHost 'rik@web01: ~/app' $local)
    Assert-Equal 'db01' (Get-SshHost 'root@db01:~#' $local)
    Assert-Equal 'app01' (Get-SshHost '[admin@app01 ~]$' $local)
}

Test 'PuTTY and friends show the host' {
    Assert-Equal 'fw01.fabrikam.nl' (Get-SshHost 'fw01.fabrikam.nl - PuTTY' $local)
    Assert-Equal 'web02' (Get-SshHost 'admin@web02: ~ - KiTTY' $local)
}

Test 'your own machine and local shells are not servers' {
    Assert-Equal $null (Get-SshHost 'rik@LAPTOP-RIK: /mnt/c/Users' $local)
    Assert-Equal $null (Get-SshHost 'rik@laptop-rik.home: ~' $local)
    Assert-Equal $null (Get-SshHost 'ssh localhost' $local)
    Assert-Equal $null (Get-SshHost 'Windows PowerShell' $local)
    Assert-Equal $null (Get-SshHost 'Administrator: Command Prompt' $local)
    Assert-Equal $null (Get-SshHost 'C:\WINDOWS\system32\cmd.exe' $local)
}

Test 'Linux terminals' {
    Assert-Equal 'web01' (Get-SshHost "rik@web01: ~ $([char]0x2014) Konsole" $local)
    Assert-Equal 'db01.contoso.local' (Get-SshHost 'ssh admin@db01.contoso.local - Terminal' $local)
    Assert-Equal $null (Get-SshHost "rik@fedora: ~/Projects $([char]0x2014) Konsole" @('fedora'))
    Assert-True (Test-App 'konsole' $defaultApps)
    Assert-True (Test-App 'gnome-terminal-server' $defaultApps)
}

Test 'this computer is never a server' {
    $me = [Environment]::MachineName
    Assert-Equal $null (Get-SshHost "rik@$($me): ~")
}

Complete-Tests
