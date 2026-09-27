# Copilot Prompt — Self-Contained Axum + WASM Single-Binary Deployment Plan

You are a senior Rust systems engineer and build/release engineer.

I need you to produce a **hyper-explicit, implementation-ready engineering plan** for building a Rust/Axum single-binary web server that embeds and serves the deployable web assets produced by this WebAssembly project:

https://github.com/mailvibi/selfsignedcert

## Primary goal

Create a Rust-based web server using **Axum** whose final deployment artifact is exactly one executable binary.

The intended production deployment must be as simple as:

```
./<webserver-binary> --listen=<listen-address> --port=<port-number>
```

with an optional:

```
--verbose
```

or:

```
--debug
```
flag.

There must be **no requirement to install Node.js, npm, a Rust toolchain, wasm-bindgen, wasm-pack, Trunk, a web server, or any other runtime/build dependency on the deployment machine**.

The final executable must contain all deployable runtime assets required to serve the WebAssembly application.

The deployment machine should therefore need only:

```
<webserver-binary>
```

and nothing else.

The server should behave conceptually like:

```
                  BUILD TIME
                     │
                     ▼
        ┌──────────────────────────┐
        │ selfsignedcert WASM app  │
        │ HTML / JS / WASM / CSS   │
        │ other runtime assets     │
        └────────────┬─────────────┘
                     │
                     │ embed into binary
                     ▼
        ┌──────────────────────────┐
        │ Rust Axum server binary  │
        │                          │
        │ embedded static assets   │
        │ HTTP routing             │
        │ CLI configuration        │
        └────────────┬─────────────┘
                     │
                     ▼
              single executable
                     │
                     ▼
       ./server --listen=... --port=...
```

---

# 1. Inspect the actual repository first

Before proposing the implementation, inspect the actual contents and build system of:

https://github.com/mailvibi/selfsignedcert

Determine precisely:

1. What Rust/WASM framework it uses.
2. Whether it uses:

- `wasm-bindgen`
- `wasm-pack`
- `Trunk`
- `wasm32-unknown-unknown`
- `wasm32-wasip1`
- another mechanism.
3. Its Cargo workspace structure.
4. Its `Cargo.toml` files.
5. Its build scripts, if any.
6. Its frontend/static assets.
7. Its generated WASM/JS requirements.
8. Whether generated output contains:

- `.wasm`
- JavaScript glue
- HTML
- CSS
- images
- fonts
- JSON
- source maps
- other files.
9. How the project is currently built and served.
10. What files are actually required at runtime.
11. Whether the project currently assumes a particular URL/base path.
12. Whether it uses browser APIs that impose special HTTP requirements.
13. Whether WASM MIME types or special headers are required.
14. Whether client-side routing/fallback behavior is required.
15. Whether the application uses `fetch()` or other relative/absolute resource URLs.
16. Whether the application requires:

- HTTPS
- COOP
- COEP
- CORS
- CSP
- other special headers.

17. Whether generated files contain build-time absolute paths or environment-specific information.
18. Which parts of the repository are source-only and which are deployable runtime artifacts.

**Do not assume the technology stack from the repository name. Verify it from the actual repository.**

Inspect a pinned commit, tag, or release revision, and record its immutable commit SHA in the plan. Do not use an unqualified default branch as the source of repository facts.

If the repository cannot be accessed, stop before proposing implementation details. State that repository-specific findings are unavailable and provide only a discovery checklist and a clearly marked provisional architecture. Do not invent filenames, commands, framework details, generated asset names, or source code.

The repository findings must include the exact revision inspected, the commands used to inspect it, and a table mapping each browser-requestable runtime URL to the source/generated file that supplies it.

---

# 2. Desired architecture

Design a solution where the repository contains a dedicated server component, for example:

```
/
├── <existing selfsignedcert project>
├── server/
│   ├── Cargo.toml
│   ├── src/
│   │   └── main.rs
│   └── build.rs
└── ...
```

The exact structure may differ if there is a better approach.

The important requirement is:

> **The final server executable must contain the complete deployable WASM frontend.**

