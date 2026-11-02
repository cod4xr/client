# Native player-objective sections

`player_objectives::SnapshotObjectives::parse` decodes a complete native objective
section for an explicit protocol-6 or protocol-21 profile. It takes an optional
immutable prior set, returns fully owned inert words and commits both input cursors
only after the whole section succeeds. `None` means an explicit zero reference.
A prior set from another profile rejects before any read. Connection, map and
snapshot sequence binding remain host requirements beyond that profile check.

The leading bit either retains the entire prior set or introduces all sixteen
native objective slots. Each transmitted slot has three raw status bits and a
separate delta bit for six words: origin X/Y/Z, icon, entity and team. A changed
entry has all six per-field flags, with no prefix count or sparse slot index.
Status-only changes preserve every prior word, including when status becomes zero.
Values 6 and 7 remain raw wire statuses requiring later gameplay validation.

The owned result has fixed schema-sized storage, so peer lengths never allocate
memory. Sixteen slots and the 12/10/4-bit identifier widths are native wire
constraints, not engine-wide objective/resource ceilings. Origin fields reuse the
shared ordinary float rules, including prior XOR, raw NaNs, signed zero and compact
integer truncation. They do not use legacy map-center coordinate bias. Debug/error
formatting excludes coordinates, resource identifiers and raw words.

Parsing clones the prior set privately and reads through a candidate `BodyReader`.
Failure in any slot drops the unpublished candidate while preserving the entire
prior set, original byte cursor and cached unread bits. Success returns data that
outlives both prior and network bytes. Following byte and cached-bit fields remain
available without alignment. This section is not an engine effect, a complete
player state, a snapshot acceptance or a network acknowledgment.

`tools/import_objective_table.py` reproduces all six field definitions and verifies
the sixteen-slot declaration from hash-locked public `msg.c`/`q_shared.h`. A
read-only audit of the reviewed installed image also compared every native table
offset, width and hint and checked the parent sixteen-entry loop and leaf reader.
[Audit metadata](player-objective-reader-evidence.json) records exact image identity
and addresses without executable bytes, disassembly or game values.

Fourteen integration tests cover independent physical vectors, both profiles,
status-only and unchanged retention, every field and semantic mapping, float forms,
zero clearing from nonfinite references, cached/following fields, every truncated
full-section prefix, late invalid-reference rollback, profile binding, owned
lifetimes, private Debug output and 128 independent modeled traces per profile.
[Stats, ammunition and weapon-model sections](player-arrays.md) now have separate
coverage, along with [complete HUD sections](player-hud.md). Player scalar fields,
prediction provenance and whole-snapshot/reference-history admission remain
separate work.
