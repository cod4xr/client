# Protocol-21 entity delta records

`entity_deltas::EntityDeltaDecoder` decodes individual protocol-21 entity records
against a caller-selected prior state. It shares the existing generated eighteen
field catalogs, primitive readers and checked ordered-index codec with initial
baselines. No second wire codec, entity table, C struct layout or native pointer
cast is introduced. Protocol 6 remains unsupported until its coordinate/map-center
reader has separate conformance evidence.

`prepare` reads the next index into private candidate reader/index state, retaining
exclusive borrows of both originals. Its number lets the host look up a prior state
without callbacks or indexing an engine array from peer input. `decode` accepts
that matching state or explicit `None` for zero reference, plus the complete message
time bits. A supplied different entity number rejects before commit. Dropping the
preparation, malformed fields, truncation and invalid counts leave both original
owners intact, including cached unread bits. Complete success commits both and
returns an owned state or removal record. Two compile-fail examples protect both
exclusive borrows.

`EntityState::from_baseline` copies an accepted initial baseline. `EntityWord` shares
the existing sixty semantic word names, independently of native union layout. The
delta decoder retains all prior words outside the changed prefix. Its decoded type
selects subsequent target-layout field order; unknown types use layout 17. Native
hint 2 fields omit an outer changed flag, including their absolute or relative
value forms. Owned results retain no reference, source frame or decoder borrow.
Removing a missing host entity, reference snapshot selection and connection/session
epoch binding remain explicit host policy.

The shared value reader now takes prior-word and message-time context. Initial
baselines still call it with zero reference/time and preserve their existing exact
vectors. Changed ordinary integers mask the prior value to their field width before
XOR; raw float forms XOR their complete prior words. In particular, narrowing a
type-dependent field does not retain high bits from its prior layout. Signed
eight-bit fields XOR the low prior byte and sign-extend the result. Native
zero markers reset the value rather than XORing it. Compact floats use the prior
truncated integer component and a thirteen-bit bias/XOR representation. A compact
form with a nonfinite or out-of-range prior integer component rejects; the reviewed
writer selects a raw form for such prior states. Fractional prior values truncate
before that range check. Raw float XOR preserves arbitrary bits, including NaNs.

Single-bit flag updates toggle the prior flag, while full 24-bit flag values replace
it. Relative time uses wrapping subtraction from message time, independently of the
prior field. Angles use an absolute signed 16-bit turn converted to degrees, including
negative angles in the high half of the wire word. Ground references and event bytes
use their absolute forms.
Protocol-21 coordinates are absolute binary32 words, not XORs or legacy map-biased
coordinates. No decoded coordinate, identifier, NaN or union word becomes an engine
object merely by passing this record codec. Debug/errors expose fixed categories,
numbers and counts, not raw state values.

Ten integration tests cover independent physical no-delta/removal bytes, owned prior
state retention, every target layout and unknown fallback, type changes, nonzero
tail preservation, signed XOR/absolute coordinate distinction, reference mismatch,
cancel/retry, every truncated record, invalid compact references, late count failure
with existing cached bits, ordering/reset and high-index exhaustion. One test's
32-by-32 writer-grounded player-field trace checks full/no/partial changes across
explicitly reset streams. Three private field tests add independent nonzero vectors
for integer/float/angle/flag/time/ground/event/coordinate forms, short inputs, exact
unchanged-word retention and compact reference boundaries. Existing baseline tests
continue to pass, with new signed high-angle vectors for the shared native rule.
These are source-grounded synthetic checks, not native peer traces.

The primary evidence is the pinned public client
[field writer](https://github.com/callofduty4x/CoD4x_Client_pub/blob/17182c7bf04b859dea5c0db4acdb0b83c28d6dd2/src/msg.c#L2182),
[entity record writer](https://github.com/callofduty4x/CoD4x_Client_pub/blob/17182c7bf04b859dea5c0db4acdb0b83c28d6dd2/src/msg.c#L2436)
and [flag writer](https://github.com/callofduty4x/CoD4x_Client_pub/blob/17182c7bf04b859dea5c0db4acdb0b83c28d6dd2/src/msg.c#L360).
The normalized writer hash is already locked. The public client's reader entry
points delegate to the native binary. A read-only audit of the previously reviewed
installed image checked entity/field control flow, including the prior integer
width mask and signed angle conversion absent from the writer's description.
[Metadata evidence](entity-delta-reader-evidence.json)
records its exact image hash, addresses and reviewed rules, without binary bytes or
disassembly. The sixteenth client source hash binds its public assembly entry point.
Protocol-21 coordinate behavior follows the public C reader/writer; the stock
installed coordinate routines retain their distinct legacy representation.
Complete snapshot framing, stream terminators,
player-state/client-state deltas, host snapshot storage, interpolation/prediction and
engine application remain separate work. Individual record success never advances a
network ACK, admits a complete snapshot or makes the client playable.