For example, conceptually:

```
static INDEX_HTML: &[u8] = include_bytes!(...);
static APP_WASM: &[u8] = include_bytes!(...);
static APP_JS: &[u8] = include_bytes!(...);
```

However, do **not** blindly use this exact mechanism if another mechanism is more appropriate.

Evaluate alternatives such as:

- `include_bytes!`
- `rust-embed`
- `include_dir`
- generated Rust source containing assets
- a build script that packages generated assets
- embedding a generated tar/archive/blob
- another suitable compile-time embedding mechanism.

Evaluate each based on:

- simplicity
- reliability
- binary size
- startup performance
- request performance
- maintainability
- reproducibility
- cross-compilation
- ease of debugging
- ease of upgrading the WASM frontend.

Then select one and explain why.

---

# 3. Critical build requirement

Establish a clean distinction between three environments.

## 3.1 Development

Developers can use whatever normal Rust/WASM tooling is appropriate to develop the frontend.

## 3.2 Release/build environment

The build environment:

1. Builds the WASM frontend.
2. Produces the deployable frontend assets.
3. Validates those assets.
4. Embeds them into the Axum server.
5. Produces the final release binary.

## 3.3 Runtime/deployment environment

The runtime environment receives only:

```
server-binary
```

It must not need:

```
index.html
app.wasm
app.js
pkg/
dist/
static/
node_modules/
Cargo.toml
```

or any other frontend artifact.

---

# 4. CLI requirements

Design the command-line interface around:

```
./selfsignedcert-server \
    --listen=0.0.0.0 \
    --port=8080
```

Optional verbosity/debugging:

```
./selfsignedcert-server \
    --listen=0.0.0.0 \
    --port=8080 \
    --verbose
```

or, if you determine a better interface:

```
./selfsignedcert-server \
    --listen=0.0.0.0 \
    --port=8080 \
    --debug
```

Prefer idiomatic Rust CLI parsing such as `clap`, unless there is a compelling reason not to.

The implementation must make one concrete CLI decision rather than leaving alternatives for the implementer. Use `--verbose` as the only verbosity flag unless repository constraints require otherwise; do not implement both `--verbose` and `--debug` without documenting their distinct behavior.

Define:

- default listen address, if any
- default port, if any
- whether `--listen` and `--port` are mandatory
- accepted address formats
- IPv4 behavior
- IPv6 behavior
- invalid port handling
- startup error behavior
- logging behavior
- exit codes
- help output.

For the primary interface, use `--listen=<IP address>` and `--port=<u16>` with explicit documented defaults or explicitly mark both as required. If `listen` is typed as `IpAddr`, state that hostnames are intentionally rejected. Invalid ports, invalid addresses, unknown arguments, help, and bind failures must each have defined exit behavior and user-facing diagnostics.

Avoid unnecessary configuration files or environment variables unless there is a strong reason to include them.

The primary deployment interface should remain:

```
./selfsignedcert-server --listen=0.0.0.0 --port=8080
```

---

# 5. HTTP server requirements

Use **Axum**.

Specify precisely:

1. Axum version.
2. Tokio configuration.
3. Router structure.
4. How static assets are represented.
5. How assets are selected by URL.
6. How MIME types are determined.
7. What happens for `/`.
8. What happens for `/index.html`.
9. What happens for the actual WASM path.
10. What happens for JavaScript files.
11. What happens for CSS.
12. What happens for other assets.
13. What happens for unknown paths.
14. Whether SPA fallback is required.
15. Whether redirects are required.
16. Cache-Control behavior.
17. Content-Encoding behavior, if applicable.
18. Range requests, if relevant.
19. HEAD requests.
20. 404 behavior.
21. 500 behavior.
22. Security-related HTTP headers that are appropriate.
23. Whether CORS is needed.
24. Whether CSP is appropriate.
25. Whether WASM requires any special response headers.

Do not add middleware merely because it is available.

Explain why every nontrivial middleware component exists.

---

# 6. WASM-specific requirements

Pay particular attention to serving `.wasm` correctly.

Determine whether:

