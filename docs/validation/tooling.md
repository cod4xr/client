# Tooling and inspector validation

These records describe historical checkpoints. Remaining-gate statements apply to
that checkpoint; see [current validation](../validation.md) for the latest status.

At the recorded consolidation checkpoint, the core harness loaded all 31 component
suites in one executable
and shares four fixture modules once. Before/after listing comparisons preserved
the exact qualified identities of 289 integration and nine unit cases; the
[inventory evidence](../core-test-inventory.json) records their canonical hash. Full
workspace coverage remains unchanged. Missing, duplicate, unknown and conditional
harness declarations fail repository policy. Production runtime code, release
settings and the existing local Cargo target remain unchanged; see
[test organization](../testing.md).

Twenty-two content/inspector tests add synthetic PE32 header/section/address
validation, every required prefix, virtual-only/overflow geometry, deterministic
header/address mutations, exact image identity, native table shape/order/terminators,
short budgets/storage with complete rollback, raw defaults beyond wire lengths,
source-independent plans, native/non-Unicode arguments, bounded regular-file reads
and JSON/human process failures without paths or values. Debug and release inspector
runs against the reviewed installed client both imported 832 records/18,813 value
bytes; [metadata evidence](../client-image-defaults-evidence.json) records its identity.
No game binary or values are included in fixtures or Git. Session admission,
engine/content application, maps/rendering and playable sessions remain open.

The local workspace contains a pre-existing edit to the challenge process test;
it is excluded from this checkpoint. Hosted CI checks the committed version.
