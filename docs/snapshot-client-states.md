# Snapshot client-state records

`client_states` decodes the native snapshot client-state record format for explicit
protocol-6 and protocol-21 profiles. This schema is separate from protocol-21
opcode-11 identity updates: its name is sixteen packed raw bytes, with no string
normalization, UTF-8 assumption or terminator requirement.

`ClientStateDecoder::prepare` includes the leading more-records bit. A zero bit
returns `None`, commits just that end bit and marks the decoder ended. A one bit
prepares the ordered six-bit client index under an exclusive reader/index lease.
Client 63 is valid. The shared ordered index codec rejects duplicates, descending
indices and increment overflow without modifying either owner. There is no implicit
alignment before or after this substream; cached bits survive intervening bytes.

The prepared number lets the host select an immutable prior `ClientState`. Passing
`None` supplies zero words, rather than consulting entity baselines. Complete
decode returns owned inert state/removal and commits both cursors together. Drop,
reference mismatch, malformed counts and truncated fields preserve the original
more bit, byte cursor, cached bits and index owner. Reset starts a new stream while
retaining the explicit profile. An ended stream refuses further reads until reset.

Twenty-four semantic words cover team, model/attachment indices, packed name,
rank, prestige, perks, sprint multiplier and vehicle references. Semantic storage
order is independent of wire order and C offsets. Omitted words retain their full
prior value. Present records carry either no delta or a five-bit field-prefix
count; the native reader accepts a zero-length prefix. Counts above 24 reject.
Changed fields reuse the same position-independent integer/float value rules as
entity deltas, including masked narrow references, raw float XOR, signed zero and
compact float truncation. Removal consumes no field payload.

The field catalog is reproduced by `tools/import_client_state_table.py` from
hash-locked public `msg.c` and `q_shared.h`. All 24 native catalog offsets, widths
and hints were also compared against the reviewed installed image. The native
client stream, generic delta reader and index reset were inspected read-only;
[audit metadata](snapshot-client-state-reader-evidence.json) records image identity
and instruction addresses without executable bytes or game data.

Fifteen integration tests cover physical byte vectors, both profiles, every field,
raw name bytes, float forms, prefix retention, removal, complete source-independent
ownership, all 64 incremented client indices, cancellation, end/reset behavior,
every truncated full-record prefix, cached-bit rollback, invalid counts/references,
ordering failures and 128 modeled update traces per profile. Compile-fail examples
prevent mutating either leased owner during prior-state selection. Debug/error
output contains only structural metadata.

This record decoder does not merge a complete client-state substream against a
prior snapshot or admit a complete snapshot/message. Player-state decoding,
epoch-bound reference history, typed resource validation, identity reconciliation
and gameplay application remain separate. Six-bit indices and field widths are
legacy wire constraints, not engine-wide allocation limits or extension promises.
