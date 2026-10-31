# Citrix, Azure Virtual Desktop and Horizon

**Works on:** Windows and Linux. On Linux, plugins need PowerShell 7 (`pwsh`).

Recognises the remote desktop or app you work in through Citrix Workspace, Azure Virtual Desktop and Windows 365 (Windows App, Remote Desktop client) and VMware Horizon. The workplace name is matched against customer names and aliases.

## What it does

Recognises the remote desktop or app you work in through:

- Citrix Workspace,
- Azure Virtual Desktop and Windows 365 (Windows App and the Remote Desktop client),
- VMware Horizon.

The name of the workplace or published app is matched against customer names and aliases.

## Setting it up

Give the customer an alias that appears in the workplace name the customer gave you (for example `CTS-Desktop` gets the alias `CTS`).

## What it reads

Only the title of the remote workplace window in front.

## Settings

| Setting | Default | What it does |
|---|---|---|
| Check every | `2` | How often the window title is checked, in seconds. |
