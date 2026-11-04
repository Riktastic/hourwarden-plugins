# HourWarden plugins

The plugin store for [HourWarden](../HourWarden). The app's **Plugins** screen lists what is in `index.json` here and installs plugins from this repository, checking every file's SHA-256 and the signature of `index.json`.

Plugins are WebAssembly modules that run sandboxed inside the app. They see the window in front, and only reach the rest of the system through the capabilities their `plugin.toml` lists.

HourWarden reads plugin repositories from GitHub (`https://github.com/Riktastic/hourwarden-plugins` by default), from any HTTPS address serving these files, or from a folder; a user can add several on the Plugins screen. The app downloads `index.json`, `index.json.sig` and the files the index lists, so a repository needs nothing more than this layout. A development build of HourWarden also adds this folder when it sits next to the HourWarden source.

| Plugin | What it adds | Works on |
|---|---|---|
| `teams-meetings` | Teams calls and meetings, also in the background; the meeting title decides the customer | Windows only |
| `outlook` | Subject and mail domains of the selected or open Outlook item; mail sent and received per customer | Windows only |
| `outlook-calendar` | Your appointments, which the daily review offers as time for the customer of the attendees or subject | Windows only |
| `office-paths` | Full path of the Word, Excel or PowerPoint document and the Explorer folder in front | Windows only |
| `remote-desktop` | Remote Desktop Manager folder and server; Remote Desktop Connection server | Windows only |
| `terminal-ssh` | The server of SSH sessions in Windows Terminal, PuTTY, Konsole, GNOME Terminal and other terminals | Windows and Linux |
| `admin-tools` | The server, database or domain in SQL Server Management Studio, WinSCP, FileZilla, Hyper-V and VMware consoles and management consoles (Active Directory, DNS) | Windows and Linux |
| `remote-workspaces` | The Citrix, Azure Virtual Desktop, Windows 365 or Horizon workplace you work in | Windows and Linux |
| `vscode-git` | The folder and git repository of the VS Code (or Cursor) window in front | Windows and Linux |
| `web-addresses` | The domain of the page in front in Edge and Chrome, matched like a mail domain; the customer's Microsoft tenant, Azure subscription and Power BI or Fabric workspace in Azure portal, Entra, Power BI and Fabric addresses | Windows only |
| `ticket-ids` | Ticket IDs (Jira, TopDesk, ServiceNow) in window titles, offered as the task | Windows and Linux |
| `networks` | Wi-Fi network or VPN, as weak evidence for the customer | Windows and Linux |
| `file-changes` | Files saved in customer folders | Windows and Linux |

## Layout

```text
wasm/<id>/              the source of each plugin (Rust)
wasm/sdk/               the SDK every plugin uses
wasm/build-store.sh     builds every plugin and the store below
index.json              the store's index, generated
index.json.sig          its Ed25519 signature
plugins/<id>/           one folder per plugin: plugin.toml, README.md, plugin.wasm
docs/creating-plugins.md  step-by-step guide to a new plugin
```

## Changing or adding a plugin

New to plugins? [docs/creating-plugins.md](docs/creating-plugins.md) builds one step by step. Change the source in `wasm/<id>/`, never the built copy in `plugins/`: `build-store.sh` overwrites it.

## Releasing plugins

[docs/releasing-the-store.md](docs/releasing-the-store.md): how a plugin update reaches people, and the signing key.
