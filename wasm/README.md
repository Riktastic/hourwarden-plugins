# WebAssembly plugins (plugin API 2)

These plugins run sandboxed inside the HourWarden app instead of as PowerShell
processes, so there is no hidden script for antivirus to flag and third-party
plugins are contained. See `docs/plugins/protocol-wasm.md` in `hourwarden-core`
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
`build-index.ps1` copies each into `plugins/<id>/plugin.wasm` and records its
hash, the same as for the PowerShell plugins.

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

## Migration status

Ported to API 2 (6 of 13): `ticket-ids`, `terminal-ssh`, `remote-workspaces`,
`admin-tools`, `networks`, `vscode-git`. What they read from the system they
ask the app for through capabilities (see the catalogue in
`docs/plugins/protocol-wasm.md` in `hourwarden-core`).

Still PowerShell (API 1) until the Windows capabilities they need exist in
the desktop app: `office-paths`, `web-addresses`, `remote-desktop`, `outlook`,
`outlook-calendar`, `teams-meetings` (COM and UI Automation) and
`file-changes` (folder watching). On Windows, `networks` and the `ssh`
fallback of `terminal-ssh` also wait for the app's Windows implementation of
`net.connections` and `process.ssh_commands`.
