# Creating a plugin

This guide takes a new plugin from an empty folder to an update in the app's
store. The reference for every message, manifest field and capability is
`docs/plugins/protocol-wasm.md` in `hourwarden-core`; this page is the how-to.

## What a plugin is

A plugin is a small WebAssembly module that HourWarden runs inside the app. It
tells HourWarden things the app cannot see by itself, such as the document
open in Word, the server of an SSH session or a Teams meeting in progress.
HourWarden does the rest: it weighs that information with everything else
and picks the customer.

- It is written in Rust with the SDK in `wasm/sdk` (`hourwarden-plugin`),
  which handles the plumbing.
- It runs **sandboxed**: no files, no network, no processes of its own. It
  sees the window in front, its settings and the customer folders, and
  everything else it asks the app for through **capabilities** that its
  `plugin.toml` lists (for example `outlook.current_item`). Any other request
  is refused.
- Each call has a fixed budget; a plugin that loops is stopped and restarted.
- A plugin only **adds information**. It cannot change entries, read the
  database or send anything to the server.

## 1. Pick what it adds

| You know... | Send |
|---|---|
| the file, folder, mail domain, remote server or ticket of the window in front | `detail` |
| the user is in a meeting | `meeting` |
| appointments | `calendar` |
| the network or VPN | `network` |
| a file was saved | `file_changed` |
| something to count per customer, such as mails sent | `metric` |

Most plugins send `detail` for the window in front. The example below does.

## 2. Make the crate

```text
wasm/notepad-files/
  Cargo.toml
  plugin.toml
  README.md
  src/lib.rs
```

The folder name is the plugin's `id`: lower case letters, digits and `-`. Add
it to `members` in `wasm/Cargo.toml`.

### Cargo.toml

```toml
[package]
name = "notepad-files"
version = "0.1.0"
edition = "2021"

[lib]
name = "notepad_files"
crate-type = ["cdylib"]

[dependencies]
hourwarden-plugin = { path = "../sdk" }
serde_json = "1"
```

### plugin.toml

```toml
id = "notepad-files"
name = "Files open in Notepad"
version = "0.1.0"
author = "Your name"
category = "Documents"
description = "Reports the file open in Notepad, so its folder can decide the customer."
docs = "README.md"
api = 2
capabilities = []
platforms = ["windows"]
permissions = [
    "Reads the title of the window in front",
]

[run]
module = "plugin.wasm"

[[settings]]
key = "poll_seconds"
label = "Check every"
type = "number"
description = "How often the window is checked, in seconds."
default = 2
min = 1
max = 30
```

- `version` must be the same here and in `Cargo.toml`.
- `capabilities` lists what the plugin may ask the app for; leave it empty
  when the window title is enough.
- `permissions` is shown to the user before installing. Say plainly what the
  plugin reads, such as "Reads your Outlook calendar".
- `platforms` lists `windows`, `linux` or both. A capability that only exists
  on Windows (Outlook, Office, UI Automation) makes the plugin Windows only.
- Each `[[settings]]` entry becomes a field on the Plugins screen. `type` is
  `number`, `bool`, `text` or `list`.

### src/lib.rs

```rust
//! Files open in Notepad: reports the document of the Notepad window in front.

#![allow(static_mut_refs)]

use hourwarden_plugin as hw;
use hw::{json, Value};

/// "notes.txt - Notepad" -> "notes.txt"; None when the title has no file.
fn notepad_document(title: &str) -> Option<&str> {
    let doc = title.strip_suffix(" - Notepad")?.trim_start_matches('*');
    (doc != "Untitled").then_some(doc)
}

static mut POLL: hw::ForegroundPoll = hw::ForegroundPoll::new();
static mut POLL_MS: i64 = 2000;

fn on_init(data: Value) {
    unsafe { POLL_MS = hw::poll_ms(&data["settings"], 2) };
    hw::status("running", "Watching Notepad");
}

fn on_event(event: Value) {
    if event["kind"] == "settings" {
        unsafe { POLL_MS = hw::poll_ms(&event["settings"], 2) };
    } else if let Some(fg) = unsafe { POLL.due(&event, POLL_MS) } {
        // Details count for 30 seconds; ForegroundPoll repeats them while the window stays in front.
        if hw::app_name(fg["exe"].as_str().unwrap_or("")) == "notepad" {
            if let Some(doc) = notepad_document(fg["title"].as_str().unwrap_or("")) {
                hw::emit(&json!({ "method": "detail", "params": { "hwnd": fg["hwnd"], "exe": fg["exe"], "document": doc } }));
            }
        }
    }
}

hw::plugin!(event = on_event, init = on_init);

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn file_name_from_the_title() {
        assert_eq!(notepad_document("notes.txt - Notepad"), Some("notes.txt"));
        assert_eq!(notepad_document("*notes.txt - Notepad"), Some("notes.txt"));
        assert_eq!(notepad_document("Untitled - Notepad"), None);
    }
}
```

The SDK gives you:

| Function | Does |
|---|---|
| `plugin!(event = .., init = ..)` | Exports the entry points the app calls |
| `emit(&json!(..))` | Sends a message (`detail`, `meeting`, ...) |
| `status(state, message)` | `running`, `degraded` (part of it does not work; say why) or `failing` |
| `log(message)` | A line in the plugin's log in the app |
| `capability(name, &arg)` | Asks the app for something its manifest lists; `None` when refused or unavailable |
| `ForegroundPoll` | Hands you the window in front every `poll_seconds` |
| `poll_ms`, `setting_i64`, `string_list` | Read settings |
| `app_name`, `mail_domains` | Small helpers |

### README.md

The app shows this in its help screen. Start with a `# Title` and a
**Works on** line, then say what the plugin does, how to set it up, what it
reads, and what each setting means.

## 3. Test it

The plugin's own logic is plain Rust and runs on your computer:

```bash
cd wasm && cargo test -p notepad-files
```

Then run the built module the way the app does, with the `wasm_run` example
in `hourwarden-core`:

```bash
cargo build --release --target wasm32-unknown-unknown -p notepad-files
cargo run --example wasm_run -- notepad_files.wasm "notes.txt - Notepad" notepad.exe
```

## 4. Try it in the app

Copy `plugin.toml`, `README.md` and the built module (renamed to
`plugin.wasm`) into a folder. On the **Plugins** screen, choose **Install
from folder** and pick it. It shows up as a local plugin with its log
(**Show log**) and settings.

## 5. Publish it

1. Raise `version` in `plugin.toml` and `Cargo.toml` for every change, or the
   store offers no update.
2. Run `wasm/build-store.sh`. It builds every plugin, copies it into
   `plugins/<id>/`, writes the SHA-256 of every file to `index.json` and signs
   it.
3. Commit and push. Installed apps see the new plugin or update in the store.

HourWarden checks every file against `index.json` when installing and before
each start, so a changed file never runs without being reinstalled.

### Your own repository

You don't need this repository to share plugins. Copy its layout (`wasm/`,
`plugins/`, `index.json`), make your own signing key, and push it to GitHub or
any HTTPS address. Users add it on the Plugins screen and trust your key once.

### Contributing here

Pull requests are welcome. Plugins in this store need tests and a README, and
are published under the MIT licence of this repository.

## Tips

- **Read as little as you need.** Prefer the window title over a capability,
  and say exactly what you read in `permissions` and the README.
- **Fail softly.** When a capability answers `None` (Outlook is not
  installed, say), send `degraded` with the reason and keep running.
- **Repeat what expires.** `detail` lasts 30 seconds, `meeting` 20 seconds and
  `network` 3 minutes; `calendar` should be sent again every hour.
