# Validation evidence

Latest local checkpoint: 705 tests pass in both debug and release on the pinned
Windows/Rust host. All-target Clippy and rustdoc pass with warnings denied;
formatting and repository policy pass (676 source files, 6 crates; all checked source
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

The 699-test mixed snapshot-carrier checkpoint passed hosted Windows/Linux CI in
[run 41521657642](https://github.com/cod4xr/client/actions/runs/41521657642), including
locked sources and table reproduction. The 704-test mixed original-owner ACK run is
[41523347146](https://github.com/cod4xr/client/actions/runs/41523347146); its final hosted
status is separate from the successful local checks. These changes edit only the client repository and consume
the published shared revision.

A real protocol-21 loopback UDP case returns prepared mixed snapshots from scoped
receive without copying packet bytes, rejects malformed tails, retries host identity
resource preflight and verifies the next control's original key and three ACK words.
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
