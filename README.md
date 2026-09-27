# Self-Signed Certificate Server

This repository packages the Yew application from `mailvibi/selfsignedcert` at
commit `6a2624f54e335460d79d1ef5742aac48164f0e26` into a single Axum binary.
The frontend is built with Trunk, validated into `target/frontend-assets`, and
embedded by Cargo. `build.rs` never invokes the frontend build and fails if the
staged manifest or any asset is missing, changed, or unexpected.

## Build

Install the build-only tools once:

```sh
rustup target add wasm32-unknown-unknown
cargo install trunk --locked
```

From the repository root, the only release-owner command is:

```sh
./scripts/release.sh
```

It produces `target/release/selfsignedcert-server`. Python 3 is used only to
write the deterministic asset manifest. The deployment machine needs only the
resulting executable.

## Run

```sh
./target/release/selfsignedcert-server --listen=0.0.0.0 --port=8080
```

`--listen` accepts an IP address, not a hostname, and defaults to
`127.0.0.1`. `--port` is a `u16` and defaults to `8080`. `--verbose` enables
structured HTTP request tracing. `/health` returns uncached `OK`; unknown and
ambiguous paths return `404`.

The server is plain HTTP. Use a TLS-terminating reverse proxy for HTTPS and
ensure the proxy and Axum agree on HTTP version, header limits, and framing.
Binding to `0.0.0.0` exposes the service on every IPv4 interface; it provides
no authentication or TLS.

## Validation

The Rust route tests can run with a staged test directory:

```sh
SELF_SIGNED_CERT_ASSETS="$PWD/target/frontend-assets" cargo test --locked
```

The release smoke test must copy only the binary to a clean directory and
verify `/`, `/health`, the actual JavaScript and WASM URLs, and traversal
variants. No runtime frontend directory is read.