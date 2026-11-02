# Configurable server-command history

Validated service plans preserve complete normalized command records without
executing them. `ServerCommandHistory` retains those records for later typed game/
presentation handling and provide the latest native key for outgoing control
frames. Stored command text remains up to 8191 baseline bytes; the outgoing key
uses its separate 1023-byte prefix. No text enters operational Debug/log output.

Storage is caller-owned and budgeted before admission, with explicit checked
byte/metadata geometry. The chosen number of retained slots is configurable
and may exceed 128. The per-message baseline command window does not require an
engine-wide retention ceiling. Slot tags identify original command sequences so
old/missing references cannot alias a reused slot. Byte storage remains opaque
until the corresponding slot has been initialized.

`required_server_command_bytes` checks slot-count multiplication before host
allocation. Bytes must match metadata geometry exactly. Sequence modulo the chosen
capacity selects a slot; exact sequence tags detect reuse. With sparse sequences,
an older non-colliding record can remain available until its slot is reused.

Preparation validates original profile and strictly increasing validated
message order, then select commands newer than the prospective highest sequence.
Duplicate/older command records remain unapplied, matching native monotonic
handling. Gaps are counted without fabricating missing commands. All routing,
length, sequence and capacity checks finish before any retained state changes.

A prepared update borrows the owner exclusively and the validated command
bytes immutably. Dropping it cancels without mutation. Commit is infallible
after preparation, enabling the host to validate/commit its outgoing client-command
acknowledgement before committing the already prepared incoming store. A failed
client acknowledgement therefore cancels incoming changes too. The host must keep
both histories on the same connection and preflight other game resources first.

Eight core tests cover complete/dirty storage checks, duplicate/gap/order/profile cases,
retention and eviction at varied capacities, a history larger than the native 128
entries, full command/key separation, prepare cancellation, atomic failure behavior,
and outgoing-acknowledgement failure cancellation. A bidirectional UDP test validates
a server command, prepares its storage, commits the checked client acknowledgement,
then commits retained records. The next client frame uses validated message sequence
7, server-command sequence 3, and the normalized retained key. Its peer decodes the
next complete command and EOF using that exact context.

Updates return bounded counters for host logging without command text. No typed
execution, gamestate seeding/reset, separate reliable transport or ordinary gameplay
session is implemented by this storage owner.

`new_commands` exposes precisely the staged new records, independently of retention
slot collisions. [Typed pre-game preparation](pre-game-controls.md) consumes that
selection, adds bounded inert disconnect/map/restart notices and returns them only
after history commit. It preserves cancellation and keeps unknown commands as keys.

The same preparation API accepts [mixed service plans](mixed-service-controls.md).
One UDP test stages commands and identities from the same complete plan, then
commits the checked outgoing acknowledgement and both owners. Its next frame proves
all three accepted sequence values and the new command obfuscation key. No peer
command text is executed by that path.