```
Content-Type: application/wasm
```

is required and whether Axum's chosen serving mechanism provides it automatically.

Inspect the actual generated frontend to determine whether the browser loads:

```
foo.wasm
```
directly, through JavaScript glue, or through another mechanism.

Trace the complete browser startup sequence:

```
GET /
  ↓
index.html
  ↓
JavaScript
  ↓
WASM
  ↓
other assets
```

Identify every request the browser can make during normal startup.

Ensure the embedded server can satisfy every one of those requests.

---

# 7. Build pipeline

Design the exact build pipeline:

```
clean checkout
      │
      ▼
build WASM frontend
      │
      ▼
produce deployable frontend directory
      │
      ▼
validate expected files
      │
      ▼
embed assets into Axum binary
      │
      ▼
build release binary
      │
      ▼
single deployable executable
```

Use one explicit build-owner model and state it before giving commands. Unless repository inspection proves an existing equivalent is better, use this model:

1. A top-level release command or script builds the frontend and produces a clean, dedicated asset staging directory.
2. The frontend build is never invoked recursively from the server's `build.rs`.
3. The staging directory contains only deployable runtime assets plus a generated manifest.
4. The server build consumes that staging directory, validates it, and embeds it. It fails closed if the directory, manifest, or required assets are missing.
5. `build.rs` must not silently reuse stale output. It must use explicit input/output paths and emit precise Cargo rerun directives for the manifest and every staged asset, or use a generated source file whose changes are tracked by Cargo.
6. Frontend changes must cause the frontend build and server embedding step to run. Backend-only changes may reuse validated staged assets.

If the chosen repository tooling requires a different model, explain why, show how recursion and stale output are prevented, and identify the single command that owns the complete release build.

Generate an authoritative asset manifest during the frontend packaging step. For every embedded URL, record its URL path, source file, byte length, SHA-256 hash, and MIME type. The server must validate that every required startup asset is present, that every manifest entry exists, and that no asset is served outside the manifest. Decide explicitly whether unexpected staged files are rejected; the default should be to reject them so omissions and accidental files fail the build.

Specify exactly:

- which commands run
- in what order
- from which directory
- with which environment variables
- with which Cargo targets
- which target triples are involved
- which generated directory is consumed
- how the server build knows where that directory is
- how stale assets are prevented from being embedded
- how a clean build works
- how incremental builds work
- how frontend changes trigger server rebuilds
- how backend-only changes behave
- how build failures are reported.

If using `build.rs`, explain precisely what it does.

For example, evaluate whether it should:

1. invoke the frontend build;
2. locate generated assets;
3. verify required files;
4. generate an asset manifest;
5. emit `cargo:rerun-if-changed=...`;
6. embed files;
7. fail the build if expected artifacts are missing.

Be explicit about the tradeoffs.

The release build must be safe from partial output: build into a fresh temporary staging directory and replace the prior staging directory only after frontend generation and manifest validation succeed. A failed frontend build must never leave output that a later server build can mistake for current assets.

---

# 8. Reproducible builds

Explain how to make the build reproducible.

Cover:

- `Cargo.lock`
- pinned tool versions
- Rust toolchain version
- WASM target installation
- Node/npm if actually required
- wasm-bindgen/wasm-pack/Trunk versions if applicable
- generated assets
- timestamps
- environment-dependent paths
- release profile
- stripping symbols
- deterministic asset generation.

If the project currently does not provide reproducible builds, identify exactly what should be changed.

Add an explicit reproducibility check. Two clean builds from the same pinned source revision and tool versions must produce identical frontend asset manifests and identical hashes for every staged asset. Platform-specific server binaries may differ, but they must embed the same frontend manifest. Document any unavoidable nondeterminism and define the accepted comparison rule.

---

# 9. Cross compilation

Analyze how this works for at least:

```
Linux x86_64
Linux aarch64
```

and, if the repository/build system makes it practical:

```
Windows x86_64
macOS x86_64
macOS aarch64
```

Explain:

