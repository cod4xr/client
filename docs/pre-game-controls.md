# Typed pre-game controls

`PreparedServerCommands::with_pre_game` adds typed pre-game notices to an exclusive
command-history preparation. Only newly selected commands are inspected, in wire
order; duplicate/older commands emit nothing. A later record may reuse a retention
slot without erasing an earlier staged notice. The public `new_commands` iterator
also exposes that exact selected prefix for future reviewed typed handlers.

The native dispatcher uses the raw first command byte, rather than token zero:

| Byte | Typed notice | Host application gate |
| --- | --- | --- |
| `w` | Disconnect, optional reason and peer-supplied `PB` classification | Bind the original connection; escape/localize display text under local policy |
| `l` | Map/gametype request | Validate virtual content and session/replay policy before loading |
| `m` | Fast restart | Apply only in an active session, as in the reviewed native handler |

Uppercase, leading whitespace and a leading quote do not match those prefixes.
The complete first token need not equal the single letter. Unknown commands remain
retained for later handlers and the outgoing obfuscation key, without console or
script execution. No authentication result is inferred from a `PB` field.

A private borrowed lexer reads at most the first three arguments because those are
the only tokens consulted by these handlers. Native whitespace is bytes 1 through
32 except text markers 20, 21 and 22. Quotes strip their surrounding delimiters,
without backslash escaping; adjacent quoted/regular parts are separate tokens.
Unterminated quotes retain their remaining text. `//` ends the entire command,
including later newlines; `/* */` comments are skipped and interrupt regular tokens.
Unterminated block comments end tokenization. Comment markers inside quotes and
semicolons remain literal bytes. This is a typed command reader, not a command
buffer splitter or general console interpreter. Native global token/text-pool
exhaustion and nesting corruption are not reproduced.

Missing disconnect reason differs from an explicit empty quoted reason. The third
token selects `PB` only by exact ASCII-case-insensitive comparison after its native
1023-byte copy boundary. Missing map/gametype arguments are empty. `PreGameText`
retains each complete normalized token and exposes `legacy_bytes` separately: the
reviewed Windows handler copies 1023 reason bytes or 255 map/gametype bytes. Full
tokens remain available for future explicit expansion policy. Neither view grants
filesystem, translation, UI or command authority. Debug formatting contains lengths
and fixed categories, never text.

`PreGameBudget` independently bounds the sum of every newly admitted command's
bytes, including unknown commands, and the count of recognized notices. Preparation
returns an error on late budget exhaustion and drops the consumed history lease;
all retained text/metadata and acknowledgements remain unchanged. Dropping successful
typed preparation also cancels. The host can inspect staged notices to preflight
content/session resources, then commit the checked outgoing client-command ACK and
infallibly commit typed history. Commit returns `CommittedPreGameCommands`, carrying
the history summary and inert notices. It does not apply a disconnect/map/restart
effect. No callbacks or I/O run inside preparation or commit.

Notices borrow cooked plan bytes, independently of the network packet, decompression
scratch and history backing storage. They remain available after history slot reuse
or owner release. The caller keeps that cooked output alive until typed consumers
finish. Exclusive owner borrowing prevents changing history during preparation;
a compile-fail example checks this contract.

Fifteen core tests cover both profiles, exact prefix dispatch, missing/empty/raw
arguments, percent normalization, `PB` rules, native copy boundaries without losing
full text, comments/quotes/no escapes, every non-NUL control-byte category, every
prefix of comment/quote forms, independent budgets, cancellation, duplicate/gap
selection, slot collisions, unknown-key retention, input/history lifetimes and
private output. A bidirectional UDP test first exhausts the third-notice budget,
then cancels a successful preparation, proving no pending command is acknowledged.
After retry/commit, all three typed notices remain available despite one retained
slot. The peer decodes the next datagram with message ACK 7, command ACK 3 and the
actual newest key `m`. No native gameplay peer or engine effect is claimed.

Primary evidence is the pinned public client
[pre-game handlers and dispatcher](https://github.com/callofduty4x/CoD4x_Client_pub/blob/17182c7bf04b859dea5c0db4acdb0b83c28d6dd2/src/cl_main.c#L7134),
[tokenizer](https://github.com/callofduty4x/CoD4x_Client_pub/blob/17182c7bf04b859dea5c0db4acdb0b83c28d6dd2/src/cmd.c#L201)
and [native path width](https://github.com/callofduty4x/CoD4x_Client_pub/blob/17182c7bf04b859dea5c0db4acdb0b83c28d6dd2/src/q_shared.h#L297).
Their normalized hashes are recorded in [the source lock](../upstream.lock.json).
The Rust lexer, typed representation and transaction are independently written.
