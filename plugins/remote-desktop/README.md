# Remote Desktop Manager and RDP

**Works on:** Windows only: it reads Remote Desktop Manager and Remote Desktop Connection on Windows.

Recognises the Remote Desktop Manager session or Remote Desktop Connection in front: the entry, its folder (such as Contoso\Production) and the server, so time goes to the customer of that folder or server.

## What it does

Recognises the Remote Desktop Manager session or Remote Desktop Connection in front: the entry name, its folder (such as `Contoso\Production`) and the server. Time goes to the customer of that folder or server.

## Setting it up

- In Remote Desktop Manager, keep entries in a folder per customer named after the customer (or an alias). That is usually enough.
- Or add a **remote server** rule on the customer page.

With the Devolutions PowerShell module installed, folders and hosts are looked up exactly; without it, the tab name is used.

## What it reads

The name of the selected tab, entry names, folders and hosts, and which server a remote desktop window is connected to.

## Settings

| Setting | Default | What it does |
|---|---|---|
| Check every | `3` | How often the active session is checked while a remote desktop app is in front, in seconds. |
| Use the Devolutions PowerShell module | `on` | Look up folders and hosts of Remote Desktop Manager entries with the Devolutions.PowerShell module, when it is installed. |
| Refresh entries every | `15` | How often the list of Remote Desktop Manager entries is read again, in minutes. |
