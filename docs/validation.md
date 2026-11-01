# Validation evidence

Latest local checkpoint: 60 tests pass in both debug and release on the pinned
Windows/Rust host. All-target Clippy and rustdoc pass with warnings denied;
formatting and repository policy pass (59 source files, 3 crates, largest source
238 lines). Discovery, challenge state/runtime, owned connect requests and bounded
reply decoding, bounded pending-connect state and owned UDP phase transitions are
covered. Channel initialization and ordinary gameplay remain open.

The 52-test pending-connect checkpoint passed hosted Windows/Linux CI in
[run 41341395727](https://github.com/cod4xr/client/actions/runs/41341395727).
Hosted validation for the latest socket-ownership checkpoint is pending. These client changes
edit only the client repository and consume the already published shared revision.

Eight new independent loopback tests exercise the reusable network crate: one
source socket across challenge/connect/retries/handoff, exact cookie/protocol/qport
fields, inert owned refusal data with socket closure, oversized-datagram rejection,
other-peer filtering, last-slot receive quotas, expiration before a delayed `run`,
silent-peer expiry with retained work counters, and pre-I/O budget/endpoint rejection.
No ordinary gameplay server or complete game channel was used for this checkpoint.

## Initial foundation

Date: 2026-11-01. Local host: Windows x86_64-pc-windows-msvc, Rust 1.95.0
(`59807616e`, LLVM 22.1.2). The existing server Cargo target directory was reused;
no client-specific target root or alternate build cache was created.

| Check | Local result |
| --- | --- |
| Rustfmt | Pass |
| Workspace/all-target Clippy, warnings denied | Pass |
| Debug tests | 16 pass: 8 wire/parser, 7 real UDP/process, 1 OS argument-encoding |
| Release tests | Same 16 pass |
| Rustdoc, warnings denied | Pass |
| Source/crate/dependency policy | 18 source files, 2 crates; largest source 220 lines |
| Pinned reference hashes | 7 files across 2 snapshots pass |
| Local cross-repository query | Pass; 1 accepted reply, 0 rejected packets |

Wire tests cover exact requests, every insufficient output prefix, required-field
truncation, malformed input, duplicate/stale correlation, byte encodings, numeric
overflow and normal-info capacity. Process tests cover wrong endpoints, stale and
oversized datagrams, deadline/receive-budget exhaustion, separate report/log output,
valid JSON-only failure logging and non-Unicode OS argument rejection.

The cross-repository check used the locally available query-only server executable,
identified by SHA-256 in [the evidence record](local-discovery-evidence.json).
It does not certify a server source checkout or ordinary CoD4X interoperability.
No native gameplay server, game renderer or complete join was tested. Auth-required
servers remain outside the selected no-auth compatibility baseline.

Windows/Linux hosted CI and source-verification jobs are configured but have not
run for this initial local commit. Linux behavior is not claimed from Windows tests.
