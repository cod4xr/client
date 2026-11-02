# Validation evidence

Latest local checkpoint: 758 tests pass in both debug and release on the pinned
Windows/Rust host. All-target Clippy and rustdoc pass with warnings denied;
formatting and repository policy pass (745 source files, 6 crates; all checked source
files below 250 lines). Discovery, challenge state/runtime, owned connect requests and bounded
reply decoding, bounded pending-connect state and owned UDP phase transitions are
covered, along with ordinary-channel integration, checked client-command history,
owned control encoding, payload/service validation, configurable server-command
retention, auxiliary reliable bodies, receive/send windows, whole ACK admission,
application framing, paced auxiliary UDP sends and socket/clock/endpoint ownership.
Complete protocol-21 gamestate constituents now have transactional coverage.
Reliable gamestate bodies now have whole-frame admission coverage. Mixed-message and host history
admission, the ongoing game loop, typed command delivery,
engine/content application and ordinary gameplay remain open.

The 749-test input-history checkpoint passed hosted Windows/Linux CI in
[run 41537255526](https://github.com/cod4xr/client/actions/runs/41537255526), including
locked sources and table reproduction. The preceding movement-packet checkpoint
also passed [run 41534071481](https://github.com/cod4xr/client/actions/runs/41534071481).
These changes edit only the client repository and consume the published shared revision.

Seven exact prediction-history cases cover 257 samples, arbitrary/wrapped time keys,
explicit replay/cancellation, stale producers, reconstruction and actual decoded
player-state request resolution. All 120 removal orders of a full five-cell table
preserve every remaining key. An independent ordered-map model checks 2,048
operations at five capacities in both profiles. Two compile-fail examples protect
original-owner updates and borrowed lookups. See [prediction history](prediction-history.md).

Nine input-history integration cases cover 257 retained inputs, exact local IDs,
backups, send cancellation, sent-only retirement, constant-time logical reset,
storage lifetimes and world/profile binding. An independent queue model checks 512
operations at four capacities in both profiles. A private case crosses 32-bit local
sequence boundaries and exhausts 64 bits without wrap; a compile-fail example
protects the original owner. The existing UDP movement case retains its input lease
through complete I/O and confirms reliable ACKs do not retire inputs. See
[input command history](input-command-history.md).

Seven complete movement-packet cases cover both profiles, full one/two/thirty-two
input batches, signed native keys, shared-decoder field reconstruction, raw prediction
reports, mode/reference binding, native limits, atomic output and compression retry
that defers the final reliable record while preserving every input. A private hash
case covers nine compiler-evaluated fixtures. A real UDP case admits a high-byte
server key, sends the complete batch through the original socket, and releases its
reliable record only after the peer's fully validated response. All seventeen pinned
public-client source hashes verify locally. See [movement packets](movement-packets.md).

Six native user-command integration cases cover an independent physical vector,
all key bits, timestamp signs/extremes, nonzero references, canonical axes/actions,
raw location bytes, native widths/presence, exact/short capacities and private Debug.
One private unit case checks every initial cached-bit width and following fields.
They reuse the published shared decoder; [writer evidence](user-command-encoding.md)
contains metadata only. Complete packet and input/send ownership now compose through
the separate movement builder and input history; engine sampling remains open.

Seven history-assisted mixed decoding cases cover actual-header reference selection
in both profiles, owned scratch/history lifetimes, full/cancel/retry behavior, exact
missing/evicted/wrong-world/profile/order rejection, prefix binding/error ordering,
independent prefix/tail budgets and late wire/prediction failure. The existing real
UDP case now covers full-to-delta retention and both next control packets. See
[history-assisted decoding](history-snapshot-decoding.md).

Eight whole-snapshot history cases cover both profiles, exact retained references,
40 slots and distance 255, independent sparse capacities, cancellation/tails,
epoch/profile/order rejection and reset, and mixed sibling-owner resource failure.
An independent queue model runs 256 operations at five capacities in both profiles.
Two compile-fail examples protect history/source borrows; joint mixed ACK preflight
now includes snapshot retention. See [snapshot history](snapshot-history.md).

A real protocol-21 loopback UDP case returns prepared mixed snapshots from scoped
receive without copying packet bytes, rejects malformed tails, retries host identity
resource preflight, retains a full snapshot and selects its exact prior for a later
delta from the actual header. The peer verifies both subsequent controls' original
keys and three ACK words. History slots are reused only after complete delta decode.
This adds synthetic transport evidence, not native gameplay interoperability.

Four mixed original-owner ACK cases cover actual decode in both profiles, original
key cancellation/retry and slot reuse, packet/workspace/scratch lifetimes, private
Debug, incoming command/identity preflight and commit, identity resource-budget
rejection, and late wire/prediction failure. A compile-fail example protects the
original outgoing owner. See [mixed snapshot acknowledgements](mixed-snapshot-acknowledgements.md).

Ten mixed snapshot-carrier cases cover commands and protocol-21 identities on
both sides of a snapshot, native prefix-only command association, normalization,
actual payload binding, all four final output buffers/tails, independent budgets,
late control/prediction failures, unsupported framing, owned lifetimes and cooked
source reuse. Existing command and identity owners preflight/cancel/commit the
same cooked control plan. The shared incremental control scanner retains the
existing codecs and all earlier suites. See [mixed snapshot services](mixed-snapshot-services.md).

## Evidence by subsystem

Historical coverage and audit details are kept in focused pages:

- [Snapshots, player state and entities](validation/snapshots.md)
- [Gamestate, identities and controls](validation/gamestate-controls.md)
- [Content catalogs, payloads, images and zones](validation/content.md)
- [Transport, reliable framing and network ownership](validation/transport.md)
- [Test organization, inspectors and local exclusions](validation/tooling.md)
- [Initial foundation checkpoint](validation/foundation.md)

Current limitations include a connection-wide session/history owner, prediction production, typed simulation,
rendering and ordinary peer captures. The CLI remains a probe/content inspector.
