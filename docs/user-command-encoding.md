# Native full-form user-command encoding

`user_commands::AbsoluteUserCommandPlan` encodes one checked native movement/action
command using shared message primitives. `UserCommandInput` carries explicit native
timestamp/button/angle/weapon/offhand words, canonical forward/right axes and optional
selected-location/melee data. Wire widths remain separate from future engine input,
resource identifiers and negotiated extensions. No engine action or I/O occurs here.

Construction checks the twenty-one-bit button word, seven-bit weapon/offhand IDs and
agreement between conditional action presence and button bits. Angles and melee yaw
are explicit sixteen-bit turn words; the encoder performs no float conversion or
implicit rounding. Selected-location bytes remain raw, including 0x92 and 0xff.
The supplied compatibility key is private protocol context, not authentication.

The initial encoder uses absolute timestamp and full command form. Every common
field is explicitly transmitted. Native conditional fields transmit only with their
enabled button; disabled fields retain the receiver's previous inert values. The
attack bit, pitch/yaw and movement mask use the original key. Subsequent fields use
that key XOR the current timestamp. Changed-field markers are unkeyed, while native
branch/action bits follow their keyed rules. The distance uses the bit cursor;
selected-location and short values use byte operations with the cached bits intact.

`encode` uses bounded private scratch, then copies the whole result into caller
output. Any error preserves every final byte, including unused tails. Exact standalone
sizes range from 17 to 22 bytes. Debug/errors exclude keys, timestamps, angles,
inventory IDs, axes and action values. Libraries install no logging subscriber.

Standalone command byte slices cannot be concatenated into a native batch: cached
bit bytes span interleaved byte fields and adjacent commands. The same private
writer implementation is now used by [complete packet assembly](movement-packets.md),
including batch count, packet-derived key, prediction trailer and reliable prefix.
Input history, authoritative input-send bookkeeping, prediction production, compact
forms and playable sessions remain separate work.

Six integration cases cover an independent physical full-zero vector, all key bits,
both timestamp signs/extremes, nonzero baselines, all canonical axis pairs, every
conditional action form, unsigned turns decoded as signed angles, raw location bytes,
width/presence rejection, exact capacities, all shorter outputs, unchanged tails and
private Debug. Shared `user_command::read_delta` reconstructs the output; its reviewed
decoder is reused rather than copied. A private unit case checks every initial bit
cursor width from zero through seven, intervening bytes and following bit/byte tails.

The public packet writer calls the same native command writer for its selected
legacy/CoD4X header path: [CL_WritePacket](https://github.com/callofduty4x/CoD4x_Client_pub/blob/17182c7bf04b859dea5c0db4acdb0b83c28d6dd2/src/cl_main.c#L4246).
The [native thunk](https://github.com/callofduty4x/CoD4x_Client_pub/blob/17182c7bf04b859dea5c0db4acdb0b83c28d6dd2/src/callbacks.asm#L791)
names the reviewed installed-image entry. [Audit metadata](user-command-writer-evidence.json)
records its identity and control-flow/field-helper addresses; no executable, game
values or disassembly is distributed. Shared command types/decoder remain pinned
to revision `c3bd151068465a40e11e3db9d616789fdffa146f`.
