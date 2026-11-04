"""Builds the plugin store at the top of this repository: compiles every
plugin in this workspace to WebAssembly, copies each one's plugin.toml,
README.md and module into plugins/<id>/, records every file's SHA-256 in
index.json and signs it.

Run it with build-store.sh, which does this in the hourwarden-linux Docker
image (with the Rust toolchain and the wasm32 target). Directly:

    python3 wasm/build_store.py [--key <store-signing.key>] [--core <hourwarden-core>]
"""
import argparse
import hashlib
import json
import re
import shutil
import subprocess
import tomllib
from pathlib import Path

WASM = Path(__file__).resolve().parent
ROOT = WASM.parent
STORE = ROOT


def sha256(path: Path) -> str:
    return hashlib.sha256(path.read_bytes()).hexdigest()


def members() -> list[str]:
    workspace = tomllib.loads((WASM / "Cargo.toml").read_text(encoding="utf-8"))
    return [m for m in workspace["workspace"]["members"] if m != "sdk"]


def build() -> Path:
    subprocess.run(["cargo", "build", "--release", "--target", "wasm32-unknown-unknown"], cwd=WASM, check=True)
    return WASM / "target" / "wasm32-unknown-unknown" / "release"


def plugin(crate: str, out: Path) -> dict:
    """Copies one plugin into the store; returns its index entry."""
    src = WASM / crate
    manifest = tomllib.loads((src / "plugin.toml").read_text(encoding="utf-8"))
    cargo = tomllib.loads((src / "Cargo.toml").read_text(encoding="utf-8"))
    pid, version = manifest["id"], manifest["version"]
    if pid != crate:
        raise SystemExit(f"{crate}: id '{pid}' must equal the folder name")
    if version != cargo["package"]["version"]:
        raise SystemExit(f"{crate}: plugin.toml says {version}, Cargo.toml {cargo['package']['version']}")
    if manifest.get("api") != 2 or manifest.get("run", {}).get("module") != "plugin.wasm":
        raise SystemExit(f"{crate}: plugin.toml must say api = 2 and [run] module = \"plugin.wasm\"")

    lib = cargo.get("lib", {}).get("name", crate.replace("-", "_"))
    target = STORE / "plugins" / pid
    target.mkdir(parents=True)
    shutil.copyfile(src / "plugin.toml", target / "plugin.toml")
    shutil.copyfile(src / "README.md", target / "README.md")
    shutil.copyfile(out / f"{lib}.wasm", target / "plugin.wasm")
    files = {f.name: sha256(f) for f in sorted(target.iterdir())}
    print(f"{pid} {version} ({(target / 'plugin.wasm').stat().st_size // 1024} KB)")
    return {"id": pid, "version": version, "path": f"plugins/{pid}", "files": files}


def sign(key: Path, core: Path) -> None:
    index = STORE / "index.json"
    signature = STORE / "index.json.sig"
    if not key.is_file():
        signature.unlink(missing_ok=True)
        print(f"Not signed: no key at {key}. HourWarden accepts this store only while no plugin store key is set.")
        return
    subprocess.run(
        ["cargo", "run", "-q", "--manifest-path", str(core / "Cargo.toml"), "-p", "hourwarden-core", "--bin", "hourwarden-sign", "--", "sign", str(key), str(index)],
        check=True,
    )


def main() -> None:
    args = argparse.ArgumentParser(description=__doc__.splitlines()[0])
    args.add_argument("--key", type=Path, default=Path.home() / ".hourwarden" / "store-signing.key")
    args.add_argument("--core", type=Path, default=ROOT.parent / "hourwarden-core")
    opts = args.parse_args()

    out = build()
    # Only the built plugins are replaced, never the rest of the repository.
    if (STORE / "plugins").exists():
        shutil.rmtree(STORE / "plugins")
    entries = [plugin(crate, out) for crate in sorted(members())]
    index = {"name": "HourWarden plugins", "api": 2, "plugins": entries}
    (STORE / "index.json").write_text(json.dumps(index, indent=2) + "\n", encoding="utf-8", newline="\n")
    print(f"index.json written with {len(entries)} plugins")
    sign(opts.key, opts.core)


if __name__ == "__main__":
    main()
