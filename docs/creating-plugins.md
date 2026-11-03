# Creating a plugin

This guide walks you through a new plugin from an empty folder to an update in the app's store. [protocol.md](protocol.md) is the reference for every message and manifest field; this page is the how-to.

## What a plugin is

A plugin is a small program that HourWarden starts in the background. It tells HourWarden things the app cannot see by itself, such as the document open in Word, the server of an SSH session or a Teams meeting in progress. HourWarden does the rest: it weighs that information with everything else and picks the customer.

- It talks over **stdin and stdout**, one line of JSON per message. Anything it writes to **stderr** goes to the plugin's log in the app.
- It runs **without a window**, with its own folder as the working directory.
- It can be written in **any language**. The plugins in this repository use PowerShell (5.1 on Windows, 7 on Linux) because it is on every Windows computer and reaches COM, UI Automation and the registry. `shared/protocol.ps1` handles the protocol for you.
- A plugin only **adds information**. It cannot change entries, read the database or send anything to the server.

## 1. Pick what it adds

Choose the message that fits what you know (see the table in [protocol.md](protocol.md#from-the-plugin-to-hourwarden)):

| You know... | Send |
|---|---|
| the file, folder, mail domain, remote server or ticket of the window in front | `detail` |
| the user is in a meeting | `meeting` |
| appointments | `calendar` |
| the network or VPN | `network` |
| a file was saved | `file_changed` |
| something to count per customer, such as mails sent | `metric` |

Most plugins send `detail` for the window in front. That is what the example below does.

## 2. Make the folder

```text
plugins/notepad-files/
  plugin.toml
  notepad-files.ps1
  README.md
```

The folder name is the plugin's `id`: lower case letters, digits and `-`.

### plugin.toml

```toml
id = "notepad-files"
name = "Files open in Notepad"
version = "0.1.0"
author = "Your name"
category = "Documents"
description = "Reports the file open in Notepad, so its folder can decide the customer."
docs = "README.md"
api = 1
platforms = ["windows"]
permissions = [
    "Reads the title of the window in front",
]

[run]
command = "powershell"
args = ["-NoProfile", "-NonInteractive", "-ExecutionPolicy", "Bypass", "-File", "notepad-files.ps1"]

[[settings]]
key = "poll_seconds"
label = "Check every"
type = "number"
description = "How often the window is checked, in seconds."
default = 2
min = 1
max = 30
```

- `permissions` is shown to the user before installing. Say plainly what the plugin reads, such as "Reads your Outlook calendar".
- `platforms` lists `windows`, `linux` or both. Use only `windows` when the plugin needs COM, Office, UI Automation or the registry. On Linux use `command = "pwsh"`.
- Each `[[settings]]` entry becomes a field on the Plugins screen. `type` is `number`, `bool`, `text` or `list`.

### notepad-files.ps1

```powershell
# Files open in Notepad: reports the document of the Notepad window in front.
# Only reads the window title; nothing leaves the computer.

. "$PSScriptRoot/protocol.ps1"

# "notes.txt - Notepad" -> "notes.txt"; $null when the title has no file.
function Get-NotepadDocument([string]$Title) {
    if ($Title -match '^\*?(.+?) - Notepad$' -and $Matches[1] -ne 'Untitled') { return $Matches[1] }
    return $null
}

if ($script:testMode) { return }   # tests load the functions above and stop here
Wait-Initialize
Send-Status 'running' 'Watching Notepad'

while ($true) {
    [void](Receive-Messages)        # takes in foreground, settings, folders and shutdown
    $fg = $script:foreground
    if ($fg -and (Test-App $fg.exe @('notepad.exe'))) {
        $doc = Get-NotepadDocument $fg.title
        # Details count for 30 seconds, so send them again while the window stays in front.
        if ($doc) { Send-Message 'detail' @{ hwnd = $fg.hwnd; exe = $fg.exe; document = $doc } }
    }
    Wait-ForHost ([int](Get-Setting 'poll_seconds' 2) * 1000)
}
```

`protocol.ps1` gives you:

| Function or variable | Does |
|---|---|
| `Wait-Initialize` | Waits for `initialize` and stores the settings and customer folders |
| `Receive-Messages` | Handles waiting messages; exits the plugin on `shutdown` or when the app is gone |
| `$script:foreground` | The window in front: `hwnd`, `pid`, `exe` (lower case), `title` |
| `$script:folders` | The customer and project folders |
| `Get-Setting key default` | A setting's current value |
| `Test-App exe names` | Whether the window belongs to one of these programs; `.exe` is optional, so it works on Linux too |
| `Send-Message method params` | Sends any message from protocol.md |
| `Send-Status state message` | `running`, `degraded` (part of it does not work; say why) or `failing` |
| `Send-Log message` | A line in the plugin's log |
| `Wait-ForHost ms` | Sleeps, but wakes for messages from the app |

Keep plugin files plain ASCII: Windows PowerShell 5.1 reads files without a byte order mark as ANSI.

### README.md

The app shows this in its help screen. Start with a `# Title` and a **Works on** line, then say what the plugin does, how to set it up, what it reads, and what each setting means.

```markdown
# Files open in Notepad

**Works on:** Windows only

Reports the file open in Notepad, so the customer folder it is in decides the customer.

It only reads the title of the window in front.

## Settings

- **Check every**: how often the window is checked, in seconds.
```

## 3. Test it

Write `test/notepad-files.tests.ps1`. `Use-Plugin` loads your script in test mode, so only the functions run:

```powershell
. "$PSScriptRoot/TestKit.ps1"
. (Use-Plugin 'notepad-files')

Test 'file name from the title' {
    Assert-Equal 'notes.txt' (Get-NotepadDocument 'notes.txt - Notepad')
    Assert-Equal 'notes.txt' (Get-NotepadDocument '*notes.txt - Notepad')
}

Test 'no file yet' {
    Assert-Equal $null (Get-NotepadDocument 'Untitled - Notepad')
}

Complete-Tests
```

Run all tests. They fail when a plugin has no test file or no documentation, or when `platforms` and the **Works on** line disagree.

```powershell
powershell -ExecutionPolicy Bypass -File test\Run-Tests.ps1
```

Then run the plugin the way the app does and watch what it sends:

```powershell
powershell -ExecutionPolicy Bypass -File test\Test-Plugin.ps1 -Plugin notepad-files -Seconds 10 `
    -Foreground '{"hwnd":1,"pid":0,"exe":"notepad.exe","title":"notes.txt - Notepad"}'
```

## 4. Try it in the app

On the **Plugins** screen, choose **Install from folder** and pick `plugins/notepad-files`. It shows up as a local plugin with its log (**Show log**) and settings. Change the script, then install from the folder again to load the change.

## 5. Publish it

1. Raise `version` in `plugin.toml` for every change, or the store offers no update.
2. Run `powershell -ExecutionPolicy Bypass -File build-index.ps1`. It copies `shared/protocol.ps1` into your plugin, writes the SHA-256 of every file to `index.json` and signs it.
3. Commit and push. Installed apps see the new plugin or update in the store.

HourWarden checks every file against `index.json` when installing and before each start, so a changed file never runs without being reinstalled.

### Your own repository

You don't need this repository to share plugins. Copy its layout (`plugins/`, `shared/`, `build-index.ps1`), make your own signing key, and push it to GitHub or any HTTPS address. Users add it on the Plugins screen and trust your key once.

### Contributing here

Pull requests are welcome. Plugins in this store need a test file and a README, must keep everything on the computer (no network calls of their own), and are published under the MIT licence of this repository.

## Tips

- **Read as little as you need.** Prefer the window title over reading a document's contents, and say exactly what you read in `permissions` and the README.
- **Fail softly.** When something you depend on is missing, for example Outlook is not installed, send `degraded` with the reason and keep running, instead of exiting.
- **Stay light.** Poll every few seconds at most, and only do work when the window in front belongs to your program.
- **Repeat what expires.** `detail` lasts 30 seconds, `meeting` 20 seconds and `network` 3 minutes; `calendar` should be sent again every hour.
