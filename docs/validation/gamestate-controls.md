# Gamestate, identity and control validation

These records describe historical checkpoints. Remaining-gate statements apply to
that checkpoint; see [current validation](../validation.md) for the latest status.

Fifteen typed pre-game tests add new-command-only disconnect/map/restart notices,
native raw-prefix/token/copy-boundary rules, normalization and raw bytes, all
control-byte categories, truncated quote/comment forms, unknown-key retention,
slot collisions, source/history lifetimes, independent byte/event budgets and
late cancellation. A compile-fail example protects exclusive history ownership.
One actual UDP test rejects a third notice on budget, cancels/retries preparation,
then returns all notices after history/ACK commit despite one retained slot. Its
peer decodes the next frame with message ACK 7, command ACK 3 and key `m`. The
fifteenth client-reference source hash now pins the tokenizer. No console, script,
UI, filesystem or session effects execute. See [typed controls](../pre-game-controls.md).

Twelve mixed-service core tests add complete protocol-21 command/identity/NOP/EOF
streams, independent identity counts beyond the command window, source/workspace
release, strict legacy/profile entry points, every truncation, late unsupported or
malformed identities, independent budgets/capacities, unchanged final tails, logical
padding isolation and joint command/identity preflight cancellation. One actual UDP
test rejects a late bad service without releasing a pending client command, then
admits a valid mixed stream. Its peer decodes the next client datagram with message
ACK 8, command ACK 3, config-data ACK 11 and the new retained key. Ordinary wire
limits stay unchanged; typed execution, other services and native gameplay peers
remain open. See [mixed controls](../mixed-service-controls.md).

Ten configstring-block tests add independent legacy packed-index and protocol-21
32-bit vectors; normalized/raw/empty strings; cooked-only plan lifetimes; both full
wire namespaces; a complete block above 128 KiB; native maximum string lengths;
invalid counts/order/IDs; every input truncation and short caller capacity; host
budgets; dirty metadata and preservation of original reader/cache state on late
failure. This validates explicit patches, without native constant defaults, entity
baseline deltas, whole gamestate application or a real gameplay peer.

Six client-data tests add owned protocol-21 gamestate and ordinary sequenced
identity vectors, maximum/empty names and tags, raw bytes and percent normalization,
source-independent ownership, large reader offsets, unsupported profiles, every
truncation, strict field/identifier bounds, sequence high-bit preservation, cached
bit rollback and private debug output. The newer identity-state and mixed control
tests cover sequence admission and complete supported carriers separately; engine
updates and remaining services stay open.

Nine gamestate tests add independent complete constituent framing, cooked-only plan
lifetimes, mixed/interleaved records, multiple configstring blocks with global text
offsets, repeated client IDs, exact following byte/bit fields, all 4884 configstrings
and 1024 baseline entities, 65 retained client records, a complete body above 128 KiB,
every fixture truncation, invalid nested/opcode/trailer records, unsupported profiles,
dirty-output and cached-bit rollback, short capacities and exact host-budget bounds.
Enclosing-message admission, default importing, engine application and native peer
captures remain outside this constituent-level validation.

Five reliable-wrapper tests add independent complete body bytes, opaque tail bits,
source-independent plans, native unused-bit padding, every fixture truncation,
invalid commands/opcodes, every appended byte value, whole-body/host/storage budgets
and late extra-byte rejection above 128 KiB. A real UDP test reorders complete frames,
rejects a malformed earlier body without changing gamestate output, explicitly
discards it, admits the next valid body and retains the cooked plan after network
frame release. Transport byte ACKs remain separate from application validity. An
additional immutable public writer file is hash-locked from read-only remote evidence;
local server source/worktrees were not read or edited for that verification.

Three owner-preparation UDP tests add no-frame/terminal rejection without decode
work, exclusive preparation, cancel/retry, exact-once typed commit/release, cooked
data after socket/source-buffer release, failed decode preservation and explicit
discard followed by valid admission. New saturating counters distinguish attempts,
decode failures and commits, without changing receipt time or byte ACK state. One
compile-fail documentation test prevents replacing a frame while its prepared
handle remains live. Session/engine updates remain separate from frame commit.

One block-scope regression verifies native default restoration across multiple
blocks without erasing earlier nondefault IDs. Existing standalone/mixed/minimal
tests also verify original start markers and exact block counts, with no source
frame retained or additional wire field. Existing failure tests continue to cover
whole-output rollback, including this metadata.

Eight sparse-config tests add native per-block restoration, nondefault persistence,
empty/latest overrides, no-block behavior, wide sparse IDs, large/raw values,
source-independent output, every short budget/scratch/final capacity, dirty workspace
reuse, invalid order/markers/NULs and private debug output. An independent map model
covers 256 traces; a decoded-wire integration releases parent plans and source frames
after merging. Imported default views also support source-lived byte references
from temporary copied records. No engine or session state is automatically applied.

Nine system-info tests add signed/unsigned identifier bit preservation, complete
decimal validation, missing/duplicate critical keys, unknown/raw inert values,
whole-string shared syntax checks, all forbidden control bytes, exact byte/field
budgets and big-info wire bounds. Debug/errors expose no remote text. A gamestate
wire-to-snapshot integration releases original sources before interpreting system
metadata. This performs no session admission, local CVar changes or archive mounts.

Seventeen identity-state checks add fourteen integration tests, local generation
exhaustion, exclusive-owner compile-fail coverage and a real UDP joint-commit test.
They cover explicit baseline initialization, last duplicate wins, exact wrapping
successors, gaps/duplicates, full sequence bits, independent budgets/capacities,
late rollback, cancellation, replacement epochs, dirty workspace reuse, unchanged
tails and source-free retained records. A 256-trace ordered-map model checks
thousands of batches. Reordered malformed/valid UDP frames prove that wire failure,
staging failure and joint cancellation preserve the correct frame/identity owners;
joint commit retains data after all source/work buffers are cleared. Fourteen
locked client-reference hashes verify without reading a server checkout. Whole
ordinary-message identity dispatch and remaining session/engine resources are open.

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
