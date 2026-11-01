# SSH sessions in terminals: the server from the window title, or else from
# the ssh command running in the terminal in front.
#
# Windows Terminal shows the running command or the shell prompt
# ("ssh admin@web01", "rik@web01: ~/app"); PuTTY, KiTTY, SecureCRT and
# others show the host. When the title says nothing (a custom prompt, a tab
# title), the command line of an ssh process started from that terminal
# names the server. The server is matched like a remote desktop server:
# add a remote server rule or a customer alias that appears in its name.

. "$PSScriptRoot/protocol.ps1"

# Names with or without .exe. The second line holds Linux terminals.
$defaultApps = @('windowsterminal.exe', 'openconsole.exe', 'conhost.exe', 'putty.exe', 'kitty.exe', 'mobaxterm.exe', 'mremoteng.exe',
    'securecrt.exe', 'tabby.exe', 'termius.exe', 'wezterm-gui.exe', 'alacritty.exe', 'royalts.exe',
    'konsole', 'yakuake', 'gnome-terminal-server', 'kgx', 'ptyxis', 'tilix', 'terminator', 'xfce4-terminal', 'xterm', 'foot', 'ghostty')
# ssh options that take a value: their value is not the host.
$optionsWithValue = @('-b', '-c', '-D', '-E', '-e', '-F', '-I', '-i', '-J', '-L', '-l', '-m', '-O', '-o', '-p', '-Q', '-R', '-S', '-W', '-w')
$hostPattern = '^[A-Za-z0-9]([A-Za-z0-9.-]*[A-Za-z0-9])?$'

function Test-RemoteHost([string]$Name, [string[]]$LocalNames) {
    if (-not $Name -or $Name -notmatch $hostPattern) { return $false }
    $short = $Name.Split('.')[0]
    foreach ($local in $LocalNames) { if ($local -and ($Name -ieq $local -or $short -ieq $local)) { return $false } }
    return $Name -notin @('localhost', '127.0.0.1')
}

# The server in a terminal title, or $null.
function Get-SshHost([string]$Title, [string[]]$LocalNames = @([Environment]::MachineName)) {
    if (-not $Title) { return $null }
    $t = ($Title -replace '\s+[-|\u2014]\s+(PuTTY|KiTTY|SecureCRT|Tabby|Termius|MobaXterm|mRemoteNG|Konsole|Yakuake|Tilix|Terminator|Terminal)\b.*$', '').Trim()
    $t = $t -replace '^(Administrator|Administrateur|Beheerder):\s*', ''

    # "ssh -p 2222 -i key admin@web01 uptime" or "ssh://admin@web01:22"
    if ($t -match '(?:^|[\s\\/])ssh(?:\.exe)?\s+(.+)$') {
        $tokens = @($Matches[1] -split '\s+')
        for ($i = 0; $i -lt $tokens.Count; $i++) {
            $token = $tokens[$i]
            if ($optionsWithValue -ccontains $token) { $i++; continue }
            if ($token.StartsWith('-')) { continue }
            $name = ($token -replace '^ssh://', '' -replace '^[^@]*@', '' -replace ':\d+$', '').Trim('"', "'")
            if (Test-RemoteHost $name $LocalNames) { return $name.ToLower() }
            break
        }
    }
    # Shell prompt: "rik@web01: ~/app", "root@db01:~#", "[admin@app01 ~]$"
    if ($t -match '^\[?[\w.-]+@(?<host>[\w.-]+)(?:[:\s\]]|$)' -and (Test-RemoteHost $Matches.host $LocalNames)) {
        return $Matches.host.ToLower()
    }
    # PuTTY and friends: the title is the host.
    if ($t -match '^(?<host>[\w.-]+)$' -and $t.Contains('.') -and (Test-RemoteHost $t $LocalNames)) {
        return $t.ToLower()
    }
    return $null
}

