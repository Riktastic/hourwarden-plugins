# Wi-Fi and VPN networks

**Works on:** Windows and Linux. On Linux it needs NetworkManager (nmcli), as on KDE Plasma, GNOME and most desktops.

Reports the Wi-Fi network or VPN you are connected to. Link a customer's office Wi-Fi or VPN to that customer; it is used only when nothing else decides the customer.

## What it does

Reports the Wi-Fi network or VPN you are connected to. It is the weakest evidence: it is only used when nothing else decides the customer, for example on a customer's office Wi-Fi.

When a VPN is connected (Windows VPN, AnyConnect, GlobalProtect, FortiClient and similar), the VPN is reported instead of the Wi-Fi network (*Prefer the VPN*).

## Setting it up

On the customer page, add a **network** rule with the Wi-Fi or VPN name. The customer page shows the current network, so you can add it with one click.

Add your own networks (home, your company VPN) under *Ignore these networks*.

## What it reads

Only the names of the connected networks and VPN connections.

## Settings

| Setting | Default | What it does |
|---|---|---|
| Check every | `30` | How often the connected network is checked, in seconds. |
| Prefer the VPN | `on` | When a Windows VPN or a company VPN client (AnyConnect, GlobalProtect, FortiClient and similar) is connected, report the VPN instead of the Wi-Fi network. |
| Ignore these networks | `(none)` | Network or VPN names that are never reported, such as your own VPN. |
