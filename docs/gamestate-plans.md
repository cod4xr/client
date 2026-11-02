# Complete gamestate constituent plans

`gamestate::GameStatePlan` validates the complete protocol-21 gamestate constituent
starting immediately after `svc_gamestate`. Its header is the server-command
sequence. Its inner loop admits configstring blocks, entity baselines and client
identities through EOF. Its trailer is config-data sequence, checked own-client
index and checksum-feed bits. The schema follows locked `src/cl_main.c`,
`CL_ParseGamestateX`, without the optional, undefined `FALLBACK_SIGNALING` build path.
Protocol 6 remains explicitly unsupported here pending its complete carrier and
effective per-block map-center integration. Individual
[legacy baseline coordinates](legacy-baseline-coordinates.md) now have explicit context.

This is a constituent of an ordinary service message or reliable application
command. `ReliableGameStatePlan` now validates the complete reviewed auxiliary
wrapper; [ordinary gamestate carriers](ordinary-gamestate-services.md) now validate
supported surrounding controls through outer EOF before publication. Session/world
acknowledgement and engine application remain separate. Parsing stops exactly after the
trailer, preserving following service bytes and cached bits. Sequence/checksum
words retain all 32 wire bits and grant no ordering, authentication or file authority.

The caller supplies final text and opaque configstring/entity/client slots, private
string scratch and independent service, configstring, text, entity and client-record
budgets. Service work includes EOF. Native identifiers/field capacities remain wire
constraints; they do not allocate fixed engine player/entity tables or native global
string arenas. Repeated client IDs and configstring IDs across separate blocks remain
in original order. Ordering within each configstring block and the entity baseline
stream remains checked by the existing decoders. Per-record `starts_block()` metadata
and the plan's block count preserve native default scopes after source-frame release.

The native client reapplies its constants for every configstring block. An earlier
explicit value for a default ID therefore resets when a later block omits that ID;
an omitted ID without a native default retains its earlier explicit value. Applying
defaults once and then treating all patches as one flat stream would lose this
behavior. Empty gamestates with no blocks apply no implicit constants. No engine
updates occur merely from retaining these boundaries.

One private typed walker implements the format. Its first pass uses a no-output
sink, checking every constituent, trailer, budget and final extent. Only after that
whole pass succeeds does the same walker replay immutable input with identical
cursors, profile, budgets, capacities and scratch length into the final storage.
No external callback can see partial validation. Every replay read and destination
extent has already been proven. The original reader commits after full publication.

Any validation failure preserves all final text, metadata, entity/client slots and
both original reader cursors. Scratch is unspecified. Success returns a plan that
borrows only cooked output, permitting source-frame release. Unused output tails
and slots retain their previous values and are excluded from plan views. Debug
formatting reports counts only, excluding private text, entity words and checksum feed.

The plan contains explicit patches, not synthesized native constant defaults. Those
defaults live in the original executable and now have a separately hash-reviewed
content importer. Merging/applying defaults, sequences, resources or baselines to a running engine
remains separate from byte validation. Unknown inner opcodes reject without skipping
or executing their bodies. Client records omit the ordinary update's sequence prefix.

Nine tests cover independent minimal framing, source-independent output lifetimes,
mixed/interleaved records, global text offsets across blocks, repeated IDs, exact
following byte/bit fields, all 4884 configstrings and 1024 baseline entities, more
than 64 retained client records, a complete gamestate above 128 KiB, every fixture
truncation, invalid nested records/opcodes/trailers, unsupported profiles, dirty
output rollback, existing cached bits, every short capacity and every host-budget
boundary. No live ordinary-server session or playable client is claimed.
