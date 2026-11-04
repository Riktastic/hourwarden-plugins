# Releasing plugins

This repository holds two plugin stores side by side:

| Store | Plugins | Read by | Built with |
|---|---|---|---|
| `index.json` and `plugins/` at the top | PowerShell plugins (plugin API 1) | HourWarden 0.24 and older | `build-index.ps1` |
| `v2/` | WebAssembly plugins (plugin API 2) | HourWarden 0.25 and newer | `wasm/build-store.sh` |

An app that speaks API 2 uses `v2/` when a repository has one, so one address
serves both. Both indexes are signed with the same key,
`%USERPROFILE%\.hourwarden\store-signing.key`; the apps trust its public key
(`DEFAULT_KEY` in `hourwarden-core/src/repos.rs`).

## A WebAssembly plugin (v2)

1. Change the plugin in `wasm/<id>/`, and raise its version in **both**
   `wasm/<id>/Cargo.toml` and `wasm/<id>/plugin.toml` (they must match).
   Installed plugins are only updated to a higher version.
2. Test: `cargo test` in `wasm/` (the plugins' own logic). To run a built
   module the way the app does: `cargo run --example wasm_run -- <id>.wasm "A window title" <exe>`
   in `hourwarden-core` (`HW_TICKS`, `HW_FOLDERS` for tick-driven plugins).
3. Build and sign the store: `wasm/build-store.sh`. It compiles every plugin
   in the `hourwarden-linux` Docker image, copies each one's `plugin.toml`,
   `README.md` and `plugin.wasm` into `v2/plugins/<id>/`, writes
   `v2/index.json` with every file's SHA-256 and signs it (the key is mounted
   read-only and stays on this computer).
4. Commit `wasm/` and `v2/`, push `main`.

From then on:

- people who added the GitHub repository get the new version offered on the
  Plugins screen;
- the next app release bundles it, and installed plugins are updated to it
  on the app's first start.

The desktop app's test `every_webassembly_plugin_in_the_store_starts`
installs and starts every plugin from `v2/`; run it before an app release.

A plugin may only use the capabilities its `plugin.toml` lists
(`capabilities = [...]`); see the catalogue in
`hourwarden-core/docs/plugins/protocol-wasm.md`.

## A PowerShell plugin (old store)

Only for HourWarden 0.24 and older. `build-index.ps1` copies
`shared/protocol.ps1` into the plugins, writes `index.json` and signs it.
It runs `hourwarden-sign` with `cargo` from `hourwarden-core`.

Bitdefender deletes `.ps1` files in this repository: add an exception for the
folder, then `git restore .` to get them back. Never commit those deletions.

## Writing a plugin

`docs/creating-plugins.md` describes the PowerShell plugins (API 1). For
WebAssembly plugins, follow an existing one in `wasm/` and the protocol in
`hourwarden-core/docs/plugins/protocol-wasm.md`; the SDK (`wasm/sdk`) does the
plumbing.
