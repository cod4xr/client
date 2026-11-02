# Whole reliable gamestate bodies

`gamestate::ReliableGameStatePlan` validates a complete protocol-21 auxiliary
application body before publishing caller-owned gamestate output. Its input excludes
the four-byte length prefix already handled by reliable framing. It shares the
existing deterministic gamestate walker and codecs, adding no dependency on sockets
or the transport crate.

The reviewed body has a 32-bit gamestate command, embedded byte `svc_gamestate`,
complete gamestate constituent and final reserved 32-bit word. The native writer
appends zero for that final word; it is not an outer EOF byte. The plan retains
nonzero values as opaque bits without activating capabilities or content trust.
This follows the immutable public writer's
[gamestate send path](https://github.com/callofduty4x/CoD4x_Server/blob/ddd65fce1a1093017bad2dbb6840d75d11c0d56f/src/sv_client.c#L1676-L1694)
and the already locked client's `CL_ExecuteReliableMessage`/`CL_ParseGamestateX`.

The public writer file's normalized SHA-256 is recorded in this client repository's
`upstream.lock.json`. Its bytes were retrieved read-only through GitHub and hashed
in memory. No local server source files or server worktrees were read or edited for
that evidence. Hosted source verification checks this additional immutable file.

The selected profile and whole-body host budget validate first. A no-output walk
checks the embedded gamestate and all caller capacities; the final word and exact
physical byte exhaustion must then validate as well. Only after the entire wrapper
succeeds does the private publication walk write text/metadata/entity/client slots.
Any malformed prefix, nested record, missing tail, extra byte or exhausted capacity
preserves every final output. Private string scratch is unspecified on failure.

Unused bits in a consumed legacy bit byte are bounded native padding and are never
another command. Extra physical bytes reject, including zero bytes. Debug output
reports only body bytes, unused-bit count and gamestate record counts; checksum
words, text and raw entity state stay private. The cooked plan retains no input
frame borrow and may outlive explicit frame release.

Five core tests cover independent wrapper bytes, source-independent ownership,
zero/nonzero reserved words, inert partial-byte padding, every fixture truncation,
bad prefixes, all 256 possible appended byte values, whole-body budgets, short
capacities, unsupported profiles and a body above 128 KiB with late-error rollback.
A real loopback test reorders two framed application bodies, rejects the earlier
malformed body without publication, explicitly discards it, admits the valid next
body and retains its cooked plan after releasing the network frame.

That test also separates byte-transport ACKs from application validity: reliable
fragment receipt may be acknowledged while an application frame is later rejected.
Typed decoding never silently releases a frame or commits session/engine state.
`ReliableConnection::prepare_game_state` now provides an exclusive inspected-frame
handle: drop retains the frame; explicit commit releases it and returns the cooked
plan. This remains separate from session or engine application.
Reviewed native defaults now have a separate importer. Default/wire merging,
session sequence admission, content/application ownership,
other application commands and real ordinary-server gameplay remain open.