- which parts are target-specific
- which parts are target-independent
- whether the embedded WASM frontend is identical across server targets
- whether the Axum binary can be cross-compiled
- what system libraries are required
- whether static linking is desirable/possible
- whether musl should be considered
- how final artifacts should be named.

For example:

```
selfsignedcert-server-linux-x86_64
selfsignedcert-server-linux-aarch64
selfsignedcert-server-macos-aarch64
```

Do not assume full portability if dependencies prevent it. Clearly identify limitations.

Build the frontend once because browser WASM assets are target-independent with respect to the server operating system. Reuse the validated manifest for each server target. Do not run a target-specific frontend build merely because the Axum binary target changes. For each supported target, state the linker/toolchain prerequisites and perform a clean-directory smoke test with that target's binary.

---

# 10. Binary-size analysis

Estimate the likely composition of the final executable:

```
Axum/Tokio/application code
+
Rust runtime/code
+
embedded HTML
+
embedded JavaScript
+
embedded WASM
+
embedded CSS/assets
```

Discuss whether embedding the WASM significantly increases binary size and whether compression should be considered.

Evaluate:

- raw embedded assets
- gzip
- Brotli
- precompressed WASM
- runtime decompression
- HTTP compression
- startup memory
- request memory.

Do not optimize prematurely.

Recommend the simplest reliable solution unless measurements justify additional complexity.

---

# 11. Security

Analyze the resulting server as an Internet-facing application.

Cover:

- path traversal
- arbitrary file access
- directory traversal
- unsafe dynamic filesystem access
- MIME sniffing
- CSP
- CORS
- cache poisoning
- HTTP request smuggling considerations
- oversized requests
- malformed paths
- invalid UTF-8 paths
- denial-of-service considerations
- logging sensitive information
- binding to `0.0.0.0`
- TLS expectations.

Important:

> The server should preferably have **no runtime dependency on the filesystem for frontend assets**.

HTTP requests should map only to a finite, compile-time-known set of embedded resources.

Explain how that guarantees that a request such as:

```
../../etc/passwd
```

cannot cause arbitrary filesystem access.

Treat the following security requirements as mandatory unless repository inspection documents a concrete incompatibility:

1. The request path is resolved only by exact lookup in the validated embedded asset manifest. Never concatenate a request path with a filesystem path, follow symlinks, read a directory, or serve a fallback from disk at runtime.
2. Query strings must not affect asset selection unless the frontend demonstrably requires them. Fragment identifiers are never sent to the server and must not be relied on for routing.
3. Percent decoding, dot-segment normalization, repeated separators, encoded separators, backslashes, invalid UTF-8, and oversized paths must have explicit behavior. Reject or return 404; they must never broaden the lookup set.
4. Unknown paths and missing static assets must return the documented 404 response. An SPA fallback, if required by the inspected frontend, may apply only to navigational document requests and must never turn a missing `.js`, `.wasm`, `.css`, image, font, or JSON request into an HTML 200 response.
5. Set `X-Content-Type-Options: nosniff` and document the rationale for every other security header. At minimum evaluate `Content-Security-Policy`, `Referrer-Policy`, `Permissions-Policy`, and frame protection (`frame-ancestors` in CSP). Do not add headers that break the actual application, and test the final policy in a browser.
6. Do not enable CORS by default. If cross-origin access is required, specify the exact allowed origins, methods, headers, credentials policy, and preflight behavior. Never use a reflected `Origin` with credentials.
7. Define cache behavior per asset class. Immutable hashed assets may be long-lived; HTML must not be cached as immutable. Prevent cache-key ambiguity by ensuring the cache-relevant URL and response vary consistently.
8. Configure request and header size limits where supported by the selected stack, and document the behavior for oversized requests, malformed HTTP, slow clients, and excessive concurrency. Do not buffer unbounded request bodies for this static server.
9. Use structured logging with redaction. Do not log authorization headers, cookies, full query strings, or request bodies. Define whether and how client IPs are obtained when behind a trusted proxy; do not trust forwarding headers from untrusted clients.
10. Binding to `0.0.0.0` exposes the service on every IPv4 interface. Document that this is an operational choice, not an authentication or TLS feature. State whether TLS is terminated by a reverse proxy or must be implemented separately; do not imply that the plain HTTP binary provides HTTPS.
11. Rely on the selected HTTP library's request parsing and connection handling rather than implementing a custom parser. Document the reverse-proxy requirement that the proxy and Axum agree on HTTP version, header limits, and connection framing to reduce request-smuggling risk.
12. Run dependency and secret scanning in CI, pin dependency versions through the lockfile, and document the process for responding to security advisories. Do not include source maps or debug assets in production unless their exposure is intentional.

