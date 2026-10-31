# Admin tools: the server, database or domain you manage, from the title
# of SQL Server Management Studio, WinSCP, FileZilla, Hyper-V and VMware
# consoles and Windows management consoles (Active Directory, DNS, Group
# Policy). The server counts like a remote desktop server: add a remote
# server rule, or a customer alias that is part of its name. FileZilla and
# VMware Remote Console also run on Linux; the others are Windows programs.

. "$PSScriptRoot/protocol.ps1"

$hostName = '[A-Za-z0-9][A-Za-z0-9-]*(\.[A-Za-z0-9-]+)*'

# What a title says about the system you work on, or $null.
function Get-AdminTarget([string]$Exe, [string]$Title) {
    if (-not $Title) { return $null }
    switch (Get-AppName $Exe) {
        'ssms' {
            # "Query.sql - SQL01\PROD.Sales (CONTOSO\rik (53)) - Microsoft SQL Server Management Studio"
            if ($Title -match "\s-\s(?<target>[^\s()]+)\s\([^)]*\)") {
                $target = $Matches.target
                $i = $target.LastIndexOf('.')
                if ($i -gt 0) { return @{ remote_server = $target.Substring(0, $i).ToLower(); document = $target.Substring($i + 1) } }
                return @{ remote_server = $target.ToLower() }
            }
        }
        'winscp' {
            # "/var/www - deploy@web01.contoso.nl - WinSCP"
            if ($Title -match "(?:^|\s-\s)[^\s@]+@(?<host>$hostName)\s-\sWinSCP") { return @{ remote_server = $Matches.host.ToLower() } }
        }
        'filezilla' {
            # "Contoso site - sftp://deploy@ftp.contoso.nl - FileZilla"
            if ($Title -match "(?:s?ftps?|sftp)://(?:[^@/\s]+@)?(?<host>$hostName)") { return @{ remote_server = $Matches.host.ToLower() } }
        }
        'vmconnect' {
            # "APP01 on HV-HOST01 - Virtual Machine Connection"
            if ($Title -match "^(?<vm>.+?)\son\s(?<host>$hostName)\s-\s") { return @{ remote_server = $Matches.vm.Trim(); remote_folder = $Matches.host } }
        }
        'vmrc' {
            # "APP01 - VMware Remote Console"
            if ($Title -match "^(?<vm>.+?)\s-\sVMware Remote Console") { return @{ remote_server = $Matches.vm.Trim() } }
        }
        'mmc' {
            # "Active Directory Users and Computers - [... [dc01.contoso.local]]", "DNS Manager - [DNS\dc01.contoso.local]"
            foreach ($m in [regex]::Matches($Title, "[\[\\\s:](?<name>[A-Za-z0-9-]+(\.[A-Za-z0-9-]+)+)(?=[\]\\\s]|$)")) {
                $name = $m.Groups['name'].Value
                if ($name -notmatch '^\d+(\.\d+)+$' -or $name.Split('.').Count -eq 4) { return @{ remote_server = $name.ToLower() } }
            }
        }
    }
    return $null
}

if ($script:testMode) { return }
Wait-Initialize
Send-Status 'running' 'Waiting for an admin tool to be in front'
$lastKey = $null
$lastSent = [DateTime]::MinValue

while ($true) {
    [void](Receive-Messages)
    $fg = $script:foreground
    if ($fg) {
        $target = Get-AdminTarget $fg.exe $fg.title
        if ($target) {
            $detail = @{ hwnd = $fg.hwnd; exe = $fg.exe } + $target
            $key = "$($fg.hwnd)|$($target.remote_server)|$($target.document)"
            if ($key -ne $lastKey -or ([DateTime]::UtcNow - $lastSent).TotalSeconds -ge 15) {
                Send-Message 'detail' $detail
                if ($key -ne $lastKey) { Send-Log "$($fg.exe): $($target.remote_server)" }
                $lastKey = $key
                $lastSent = [DateTime]::UtcNow
            }
        }
    }
    Wait-ForHost ([int](Get-Setting 'poll_seconds' 2) * 1000)
}
