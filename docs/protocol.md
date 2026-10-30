# Plugin protocol (API 1)

A plugin is a folder with a `plugin.toml` manifest and the files it runs. Timekeeper starts the command in `[run]` with the plugin folder as working directory, without a window, and talks to it over stdin and stdout.

Every message is one line of JSON, a JSON-RPC 2.0 notification:

```json
{"jsonrpc":"2.0","method":"status","params":{"state":"running","message":"Watching the microphone"}}
```

Anything the plugin writes to stderr ends up in the plugin's log in the app.

## From Timekeeper to the plugin

| Method | Params | When |
|---|---|---|
| `initialize` | `api`, `host_pid`, `settings` (the plugin's own settings), `customer_folders` | Right after start |
| `foreground` | `hwnd`, `pid`, `exe` (lower case), `title` | When the window in front or its title changes |
| `settings` | the plugin's settings | When the user changes them |
| `folders` | `customer_folders` | When customer or project folders change |
| `shutdown` | | Before the plugin is stopped |

A plugin must exit when stdin closes, on `shutdown`, or when the process `host_pid` no longer exists. `shared/protocol.ps1` does all of this for PowerShell plugins.

## From the plugin to Timekeeper

| Method | Params | Effect |
|---|---|---|
| `status` | `state`: `running`, `degraded` or `failing`; `message` | Shown in the plugin store. Send `degraded` with a clear reason when part of the plugin does not work. |
| `detail` | `hwnd`, `exe`, optional `path`, `document`, `mail_domains` (list), `remote_server`, `remote_folder` | Adds information about that window. Used for 30 seconds, so repeat it now and then while it stays the same. |
| `meeting` | `active` (bool), `title` | A meeting for a known customer takes the time while it runs, also when you do not type. Repeat `active: true` at least every 20 seconds. |
| `network` | `name`, `kind` (`wifi`, `wired`, `vpn`) | The current network, used only when nothing else decides. Repeat at least every 3 minutes. |
| `file_changed` | `path`, `change` | A file saved in a customer folder; supports attribution for one minute. |
| `metric` | `metric` (a-z and `_`, such as `mail_sent` or `mail_received`), `key` (unique per event), `at` (milliseconds since 1970), `domains` (list) | Counts one event for the customers with those mail domains. The same key counts only once, so it is safe to report an event again after a restart. Shown on the customer page and in reports. |
| `log` | `message` | Written to the plugin's log. |

How Timekeeper weighs the information: manual switch first, then customer folders, rules, mail domains, names and abbreviations (in titles, paths, Remote Desktop Manager folders and server names), recently saved files, and the network last.

## Manifest

```toml
id = "my-plugin"          # a-z, 0-9 and -; equals the folder name
name = "My plugin"
version = "0.1.0"         # raise it for every change; the store offers updates
author = "You"
category = "Documents"
description = "One or two sentences shown in the store."
api = 1
platforms = ["windows"]   # and/or "linux"
permissions = ["What it reads, in plain words", "Runs a PowerShell script"]

[run]
command = "powershell"    # Windows PowerShell 5.1; "pwsh", or a file in the plugin folder
args = ["-NoProfile", "-NonInteractive", "-ExecutionPolicy", "Bypass", "-File", "my-plugin.ps1"]

[[settings]]              # shown as a form in the store; values arrive in initialize/settings
key = "poll_seconds"
label = "Check every"
type = "number"           # number, bool, text or list
description = "Help text."
default = 5
min = 1
max = 60
```

## Publishing

1. Put the plugin in `plugins/<id>/`.
2. Run `build-index.ps1`. It copies `shared/protocol.ps1` into plugins that use it and writes the SHA-256 of every file to `index.json`.
3. Commit. Timekeeper checks every file against `index.json` when installing, and again before each start, so a changed script is never run without being reinstalled.

Test a plugin without the app:

```powershell
powershell -ExecutionPolicy Bypass -File test\Test-Plugin.ps1 -Plugin networks -Seconds 10
```

To try a plugin in the app before publishing, use **Install from folder** in the plugin store. It is marked as a local plugin.