The security analysis must distinguish what this static server protects (for example, arbitrary host filesystem access) from what it does not protect (for example, application-layer vulnerabilities in the WASM frontend, exposed HTTP traffic, or a compromised deployment host).

---

# 12. Logging

Design simple logging behavior.

Normal mode should produce useful but not excessive output.

Verbose/debug mode should provide:

- startup configuration
- listening address
- request logging
- status codes
- useful error details.

Avoid logging secrets or unnecessarily dumping request data.

Use an idiomatic Rust logging/tracing stack such as:

- `tracing`
- `tracing-subscriber`

if appropriate.

Explain the exact behavior of:

```
--verbose
```
and/or:

```
--debug
```

---

# 13. Health endpoint

Evaluate whether the server should provide:

```
GET /health
```

returning something simple such as:

```
OK
```

If included, explain whether it should be:

- plaintext
- JSON
- cache-disabled.

Do not add unnecessary API endpoints.

---

# 14. Testing strategy

Create an explicit testing plan.

## 14.1 Unit tests

Examples:

- asset lookup
- MIME type lookup
- missing asset behavior
- path normalization
- CLI parsing.

## 14.2 Integration tests

Start the actual Axum application on an ephemeral port and verify:

```
GET /
GET /index.html
GET <actual wasm path>
GET <actual JS path>
GET <actual CSS path>
GET /nonexistent
GET /health
```

Verify:

- status codes
- content types
- response bodies
- important headers.

The integration suite must assert exact expected status codes and content types for each discovered asset, including `application/wasm` for the actual WASM response. It must assert that a missing JavaScript, CSS, or WASM asset returns 404 rather than the HTML fallback. It must test `HEAD` behavior if supported and document whether range requests are intentionally unsupported.

The security portion of the integration suite must request encoded and normalized traversal variants, including paths containing percent-encoded dot segments, encoded slashes, repeated separators, and backslashes where the platform permits them. Every such request must be unable to access the host filesystem and must return the documented 404/400 response.

## 14.3 Browser-level validation

Describe how to verify that the actual WASM application loads successfully in a real browser. Use a pinned browser automation tool/version in CI where practical.

The test must not merely verify that `/` returns HTTP 200.

It must verify:

```
HTML loads
→ JS loads
→ WASM loads
→ application initializes
```

Specify the application-specific ready element, initialized state, or other observable success condition. The browser test must fail on console errors, uncaught page exceptions, failed network requests, incorrect MIME types, or a response that contains HTML where JavaScript/WASM was requested. A status-200 response for `/` alone is insufficient.

---

# 15. Deployment validation

Define a final smoke test proving the single-binary requirement.

For example:

```
mkdir /tmp/selfsignedcert-test

cp target/release/selfsignedcert-server \
   /tmp/selfsignedcert-test/

cd /tmp/selfsignedcert-test/

./selfsignedcert-server \
    --listen=127.0.0.1 \
    --port=8080
```

Then verify that the directory contains **only the executable**.

This test is mandatory.

Make the smoke test non-interactive and deterministic: select an ephemeral available port, start the binary from a directory containing only the executable, wait for readiness, issue requests from an external client, assert the browser/application behavior, then terminate the process and assert its exit handling. Run it with no project-root working directory assumptions and with frontend source/build directories unavailable.

Explicitly include a test proving that the server does not accidentally depend on:

```
./static
./dist
./pkg
./public
./index.html
```

or any other external frontend directory.

The test must also run after renaming or removing the source checkout and must verify that the process has no open frontend asset files outside the executable. The primary proof is successful application loading from the isolated directory, not merely the directory listing.

