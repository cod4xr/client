# Read-only native system info

`system_info::SystemInfo` interprets native configstring 1 (`CS_SYSTEMINFO`) from
the caller-owned effective config snapshot. It can also validate explicit bytes.
It reuses the pinned shared `InfoString` big-info codec rather than implementing
another key/value grammar. Host byte and field inspection budgets are explicit;
the reviewed baseline's 8191-byte encoded limit still applies.

The reviewed native client's `CL_SystemInfoChanged` reads `sv_serverid` to supply
subsequent control headers. The new view requires exactly one case-insensitive
occurrence of that critical field. Decimal signed-32 and unsigned-32 forms retain
all identifier bits, including zero and the high bit. Optional signs and leading
zeros are accepted; whitespace, trailing junk, non-ASCII digits and overflow reject.
This intentionally rejects ambiguous duplicate identifiers and malformed numeric
text that native `atoi` can interpret loosely. Unknown fields retain native
first-match, case-insensitive lookup and original byte encoding.

Parsing validates the entire info string before returning any metadata. The
result borrows cooked config storage, never the original network frame. Debug
output contains encoded length and field count only; errors contain fixed
categories. No raw info/pair iterator with a text-revealing derived Debug is exposed.

Unlike native `CL_SystemInfoChanged`, this layer applies no arbitrary remote CVar,
mounts no archive and treats path/resource/extension fields as inert bytes. A parsed
server ID does not commit a session epoch, admit message or config acknowledgements,
select a protocol, enable extensions or establish gameplay compatibility. Those
operations require their own explicit validated ownership transitions.

Nine tests cover signed/unsigned edges, complete decimal validation, unknown/raw
fields, duplicate critical keys, full trailing-field validation, every forbidden
control byte, exact host and big-info limits, metadata-only errors/Debug and missing
native records. A wire integration decodes a gamestate, merges its configstring and
releases all original wire/plan storage before reading the effective system info.

Source evidence is the immutable client reference recorded in `upstream.lock.json`:
[`q_shared.h` configstring identifiers](https://github.com/callofduty4x/CoD4x_Client_pub/blob/17182c7bf04b859dea5c0db4acdb0b83c28d6dd2/src/q_shared.h#L1248-L1249)
and [`CL_SystemInfoChanged`](https://github.com/callofduty4x/CoD4x_Client_pub/blob/17182c7bf04b859dea5c0db4acdb0b83c28d6dd2/src/cl_main.c#L8523-L8616).
These files were inspected in the read-only client reference, without local server
source access or changes.
