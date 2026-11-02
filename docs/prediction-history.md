# Exact prediction sample history

`player_state::history::PredictionHistory` retains complete inert prediction samples
in caller-funded `PredictionSlot` storage. Capacity is independent of the native
128-entry ring. World epoch and profile are explicit; host labels must remain unique
across reconstructed owners and provide provenance rather than authentication.

Native 32-bit command times are exact keys. Insertion accepts arbitrary order and
wrapped values without imposing signed ordering, monotonicity or a half-range rule.
An older or nearby sample is never silently substituted. `resolve` checks the world,
decoded request profile and exact native time. `sample_at` supports explicit host
lookup within the current owner's profile. Neither operation maps times to input
IDs, processing evidence or reliable acknowledgements.

## Publication and replay

`prepare_insert` validates world/profile and requires a new exact-time key and free
cell. A duplicate returns an error even when words match. Full capacity rejects
without eviction. `prepare_replace` instead requires an existing exact-time key:
the host explicitly selects this operation after recomputing a prediction during
replay. It cannot create a missing record or increase retention count.

Both operations lease the original owner and hold a checked copy. Dropping cancels
all publication; commitment writes only the prechecked cell and count. Borrowed
samples prevent reset/replacement while used. Two compile-fail examples enforce
the update and lookup borrow contracts. A copied sample is still inert data, not
independent proof of a session or server identity.

`remove` explicitly releases one exact key. Backward cluster compaction preserves
other records without accumulating tombstones. No timestamp ordering or automatic
ACK retirement occurs. Construction/reset clear private cells in linear time;
reset requires a strictly newer epoch and can change profile. Producers must supply
the current epoch to insert/replace, so stale world labels reject before mutation.

## Storage and evidence

The table uses deterministic open addressing over raw time bytes, with no allocation,
external dependencies or serialized hash contract. Lookup and updates are bounded
by caller capacity; full occupancy is supported, while spare cells reduce probe
lengths. Cluster removal visits each other physical cell at most once after lookup.
The hash is for local engine keys and makes no cryptographic claim. Debug/errors
omit world labels, native times, coordinates and raw player words.

Seven integration cases cover both profiles, 257 retained samples, arbitrary/wrapped
times, full storage, explicit replay/cancellation, reset/reconstruction, stale
producers and real decoded whole-player-state request resolution. Every removal
order of a full five-cell table checks all remaining exact keys. An independent
ordered-map model runs 2,048 operations at five capacities in both profiles, including
full-table misses, cancellation, replacement, removal and a world reset.

The existing player-state transaction consumes an exact resolved sample and still
validates profile/time before publishing. [Complete snapshot admission](retained-snapshot-prediction.md)
now selects exact samples through an explicit borrowed world-bound history source,
after full body/carrier validation. Prediction production, collision/physics,
replay scheduling, authoritative input reconciliation and playable sessions remain
separate implementation work. Native prior fallback remains an explicit choice,
never a cache-miss policy.