---

# 16. Failure modes

Document what happens if:

- frontend build fails
- WASM target is missing
- generated output is missing
- expected `index.html` is missing
- expected `.wasm` is missing
- generated JS is missing
- an unexpected asset appears
- server cannot bind the requested address
- port is invalid
- port is already occupied
- malformed HTTP request arrives
- unknown URL is requested.

Distinguish:

## Build-time failures

These should stop the build and provide actionable diagnostics.

## Runtime failures

These should produce appropriate logs and exit codes.

---

# 17. Repository changes

Produce a precise proposed repository layout.

For every new or modified file, explain:

- filename
- purpose
- important contents
- dependencies
- build-time vs runtime role.

For example:

```
server/Cargo.toml
server/src/main.rs
server/src/assets.rs
server/build.rs
Cargo.toml
Cargo.lock
rust-toolchain.toml
```

But do **not** assume these exact files are appropriate.

Derive the structure from the actual repository.

---

# 18. Dependency analysis

List every new Rust dependency you propose.

For each dependency provide:

```
crate
version
purpose
compile-time/runtime role
why it is needed
whether an existing dependency can replace it
```

At minimum consider whether the project needs:

```
axum
tokio
clap
tracing
tracing-subscriber
mime_guess
rust-embed
```

Do not add crates that are unnecessary.

If an existing dependency can perform a required task, prefer reusing it.

---

# 19. Exact implementation plan

After investigating the repository and evaluating the architecture, produce a numbered implementation plan detailed enough that another engineer can implement it without making architectural decisions.

For each step provide:

1. Goal
2. Files changed
3. Exact code/component to add
4. Commands to execute
5. Expected output
6. Validation criteria
7. Failure modes
8. Dependencies on previous steps.

Avoid vague instructions such as:

> Add a server.

Instead provide instructions at this level:

> Create `server/src/main.rs`. Define a `Cli` struct using `clap::Parser` with fields `listen: IpAddr`, `port: u16`, and `verbose: bool`. Construct the socket address using `SocketAddr::new(cli.listen, cli.port)`. Initialize `tracing_subscriber` based on `cli.verbose`. Build the Axum router and bind with `tokio::net::TcpListener`...

The plan should go to this level of specificity throughout.

---

# 20. produce a plan that can be handed directly to an engineer or used as the specification for implementing the Proposed source code

After the plan, provide **representative complete source code** for the critical pieces.

At minimum show:

1. `Cargo.toml`
2. `main.rs`
3. asset embedding mechanism
4. build script if needed
5. frontend build integration
6. router
7. static asset handler
8. MIME handling
9. CLI parsing
10. logging initialization
11. health endpoint
12. tests.

The code must be consistent with the repository you actually inspected.

Do not provide pseudocode where real Rust code can reasonably be provided.

If exact generated asset filenames depend on the repository, first determine them from the repository rather than inventing filenames.

---

# 21. Build commands

Give exact commands for:

## Development build

```
...
```

## Release build

```
...
```

## Clean release build

```
...
```

## Linux deployment build

```
...
```

## Optional cross-builds

```
...
```

For every command, explain where it should be executed and what it produces.

---

# 22. Final artifact

Clearly identify the exact final artifact.

For example:

```
target/release/selfsignedcert-server
```

Then demonstrate:

```
cp target/release/selfsignedcert-server \
   /some/deployment/directory/

cd /some/deployment/directory/

./selfsignedcert-server \
    --listen=0.0.0.0 \
    --port=8080
```

The deployment directory must contain only:

```
selfsignedcert-server
```

No frontend files.

---

# 23. Acceptance criteria

The implementation is complete only if all of these are true:

