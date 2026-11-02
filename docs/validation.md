# Validation evidence

Latest local checkpoint: 699 tests pass in both debug and release on the pinned
Windows/Rust host. All-target Clippy and rustdoc pass with warnings denied;
formatting and repository policy pass (665 source files, 6 crates; all checked source
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

The 689-test original-owner snapshot ACK checkpoint passed hosted Windows/Linux CI
in [run 41518825433](https://github.com/cod4xr/client/actions/runs/41518825433), including
locked sources and table reproduction. Hosted mixed snapshot-carrier validation
will follow its commit. These changes edit only the client repository and consume
the published shared revision.

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

Current limitations include mixed-carrier original-owner ACK integration, a
connection-wide session/history owner, prediction production, typed simulation,
rendering and ordinary peer captures. The CLI remains a probe/content inspector.
