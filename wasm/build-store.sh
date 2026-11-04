#!/usr/bin/env bash
# Builds the plugin store in the hourwarden-linux Docker image: compiles the
# plugins to WebAssembly, writes plugins/ and index.json and signs it with
# the key in ~/.hourwarden (mounted read-only; it never leaves this computer).
#
#   wasm/build-store.sh
#
# Expects hourwarden-core next to this repository, for hourwarden-sign.
set -euo pipefail
here=$(cd "$(dirname "$0")/.." && pwd)
core=$(cd "$here/../hourwarden-core" && pwd)
keys="${HOURWARDEN_KEYS:-$HOME/.hourwarden}"

docker run --rm \
    -v "$here:/work" -v "$core:/core" -v "$keys:/keys:ro" \
    -v hourwarden-cargo:/cargo -v hourwarden-wasm-target:/work/wasm/target -v hourwarden-linux-target:/core/target \
    -e CARGO_HOME=/cargo -w /work \
    hourwarden-linux bash -c '
        export PATH=/root/.cargo/bin:$PATH
        rustup target add wasm32-unknown-unknown >/dev/null 2>&1
        python3 wasm/build_store.py --key /keys/store-signing.key --core /core'