- The server uses Axum.
- The frontend is the actual `mailvibi/selfsignedcert` application.
- The WASM frontend is built as part of the build process.
- All runtime frontend assets are embedded into the executable.
- No frontend runtime files are required beside the executable.
- `index.html` is served correctly.
- JavaScript is served correctly.
- WASM is served correctly.
- CSS/assets are served correctly if the application uses them.
- Correct MIME types are returned.
- Browser initialization succeeds.
- `/health` works if included.
- Unknown resources return an appropriate HTTP error.
- Arbitrary filesystem paths cannot be accessed.
- `--listen` works.
- `--port` works.
- verbose/debug logging works.
- invalid CLI arguments fail cleanly.
- bind failures are reported clearly.
- release build succeeds from a clean checkout.
- resulting binary works when copied to a clean deployment directory.
- clean deployment directory contains no frontend files.
- application still loads from that clean directory.
- tests verify the critical behavior.
- build process is documented.
- solution does not rely on undocumented assumptions about the existing project.

The following are additional mandatory pass/fail checks:

- The inspected frontend revision is recorded by immutable commit SHA.
- The release build has one documented owner and does not invoke Cargo recursively from `build.rs`.
- A generated asset manifest lists every embedded URL, byte length, hash, and MIME type.
- Missing, stale, partial, or unexpected staged assets fail the build rather than being silently ignored.
- The server serves only manifest entries; it never maps request paths to arbitrary filesystem paths.
- Encoded traversal and malformed-path tests pass.
- Missing static assets do not receive an SPA HTML fallback.
- `X-Content-Type-Options: nosniff` is present, and the chosen CSP, referrer, permissions, and frame policies are tested against the actual application.
- CORS is disabled unless a concrete cross-origin requirement is documented with an exact allowlist and tests.
- Oversized paths/headers and malformed requests fail within documented bounds without unbounded allocation or process termination.
- Logs do not expose authorization data, cookies, request bodies, or secrets, and proxy-derived client identity is trusted only under a documented proxy policy.
- TLS termination, `0.0.0.0` exposure, reverse-proxy framing, and the limits of the static server's security boundary are documented.
- Dependency and secret scanning are part of the release or CI validation process.
- Browser automation observes successful application initialization, with no console errors or failed network requests.
- Two clean builds from the same pinned inputs produce identical frontend asset manifests and hashes.
- The isolated deployment smoke test succeeds when the source checkout and all frontend directories are unavailable.
- The chosen CLI flags, defaults, accepted address formats, and exit codes are specified without alternatives left to the implementer.

---

# 24. Required output structure

Your response must use exactly this high-level structure:

## 1. Repository findings

What you discovered by inspecting `mailvibi/selfsignedcert`.

## 2. Runtime architecture

Detailed architecture of the final single binary.

## 3. Build architecture

Exact frontend → embedding → server build pipeline.

## 4. Repository changes

Complete file-by-file change list.

## 5. Dependencies

Exact dependency additions/removals and rationale.

## 6. Implementation plan

Numbered, highly explicit implementation steps.

## 7. Critical source code

Complete representative Rust/build configuration.

## 8. Build and release commands

Exact commands from clean checkout through final binary.

## 9. Testing strategy

Unit, integration, browser, and clean-directory deployment tests.

## 10. Security considerations

Concrete security analysis.

## 11. Cross-compilation

Target-specific build considerations.

## 12. Operational behavior

CLI, logging, startup, errors, and HTTP behavior.

## 13. Acceptance checklist

Strict pass/fail checklist.

## 14. Open questions / repository-specific uncertainties

Only include questions that genuinely cannot be answered by inspecting the repository.

---

# Important constraints

Do not hand-wave.

Do not invent repository files, build commands, generated filenames, or framework details.

Do not assume that "WASM" means a particular Rust framework.

Do not merely describe a possible architecture.

The objective is to produce a plan that can be handed directly to an engineer or used as the specification for implementing the solution.

Where multiple implementation approaches are possible, analyze them briefly and then choose one based on explicit technical criteria.

Prioritize:

1. Correctness
2. Single-binary deployment
3. Reproducible builds
4. Simplicity
5. Maintainability
6. Security
7. Performance
8. Binary size

The final deployment experience should remain as close as possible to:

```
./selfsignedcert-server --listen=0.0.0.0 --port=8080
```

with the binary alone being sufficient to serve the complete WebAssembly application.
