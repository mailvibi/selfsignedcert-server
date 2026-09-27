#!/usr/bin/env bash
set -euo pipefail

ROOT=$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)
STAGING="$ROOT/target/frontend-assets"
mkdir -p "$ROOT/target"
TMP=$(mktemp -d "$ROOT/target/frontend-assets.XXXXXX")
trap 'rm -rf "$TMP"' EXIT

command -v trunk >/dev/null || { echo "error: trunk is required (cargo install trunk --locked)" >&2; exit 1; }
rustup target list --installed | grep -qx wasm32-unknown-unknown || { echo "error: install wasm32-unknown-unknown with rustup" >&2; exit 1; }
trunk build --release --config "$ROOT/frontend/Trunk.toml" --dist "$TMP/dist" --public-url "/"

python3 - "$TMP/dist" "$TMP/manifest.json" <<'PY'
import hashlib, json, mimetypes, pathlib, sys
root = pathlib.Path(sys.argv[1]); output = pathlib.Path(sys.argv[2])
files = sorted(p for p in root.rglob('*') if p.is_file())
if not files: raise SystemExit('error: Trunk produced no assets')
entries = []
for path in files:
    relative = path.relative_to(root).as_posix()
    if relative.endswith('.map') and relative != 'index.html': raise SystemExit(f'error: unexpected source map: {relative}')
    data = path.read_bytes()
    mime = 'application/wasm' if path.suffix == '.wasm' else (mimetypes.guess_type(path.name)[0] or 'application/octet-stream')
    entries.append({'url': '/' + relative, 'file': relative, 'bytes': len(data), 'sha256': hashlib.sha256(data).hexdigest(), 'mime': mime})
if not any(e['url'] == '/index.html' for e in entries): raise SystemExit('error: index.html missing')
if not any(e['mime'] == 'application/wasm' for e in entries): raise SystemExit('error: WASM asset missing')
output.write_text(json.dumps({'version': 1, 'assets': entries}, indent=2) + '\n')
PY
cp -a "$TMP/dist/." "$TMP/"
rm -rf "$TMP/dist"
rm -rf "$STAGING"
mv "$TMP" "$STAGING"
trap - EXIT
SELF_SIGNED_CERT_ASSETS="$STAGING" cargo build --release --locked
echo "built target/release/selfsignedcert-server"