# The server in an ssh command line ("ssh -p 22 admin@web01"), or $null.
function Get-SshCommandHost([string]$CommandLine, [string[]]$LocalNames) {
    if (-not $CommandLine) { return $null }
    $line = $CommandLine -replace '^\s*"?[^"]*?[\\/]?ssh(\.exe)?"?(?=\s|$)', 'ssh'
    return Get-SshHost $line $LocalNames
}

# ssh processes and the process that started each: @{ Pid; Parent; Line }.
function Get-SshProcesses {
    if ($onWindows) {
        return @(Get-CimInstance Win32_Process -Filter "Name='ssh.exe'" -ErrorAction SilentlyContinue |
            ForEach-Object { @{ Pid = [int]$_.ProcessId; Parent = [int]$_.ParentProcessId; Line = [string]$_.CommandLine } })
    }
    return @(Get-Process ssh -ErrorAction SilentlyContinue | ForEach-Object { @{ Pid = $_.Id; Parent = $_.Parent.Id; Line = $_.CommandLine } })
}

function Get-ParentId([int]$Id) {
    if ($onWindows) { return [int](Get-CimInstance Win32_Process -Filter "ProcessId=$Id" -ErrorAction SilentlyContinue).ParentProcessId }
    return (Get-Process -Id $Id -ErrorAction SilentlyContinue).Parent.Id
}

# The server of the one ssh session started from the terminal `TerminalPid`;
# $null when there is none, or several (other tabs): then it cannot tell.
function Get-SshFromProcesses([int]$TerminalPid, [string[]]$LocalNames) {
    if (-not $TerminalPid) { return $null }
    $hosts = @()
    foreach ($p in Get-SshProcesses) {
        $id = $p.Parent
        for ($depth = 0; $id -and $depth -lt 6; $depth++) {
            if ($id -eq $TerminalPid) {
                $h = Get-SshCommandHost $p.Line $LocalNames
                if ($h -and $hosts -notcontains $h) { $hosts += $h }
                break
            }
            $id = Get-ParentId $id
        }
    }
    if ($hosts.Count -eq 1) { return $hosts[0] }
    return $null
}

if ($script:testMode) { return }
Wait-Initialize
Send-Status 'running' 'Watching terminal titles for SSH sessions'
$lastKey = $null
$lastSent = [DateTime]::MinValue
$script:procHwnd = $null
$script:procAt = [DateTime]::MinValue
$script:procServer = $null

while ($true) {
    [void](Receive-Messages)
    $fg = $script:foreground
    $apps = @(Get-Setting 'apps' $defaultApps | ForEach-Object { $_.ToLower() })
    if ($fg -and (Test-App $fg.exe $apps)) {
        $local = @([Environment]::MachineName) + @(Get-Setting 'local_hosts' @())
        $server = Get-SshHost $fg.title $local
        if (-not $server -and $fg.pid) {
            # Looking up processes takes a moment: once per window every 10 seconds.
            if ($fg.hwnd -ne $script:procHwnd -or ([DateTime]::UtcNow - $script:procAt).TotalSeconds -ge 10) {
                $script:procServer = Get-SshFromProcesses ([int]$fg.pid) $local
                $script:procHwnd = $fg.hwnd
                $script:procAt = [DateTime]::UtcNow
            }
            $server = $script:procServer
        }
        if ($server) {
            $key = "$($fg.hwnd)|$server"
            if ($key -ne $lastKey -or ([DateTime]::UtcNow - $lastSent).TotalSeconds -ge 15) {
                Send-Message 'detail' @{ hwnd = $fg.hwnd; exe = $fg.exe; remote_server = $server; document = $server }
                if ($key -ne $lastKey) { Send-Log "SSH session to $server" }
                $lastKey = $key
                $lastSent = [DateTime]::UtcNow
            }
        }
    }
    Wait-ForHost ([int](Get-Setting 'poll_seconds' 2) * 1000)
}
