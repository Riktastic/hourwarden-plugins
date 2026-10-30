# SSH sessions in terminals: the server from the window title.
#
# Windows Terminal shows the running command or the shell prompt
# ("ssh admin@web01", "rik@web01: ~/app"); PuTTY, KiTTY, SecureCRT and
# others show the host. The server is matched like a remote desktop server:
# add a remote server rule or a customer alias that appears in its name.

. "$PSScriptRoot\protocol.ps1"

$defaultApps = @('windowsterminal.exe', 'openconsole.exe', 'conhost.exe', 'putty.exe', 'kitty.exe', 'mobaxterm.exe', 'mremoteng.exe',
    'securecrt.exe', 'tabby.exe', 'termius.exe', 'wezterm-gui.exe', 'alacritty.exe', 'royalts.exe')
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
function Get-SshHost([string]$Title, [string[]]$LocalNames = @($env:COMPUTERNAME)) {
    if (-not $Title) { return $null }
    $t = ($Title -replace '\s+[-|]\s+(PuTTY|KiTTY|SecureCRT|Tabby|Termius|MobaXterm|mRemoteNG)\b.*$', '').Trim()
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

if ($script:testMode) { return }
Wait-Initialize
Send-Status 'running' 'Watching terminal titles for SSH sessions'
$lastKey = $null
$lastSent = [DateTime]::MinValue

while ($true) {
    [void](Receive-Messages)
    $fg = $script:foreground
    $apps = @(Get-Setting 'apps' $defaultApps | ForEach-Object { $_.ToLower() })
    if ($fg -and $apps -contains $fg.exe) {
        $local = @($env:COMPUTERNAME) + @(Get-Setting 'local_hosts' @())
        $server = Get-SshHost $fg.title $local
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
