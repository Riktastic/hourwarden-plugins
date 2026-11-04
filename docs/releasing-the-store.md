# Releasing plugins

The store is `index.json`, its signature and `plugins/<id>/` at the top of
this repository, built from the source in `wasm/`. `index.json` is signed
with `%USERPROFILE%\.hourwarden\store-signing.key`; the apps trust its public
key (`DEFAULT_KEY` in `hourwarden-core/src/repos.rs`).

Plugins are WebAssembly (plugin API 2). The PowerShell plugins (API 1) were
removed in November 2026: HourWarden 0.24 and older find no plugins they can
run here, and tell their users to update the app.

## Steps

1. Change the plugin in `wasm/<id>/`, and raise its version in **both**
   `wasm/<id>/Cargo.toml` and `wasm/<id>/plugin.toml` (they must match).
   Installed plugins are only updated to a higher version.
2. Test: `cargo test` in `wasm/` (the plugins' own logic). To run a built
   module the way the app does: `cargo run --example wasm_run -- <id>.wasm "A window title" <exe>`
   in `hourwarden-core` (`HW_TICKS`, `HW_FOLDERS` for tick-driven plugins).
3. Build and sign the store: `wasm/build-store.sh`. It compiles every plugin
   in the `hourwarden-linux` Docker image, copies each one's `plugin.toml`,
   `README.md` and `plugin.wasm` into `plugins/<id>/`, writes `index.json`
   with every file's SHA-256 and signs it (the key is mounted read-only and
   stays on this computer).
4. Commit `wasm/`, `plugins/`, `index.json` and `index.json.sig`, push `main`.

From then on:

- people who added the GitHub repository get the new version offered on the
  Plugins screen;
- the next app release bundles it, and installed plugins are updated to it
  on the app's first start.

The desktop app's test `every_webassembly_plugin_in_the_store_starts`
installs and starts every plugin from this store; run it before an app
release.

A plugin may only use the capabilities its `plugin.toml` lists
(`capabilities = [...]`); see the catalogue in
`hourwarden-core/docs/plugins/protocol-wasm.md`.

## Writing a plugin

[creating-plugins.md](creating-plugins.md).
