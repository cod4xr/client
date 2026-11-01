# CoD4XR client

CoD4XR is a general-purpose Rust client and server project. This repository owns
the client runtime; [cod4xr/server](https://github.com/cod4xr/server) supplies shared
protocol libraries at an immutable revision. Mods and communities can use future
features without defining the core product.

**Current status: runnable server-discovery foundation.** The client sends real
UDP `getinfo` queries and validates correlated replies. Rendering, assets,
movement, snapshots, GSC presentation and playable joins are not implemented.

The compatibility target is existing CoD4/CoD4X servers accepting **no-auth
clients**. Auth-required servers are outside this baseline. A discovery reply
does not establish gameplay compatibility, server identity or an authenticated
connection. In particular, discovery protocol `6` is not a negotiated join version.

## Run

Rust 1.95 is pinned. On this workstation, reuse the existing Cargo target:

```powershell
.\tools\use-existing-target.ps1 -TargetPath C:\Users\Ethan\source\cod4x-rust\target
cargo run -p cod4xr-client -- query 127.0.0.1:28960
cargo run -p cod4xr-client -- query 127.0.0.1:28960 --log-format json
```

Other workstations should supply their own **existing** target directory. The
local Cargo configuration is ignored by Git. Do not create alternative target
directories to work around executable locks; stop this client's old process with
`tools/stop-dev-client.ps1`. Its exact executable path is verified before stopping.

The query command accepts IPv4 or bracketed IPv6 socket addresses, a deadline of
1–30,000 ms and a receive budget of 1–4096 datagrams. It sends one query, filters
reply endpoints through a connected socket, requires the unique echoed random
token, rejects oversized/malformed replies and keeps one absolute deadline.
OS entropy supplies correlation tokens; this is not authentication.

Standard output contains one JSON metadata report. Human development logs use
stderr; release logs default to JSON. Unknown remote bytes remain bytes in the
core and use lossy UTF-8 only for display. Raw peer strings/tokens stay out of logs.

## Structure and checks

- `crates/client-core`: deterministic discovery over the shared wire codecs.
- `apps/client`: arguments, socket deadlines/budgets, logging and report output.
- `docs`: architecture, source evidence, compatibility gates and remaining work.
- `tools`: source/dependency checks, reference verification and local build setup.

```powershell
cargo fmt --all -- --check
cargo clippy --workspace --all-targets --locked -- -D warnings
cargo test --workspace --locked
cargo test --workspace --release --locked
cargo doc --workspace --no-deps --locked
python tools/check_repository.py
```

Handwritten source aims for 250 lines and cannot exceed 400. Workspace metadata,
strict Rust/Clippy lints, unsafe prohibition and dependency boundaries are checked.
The Windows/Linux CI workflow is configured; local results are recorded separately
from hosted CI and real stock-client/server interoperability.

See [architecture](docs/architecture.md), [compatibility](docs/compatibility.md)
and [roadmap](docs/roadmap.md). [Local validation](docs/validation.md) records 16
passing debug/release tests and the cross-repository discovery check. Game files
and private configuration are not shipped.
