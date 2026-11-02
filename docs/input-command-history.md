# Input command history

`user_commands::history::InputCommandHistory` retains checked native inputs in
caller-funded `InputCommandSlot` storage. The host selects retention capacity;
the native 128-command ring is not reproduced. Native packets still carry at
most 32 inputs. Engine retention and compatible packet limits are separate.

Each opaque `InputCommandId` binds an epoch, profile and positive 64-bit local
sequence. Local sequence numbers are independent of duplicate or wrapping native
timestamps. Hosts must assign unique world namespaces when reconstructing owners;
these labels provide provenance, not authentication.

Enqueue validates every field before changing storage. Full storage returns an
explicit error without overwriting unretired inputs. Latest, transmitted and
retired prefixes remain separate. Exact circular lookup is constant time and
checks the complete stored tag. `id_for_local_sequence` supports local replay and
batch selection without a second ID queue; it does not map native timestamps or
acknowledgements to local IDs.

## Sending and retention

`prepare_range` copies an exact contiguous retained range of one through 32 inputs
while exclusively leasing the original owner. Previously sent backups are allowed;
new inputs cannot skip an unsent prefix. Dropping the lease cancels transmission
admission without changing counters or storage. A compile-fail example verifies
that the original owner cannot be modified while this lease exists.

The lease's `movement` helper checks the prediction profile and optional snapshot's
world/profile before returning a batch for the existing owned control builder.
Prediction report time stays explicit rather than being inferred from the last
input. `MovementBatch` now lives in `user_commands` and remains reexported from
`control` for existing callers.

After actual complete frame I/O, consuming `commit_transmission` performs only
preflighted metadata writes. Backup-only packets do not regress the transmitted
prefix. Packet construction, cancellation and failed I/O do not admit a send.
Transmission does not retire input or claim that a server processed it.

`retire_through` releases only an already transmitted prefix under explicit host
reconciliation or retention policy. Repeating the same retirement releases zero
additional inputs. Reliable-command acknowledgements are a separate namespace and
cannot substitute for input IDs. Native snapshot-time reconciliation remains open.

Construction and reset invalidate records logically in constant time without
clearing physical slots. Reset requires a strictly newer epoch and can change
profile. Old IDs fail binding even when a new world's sequence restarts at one.
Old slot contents remain private and inaccessible through the new owner.

## Evidence and remaining work

Nine integration cases cover both profiles, 257 retained inputs, wraparound,
sent-only retirement, backups, cancellation, missing/range/capacity failures,
world/profile checks, storage lifetimes and opaque Debug. An independent queue
model runs 512 operations at four capacities in both profiles. A private unit case
checks local sequences crossing the 32-bit boundary and exhausting 64 bits without
changing native timestamps or wrapping counters.

The existing real UDP movement case now retains the input-owner lease through
complete I/O. A later validated reliable ACK releases its reliable command while
all three inputs remain retained. Snapshot fixtures are shared once in the core
test harness; no new test target or production callback is introduced.

Device sampling, backup scheduling, prediction production, authoritative
snapshot-to-input reconciliation, collision, physics, rendering and native peer
sessions remain separate implementation gates. A separate
[prediction history](prediction-history.md) now owns exact world-bound samples.
