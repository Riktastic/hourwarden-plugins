# Wi-Fi and VPN: the network the computer is connected to.
#
# On Windows, network profile names come from Windows' network list (for
# Wi-Fi this is normally the SSID) and do not need location access; VPN
# connections come from Get-VpnConnection. On Linux they come from
# NetworkManager (nmcli).

. "$PSScriptRoot/protocol.ps1"

# Company VPN clients that use their own network adapter instead of Windows VPN.
$vpnClients = 'AnyConnect|GlobalProtect|PANGP|Forti|Pulse|Ivanti|Check Point|Zscaler|WireGuard|OpenVPN|TAP-Windows|SonicWall|NetExtender|Juniper'

# Chooses the network from connection profiles (Name, InterfaceAlias),
# adapters by name (InterfaceDescription, HardwareInterface,
# PhysicalMediaType) and Windows VPN connections (Name, ConnectionStatus).
function Select-Network($Profiles, [hashtable]$Adapters, $Vpn, [bool]$PreferVpn, $Ignore) {
    $Ignore = @($Ignore)
    $Profiles = @($Profiles | Where-Object { $Ignore -notcontains $_.Name })

    if ($PreferVpn) {
        $connected = @($Vpn | Where-Object { $_.ConnectionStatus -eq 'Connected' -and $Ignore -notcontains $_.Name })
        if ($connected.Count) { return @{ name = $connected[0].Name; kind = 'vpn' } }
        $client = $Profiles | Where-Object { $Adapters[$_.InterfaceAlias] -and $Adapters[$_.InterfaceAlias].InterfaceDescription -match $vpnClients } | Select-Object -First 1
        if ($client) { return @{ name = $client.Name; kind = 'vpn' } }
    }

    # Physical adapters only: virtual ones (Hyper-V, WSL, Tailscale) say nothing about where you are.
    $physical = @($Profiles | Where-Object { $Adapters[$_.InterfaceAlias] -and $Adapters[$_.InterfaceAlias].HardwareInterface })
    $wifi = $physical | Where-Object { $Adapters[$_.InterfaceAlias].PhysicalMediaType -match '802\.11|Wireless' -or $_.InterfaceAlias -match 'Wi-?Fi|WLAN|Wireless' } | Select-Object -First 1
    if ($wifi) { return @{ name = $wifi.Name; kind = 'wifi' } }
    $wired = $physical | Select-Object -First 1
    if ($wired) { return @{ name = $wired.Name; kind = 'wired' } }
    return $null
}

# Chooses the network from NetworkManager's active connections, as printed
# by "nmcli -t -f NAME,TYPE connection show --active": lines such as
# "Home:802-11-wireless" (a colon in a name is written as "\:").
function Select-NmNetwork([string[]]$Lines, [bool]$PreferVpn, $Ignore) {
    $Ignore = @($Ignore)
    $connections = @(foreach ($line in @($Lines)) {
        $m = [regex]::Match([string]$line, '^(?<name>(?:\\.|[^:\\])+):(?<type>[^:]+)$')
        if (-not $m.Success) { continue }
        $name = $m.Groups['name'].Value -replace '\\(.)', '$1'
        if ($Ignore -notcontains $name) { [pscustomobject]@{ Name = $name; Type = $m.Groups['type'].Value } }
    })
    if ($PreferVpn) {
        $vpn = $connections | Where-Object { $_.Type -in @('vpn', 'wireguard', 'tun') } | Select-Object -First 1
        if ($vpn) { return @{ name = $vpn.Name; kind = 'vpn' } }
    }
    $wifi = $connections | Where-Object Type -eq '802-11-wireless' | Select-Object -First 1
    if ($wifi) { return @{ name = $wifi.Name; kind = 'wifi' } }
    $wired = $connections | Where-Object Type -eq '802-3-ethernet' | Select-Object -First 1
    if ($wired) { return @{ name = $wired.Name; kind = 'wired' } }
    return $null
}

function Get-CurrentNetwork {
    $preferVpn = [bool](Get-Setting 'prefer_vpn' $true)
    if (-not $script:onWindows) {
        if (-not (Get-Command nmcli -ErrorAction SilentlyContinue)) { throw 'NetworkManager (nmcli) is not installed' }
        $lines = @(& nmcli -t -f NAME,TYPE connection show --active 2>$null)
        return Select-NmNetwork $lines $preferVpn @(Get-Setting 'ignore_networks' @())
    }
    $adapters = @{}
    foreach ($a in Get-NetAdapter -ErrorAction SilentlyContinue | Where-Object Status -eq 'Up') { $adapters[$a.Name] = $a }
    $profiles = @(Get-NetConnectionProfile -ErrorAction SilentlyContinue)
    $vpn = if ($preferVpn) { @(Get-VpnConnection -ErrorAction SilentlyContinue) } else { @() }
    return Select-Network $profiles $adapters $vpn $preferVpn @(Get-Setting 'ignore_networks' @())
}

if ($script:testMode) { return }
Wait-Initialize
Send-Status 'running' 'Watching the network'
$last = $null
$lastSent = [DateTime]::MinValue

while ($true) {
    [void](Receive-Messages)
    try {
        $network = Get-CurrentNetwork
        $name = if ($network) { $network.name } else { $null }
        # Repeated every minute so the host knows it is still current.
        if ($name -and ($name -ne $last -or ([DateTime]::UtcNow - $lastSent).TotalSeconds -ge 60)) {
            Send-Message 'network' $network
            if ($name -ne $last) { Send-Log "Connected to $name ($($network.kind))" }
            $last = $name
            $lastSent = [DateTime]::UtcNow
            Send-Status 'running' "Connected to $name"
        } elseif (-not $name) {
            $last = $null
            Send-Status 'running' 'Not connected'
        }
    } catch {
        Send-Status 'degraded' "Could not read the network: $($_.Exception.Message)"
    }
    Wait-ForHost ([int](Get-Setting 'poll_seconds' 30) * 1000)
}
