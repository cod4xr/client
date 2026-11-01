# Validation evidence

Latest local checkpoint: 118 tests pass in both debug and release on the pinned
Windows/Rust host. All-target Clippy and rustdoc pass with warnings denied;
formatting and repository policy pass (118 source files, 3 crates, largest source
238 lines). Discovery, challenge state/runtime, owned connect requests and bounded
reply decoding, bounded pending-connect state and owned UDP phase transitions are
covered, along with ordinary-channel integration, checked client-command history,
owned control encoding, payload/service validation and configurable server-command retention.
Typed command delivery, auxiliary reliable transport, gamestate and ordinary gameplay remain open.

The 109-test service-control checkpoint passed hosted Windows/Linux CI in
[run 41347783014](https://github.com/cod4xr/client/actions/runs/41347783014).
Hosted validation for the latest retained-history checkpoint is pending. These
changes edit only the client repository and consume the published shared revision.

Eight new independent loopback tests exercise the reusable network crate: one
source socket across challenge/connect/retries/handoff, exact cookie/protocol/qport
fields, inert owned refusal data with socket closure, oversized-datagram rejection,
other-peer filtering, last-slot receive quotas, expiration before a delayed `run`,
silent-peer expiry with retained work counters, and pre-I/O budget/endpoint rejection.
No ordinary gameplay server or complete game channel was used for this checkpoint.

Ten subsequent real UDP tests establish ordinary-channel handoff on the original
socket, qport bits/direction, initial sequences, empty messages, paced fragments
and exact-multiple terminators, interleaved receives, cancellation/I/O-failure
reservation, malformed/stale/reordered traffic, reassembly capacity and explicit
discard, auxiliary marker/qport checks, and expired waits without charged work.
One compile-fail documentation test establishes that receive data cannot escape
the scoped callback. Shared transport logic is consumed at the existing immutable
revision, without source changes to the server repository. Two more client research
files were hash-verified locally and in that checkpoint's hosted source job.

Eight command-history tests add native byte-cleaning/ownership/privacy checks,
valid/invalid caller storage, exact command-length and pending-window boundaries,
the full 128-entry baseline, check-before-commit acknowledgement, unsent/future/
regressing numbers, repeated reuse across every 1–128-slot capacity, and exhaustion
at the positive signed sequence bound. Three additional client source files are
hash-verified locally and in that checkpoint's hosted source job. No
server source files or server worktrees were edited for these client checkpoints.

Eight control-frame tests add independent idle/keyed wire vectors for both header
profiles, source ownership, every insufficient output prefix, a complete 128-command
window, retained-key/header boundaries, whole-prefix deferral and compression
expansion without truncation. A further UDP integration test rejects wrong protocol/
cookie contexts before sequence reservation and verifies a fragmented complete
control frame on the original socket. Marking the included command prefix sent is
separate from construction and from validated peer acknowledgement.

Eight payload tests add explicit profile mapping, independent initial/keyed server
vectors, native padding, key selection without commit, decoded-byte opacity,
unsent/future/regressing numbers, every short scratch/output prefix, incomplete/
empty/oversized input, sequence bounds and reserved compression symbols. Two UDP
tests establish original context use and rejection of unknown original protocols;
one compile-fail example prevents history release during decoded-view inspection.
All caller decoded output and history remain unchanged on decoding errors; private
scratch may be modified and is never consumed as a result on failure.

Nine service-control tests cover logical EOF/padding, complete normalized records,
duplicates/order/empty values, maximum big strings with shorter retained keys, plan
ownership without automatic commit, a complete 128-command window, every record
truncation, scratch/output capacities, quotas, invalid sequences and oversized strings.
A UDP round trip sends a real control frame, rejects a malformed acknowledged service
stream without freeing client history or exposing partial command output, then
validates the next complete message and explicitly commits its acknowledgement.
Transport receipt and host-validated message sequence remain distinct in that test.

Eight retained-history tests cover exact/dirty storage geometry, cancellation,
non-power-of-two slot reuse, retention more than 128 entries back, full big strings
with shorter keys, duplicate/older records, gaps, profile/message order, outgoing
acknowledgement failure cancellation and NOP-only message advancement. A further
UDP round trip stores a normalized server command and uses its exact key/sequence
plus the validated message acknowledgement to produce the next decodable client
control frame. Core preparation does not mutate state; its commit is infallible
under the exclusive preflighted borrows.

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
