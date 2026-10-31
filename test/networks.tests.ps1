. "$PSScriptRoot/TestKit.ps1"
. (Use-Plugin 'networks')

function Profile([string]$Name, [string]$Alias) { [pscustomobject]@{ Name = $Name; InterfaceAlias = $Alias } }
function Adapter([string]$Description, [bool]$Hardware = $true, [string]$Media = '802.3') {
    [pscustomobject]@{ InterfaceDescription = $Description; HardwareInterface = $Hardware; PhysicalMediaType = $Media }
}

$adapters = @{
    'Wi-Fi'     = Adapter 'Intel Wi-Fi 6E AX211' $true 'Native 802.11'
    'Ethernet'  = Adapter 'Realtek USB GbE'
    'Tailscale' = Adapter 'Tailscale Tunnel' $false 'Unspecified'
    'VPN'       = Adapter 'Cisco AnyConnect Virtual Miniport Adapter' $false 'Unspecified'
}

Test 'Wi-Fi before wired, virtual adapters skipped' {
    $profiles = @((Profile 'tailnet' 'Tailscale'), (Profile 'Office LAN' 'Ethernet'), (Profile 'Olifantenpoot 5G' 'Wi-Fi'))
    Assert-Equal @{ name = 'Olifantenpoot 5G'; kind = 'wifi' } (Select-Network $profiles $adapters @() $true @())
}

Test 'wired when there is no Wi-Fi' {
    Assert-Equal @{ name = 'Office LAN'; kind = 'wired' } (Select-Network @((Profile 'Office LAN' 'Ethernet')) $adapters @() $true @())
}

Test 'a connected Windows VPN wins when preferred' {
    $vpn = @([pscustomobject]@{ Name = 'Contoso VPN'; ConnectionStatus = 'Connected' })
    Assert-Equal @{ name = 'Contoso VPN'; kind = 'vpn' } (Select-Network @((Profile 'Home' 'Wi-Fi')) $adapters $vpn $true @())
    Assert-Equal 'Home' (Select-Network @((Profile 'Home' 'Wi-Fi')) $adapters $vpn $false @()).name
}

Test 'a VPN client adapter counts as VPN' {
    $profiles = @((Profile 'Home' 'Wi-Fi'), (Profile 'contoso.local' 'VPN'))
    Assert-Equal @{ name = 'contoso.local'; kind = 'vpn' } (Select-Network $profiles $adapters @() $true @())
}

Test 'ignored networks are skipped' {
    $profiles = @((Profile 'Home' 'Wi-Fi'), (Profile 'Office LAN' 'Ethernet'))
    Assert-Equal 'Office LAN' (Select-Network $profiles $adapters @() $true @('Home')).name
}

Test 'nothing connected' {
    Assert-Equal $null (Select-Network @() $adapters @() $true @())
}

Test 'Linux: Wi-Fi from NetworkManager' {
    $lines = @('lo:loopback', 'Office LAN:802-3-ethernet', 'Olifantenpoot 5G:802-11-wireless', 'docker0:bridge')
    Assert-Equal @{ name = 'Olifantenpoot 5G'; kind = 'wifi' } (Select-NmNetwork $lines $true @())
    Assert-Equal @{ name = 'Office LAN'; kind = 'wired' } (Select-NmNetwork @('Office LAN:802-3-ethernet', 'lo:loopback') $true @())
}

Test 'Linux: a VPN wins when preferred' {
    $lines = @('Home:802-11-wireless', 'Contoso VPN:vpn')
    Assert-Equal @{ name = 'Contoso VPN'; kind = 'vpn' } (Select-NmNetwork $lines $true @())
    Assert-Equal 'Home' (Select-NmNetwork $lines $false @()).name
    Assert-Equal 'wg-contoso' (Select-NmNetwork @('Home:802-11-wireless', 'wg-contoso:wireguard') $true @()).name
}

Test 'Linux: escaped colons, ignored and no networks' {
    Assert-Equal 'Cafe: guest' (Select-NmNetwork @('Cafe\: guest:802-11-wireless') $true @()).name
    Assert-Equal 'Office LAN' (Select-NmNetwork @('Home:802-11-wireless', 'Office LAN:802-3-ethernet') $true @('Home')).name
    Assert-Equal $null (Select-NmNetwork @() $true @())
    Assert-Equal $null (Select-NmNetwork @('lo:loopback') $true @())
}

Complete-Tests
