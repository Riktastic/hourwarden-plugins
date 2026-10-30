# Wi-Fi and VPN: the network the computer is connected to.
#
# Network profile names come from Windows' network list (for Wi-Fi this is
# normally the SSID) and do not need location access. VPN connections come
# from Get-VpnConnection.

. "$PSScriptRoot\protocol.ps1"

# Company VPN clients that use their own network adapter instead of Windows VPN.
$vpnClients = 'AnyConnect|GlobalProtect|PANGP|Forti|Pulse|Ivanti|Check Point|Zscaler|WireGuard|OpenVPN|TAP-Windows|SonicWall|NetExtender|Juniper'

function Get-CurrentNetwork {
    $ignore = @(Get-Setting 'ignore_networks' @())
    $adapters = @{}
    foreach ($a in Get-NetAdapter -ErrorAction SilentlyContinue | Where-Object Status -eq 'Up') { $adapters[$a.Name] = $a }
    $profiles = @(Get-NetConnectionProfile -ErrorAction SilentlyContinue | Where-Object { $ignore -notcontains $_.Name })

    if (Get-Setting 'prefer_vpn' $true) {
        $vpn = @(Get-VpnConnection -ErrorAction SilentlyContinue | Where-Object { $_.ConnectionStatus -eq 'Connected' -and $ignore -notcontains $_.Name })
        if ($vpn.Count) { return @{ name = $vpn[0].Name; kind = 'vpn' } }
        $client = $profiles | Where-Object { $adapters[$_.InterfaceAlias] -and $adapters[$_.InterfaceAlias].InterfaceDescription -match $vpnClients } | Select-Object -First 1
        if ($client) { return @{ name = $client.Name; kind = 'vpn' } }
    }

    # Physical adapters only: virtual ones (Hyper-V, WSL, Tailscale) say nothing about where you are.
    $physical = @($profiles | Where-Object { $adapters[$_.InterfaceAlias] -and $adapters[$_.InterfaceAlias].HardwareInterface })
    $wifi = $physical | Where-Object { $adapters[$_.InterfaceAlias].PhysicalMediaType -match '802\.11|Wireless' -or $_.InterfaceAlias -match 'Wi-?Fi|WLAN|Wireless' } | Select-Object -First 1
    if ($wifi) { return @{ name = $wifi.Name; kind = 'wifi' } }
    $wired = $physical | Select-Object -First 1
    if ($wired) { return @{ name = $wired.Name; kind = 'wired' } }
    return $null
}

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
