# Atomic client identity state

`client_identities::ClientIdentityState` retains sorted owned protocol-21 identities
and their accepted config-data sequence. It borrows caller-owned cells and starts
without a baseline; it never assumes sequence zero or exposes old unused storage.
No global sixty-four-entry player array, heap allocation, name execution, UI update,
network acknowledgement or authentication decision occurs inside the component.
The identity decoder's native field/index widths remain separate wire boundaries.

`prepare_game_state` accepts a complete `ReliableGameStatePlan`, whose wrapper and
reserved tail have already passed validation. It privately stages a replacement,
using the gamestate's config-data sequence and own-client index. Last duplicate
identity records win. A checked local generation increments only on baseline commit;
replacement removes old retained IDs from the visible table. Generations are local
to this owner, without peer or credential authority.

`prepare_updates` stages an entire caller-decoded batch. A record applies only when
its sequence equals `accepted.wrapping_add(1)`. Every other value is ignored without
advancing that sequence, including forward gaps. Later records compare with the
prospective sequence accepted earlier in the same batch. The locked native receiver
uses exact-successor equality, so a generic newer-than policy would be incorrect.
Rust defines all thirty-two wire bits explicitly, including signed/high-bit and
unsigned-wrap transitions.

Input record budgets include duplicates/ignored updates. Retention budgets count
distinct records; private/final caller capacities remain independent. Sparse sorted
keys avoid allocating by identifier magnitude. Every record has already passed the
owned field decoder, and cloning only copies fixed inline wire-field arrays.
Private insertion uses bounded cell rotation, without allocation or external callbacks.

Preparation can fail late after several proposed replacements/inserts. Such failures
preserve every final identity cell, accepted sequence, own-client index and generation.
Private scratch is unspecified on failure. Dropping a successful preparation also
cancels all owner changes. `commit` copies the complete checked inline state and
metadata together, without fallible work or I/O. Unused final tails stay unchanged.
Staged records retain neither source frame nor gamestate storage; retained records
remain valid after those buffers and scratch are cleared. Debug/errors contain
fixed categories and metadata, without names or clan tags.

`PreparedGameStateFrame::with_client_identities` combines the original network-frame
lease with identity preparation. The resulting `PreparedIdentityGameStateFrame`
owns both exclusive leases. Failure/cancellation leaves the same frame ready and
identity state unchanged. Joint commit releases exactly that frame and applies the
staged identities together. It returns the cooked gamestate and copied change
metadata. Byte receipt ACKs remain separate; cooked gamestate output can remain
valid even when release/application is cancelled. The host still preflights any
sibling session/content resources and binds this owner to the correct connection.

Fourteen integration tests cover absent/dirty initialization, empty baselines,
duplicate last-wins behavior, source lifetimes, cancellation/replacement, full-width
sequence transitions, gaps/duplicates, all independent budgets/capacities, late
rollback, workspace reuse, unchanged tails and private output. A 256-trace ordered
map model checks thousands of batches with cancellation and wraparound. A local
generation test proves exhaustion does not reuse an epoch. A compile-fail example
proves exclusive owner access persists through preparation.

A real UDP test delivers reordered malformed/valid gamestate frames through the
original connection. It proves that late wrapper failure, identity staging failure
and joint cancellation leave the correct owners unchanged. Joint commit releases
one accepted frame and retains the normalized identity after all source/work buffers
are cleared. These are loopback/ownership checks, not a native gameplay peer claim.

`prepare_service_plan` stages all updates from a complete validated
[mixed ordinary-message plan](mixed-service-controls.md), checking the original
profile before applying the same exact-next rules. It retains no plan/input borrow
after staging. The host preflights command history and other sibling changes before
committing acknowledgement and the prepared owners. Callers of `prepare_updates`
still validate the whole enclosing message themselves. This table does not
configure remote CVars, apply entities/snapshots, authorize content or mark the
client playable. Legacy identities carried by configstrings need their own adapter.

Primary behavior is pinned in the public client
[receiver](https://github.com/callofduty4x/CoD4x_Client_pub/blob/17182c7bf04b859dea5c0db4acdb0b83c28d6dd2/src/cl_main.c#L7256)
and [gamestate loader](https://github.com/callofduty4x/CoD4x_Client_pub/blob/17182c7bf04b859dea5c0db4acdb0b83c28d6dd2/src/cl_main.c#L8256).
Their normalized source hashes remain in [the source lock](../upstream.lock.json).
The Rust state owner and joint transaction are independently written.
