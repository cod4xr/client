# Native snapshot headers

`snapshots::SnapshotHeader::parse` reads the six physical byte fields immediately
after service opcode six: little-endian 32-bit server time, unsigned delta-distance
byte and unsigned flag byte. The player-state stream follows immediately; the
reviewed native reader has no area-mask length/payload in this position. Both
explicit baseline profiles use this framing.

The result owns all fields without retaining input. Time preserves all 32 bits;
every flag bit remains inert and available to later typed policy. A zero distance
becomes `SnapshotDelta::Full`; all nonzero distances are represented by
`SnapshotDelta::Previous(NonZeroU8)`, including values beyond the native 32-slot
snapshot ring. This expands no native wire width and imposes no engine history
capacity. The distance is relative to the containing server **message sequence**,
not the most recently received snapshot or a local arrival index.

The parser uses a private candidate reader and commits only after every header
field succeeds. Truncation preserves the original byte position and cached-bit
value. Header byte operations neither consume nor align cached bits; following
player-state bits can reside in an earlier partial byte, with more bit bytes after
the intervening header. Debug output contains profile/reference-category metadata,
excluding raw server time and flags.

The installed native reader binds the header to its global message sequence,
resolves delta references through a 32-slot ring and checks stored sequence and
parse-ring age. It also treats a nonpositive computed native reference as a full
snapshot. This constituent preserves the wire selector without reproducing signed
sequence fallback, silently substituting a reference or approving history. A
containing owner must validate message sequence semantics, connection/map epochs,
reference availability and retention policy before resolving nonzero distances.
No reference math or ring access occurs here.

Seven cases cover independent physical vectors, all time bits, every nonzero
distance and all flag patterns, every truncated header, cached bits and following
bytes, owned lifetime/private Debug and immediate/straddling player-state handoff.
[Audit metadata](snapshot-header-reader-evidence.json) binds framing and reference
boundaries to the reviewed image and locked public client sources.

This admits only a header constituent. Complete snapshot composition/history,
message/ACK admission, server-command association, simulation and playable sessions
remain open.
