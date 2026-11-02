# Ordinary gamestate service carriers

`gamestate::services::GameStateServicePlan` validates one complete protocol-21
gamestate inside an actual decoded ordinary payload. The supported outer stream
contains NOPs, server commands and sequenced identity updates before/after one
gamestate, ending at outer logical EOF. The inner gamestate retains its own EOF
and complete trailer. Neither terminator can substitute for the other.

The schema follows the pinned public client's
[ordinary dispatcher](https://github.com/callofduty4x/CoD4x_Client_pub/blob/17182c7bf04b859dea5c0db4acdb0b83c28d6dd2/src/cl_main.c#L7365)
and [protocol-21 constituent](https://github.com/callofduty4x/CoD4x_Client_pub/blob/17182c7bf04b859dea5c0db4acdb0b83c28d6dd2/src/cl_main.c#L8256).
Downloads, snapshots, duplicate gamestates and unsupported outer services reject
without skipping bodies. Bytes after outer EOF remain native padding. Protocol 6
still requires its separate trailer and per-block map-center work.

## Validation and publication

The existing private control scanner checks surrounding records and captures their
source spans. The same private gamestate walker first checks its complete
constituent with a no-output sink. Only after the entire outer tail/EOF passes does
closed immutable replay publish gamestate records, normalized commands and sequenced
identities. No host callback or public partial validation token is introduced.

All six final output areas remain unchanged on failure: gamestate text, configstring
metadata, entity baseline slots, inner client records, outer command bytes and outer
identity updates. Unused tails remain unchanged on success. Command/configstring/
identity scratch, final capacities and inner/outer budgets remain independent.
Private scratch is unspecified on failure; peer lengths never allocate engine storage.

## Order and ownership

`controls()` retains outer controls in original wire order. Its service count
includes the gamestate opcode and outer EOF, excluding inner constituent records.
`prefix()` supplies the counts of controls, commands and identity updates before
the gamestate opcode. The host must preserve this boundary around the inner
server-command baseline and future world transition. Inner unsequenced client
records and outer sequenced identity updates remain separate types and storage.

`GameStateRecords` separates complete cooked configstrings/baselines/client records
from protocol-specific trailer metadata. Existing `GameStatePlan` accessors remain
available and delegate to that coherent view. Block boundaries, repeated patches
and native default scopes survive. Its private constructor runs only after
constituent validation. A compile-fail example protects final-storage borrowing
after the containing plan is released.

Only cooked outputs are retained: packet, decode workspace, key-history and scratch
borrows end on success. Debug/errors exclude strings, identities, baseline words
and checksum feed. Parsing never releases an original outgoing key or ACK.

Seven integration cases cover an independent physical minimal carrier, mixed
inner/outer identities, command boundaries, source-independent records, untouched
tails, padding, both EOFs, every nonempty fixture truncation, late invalid opcodes/
trailers, profile rejection, six output capacities, all inner/outer budgets and
every short private scratch extent. Empty payloads reject in the earlier payload
decoder before this carrier is reachable.

Original-owner ACK leasing through host world/resource preflight, ordered epoch/
command/identity application, broader routing, the game loop and ordinary native
peer sessions remain open. This is complete wire-carrier validation; gameplay
admission and playable joins still require the host lifecycle and engine.
