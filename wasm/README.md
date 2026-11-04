# WebAssembly plugins (plugin API 2)

These plugins run sandboxed inside the HourWarden app, so there is no hidden
script for antivirus to flag and third-party plugins are contained. See `docs/plugins/protocol-wasm.md` in `hourwarden-core`
for the protocol.

- `sdk/` — `hourwarden-plugin`, the SDK every plugin uses: it implements the
  host ABI so a plugin only writes its logic (`emit`, `log`, `capability`).
- `<id>/` — one crate per plugin, built to `<id>.wasm`.

## Build

```bash
rustup target add wasm32-unknown-unknown
cargo build --release --target wasm32-unknown-unknown
```

The modules land in `target/wasm32-unknown-unknown/release/<id>.wasm`.
`build-store.sh` builds them and copies each into `plugins/<id>/plugin.wasm`
of the store, records its hash in `index.json` and signs it.

## Test

A plugin's own logic is plain Rust and runs on the host (the SDK's host
functions are no-ops off wasm):

```bash
cargo test
```

To run a built module the way the app does, use the `wasm_run` example in
`hourwarden-core`:

```bash
cargo run --example wasm_run -- path/to/<id>.wasm "A window title" <exe>
```

## Capabilities

What the plugins read from the system they ask
the app for through capabilities (see the catalogue in
`docs/plugins/protocol-wasm.md` in `hourwarden-core`):

| Plugin | Capabilities |
|---|---|
| `admin-tools`, `remote-workspaces` | none (window title only) |
| `ticket-ids` | none (window title only) |
| `terminal-ssh` | `hostname`, `process.ssh_commands` |
| `networks` | `net.connections` |
| `vscode-git` | `vscode.workspaces`, `git.config` |
| `file-changes` | `files.changes` |
| `office-paths` | `office.document` |
| `web-addresses` | `browser.address` |
| `remote-desktop` | `ui.selected_tab`, `rdp.server` |
| `outlook` | `outlook.current_item`, `outlook.mail_since` |
| `outlook-calendar` | `outlook.calendar` |
| `teams-meetings` | `teams.state` |

`remote-desktop` takes folders from Remote Desktop Manager's tab names and
the server from the RDP connection.
