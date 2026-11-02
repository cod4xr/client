# Complete gamestate constituent plans

`gamestate::GameStatePlan` validates the complete protocol-21 gamestate constituent
starting immediately after `svc_gamestate`. Its header is the server-command
sequence. Its inner loop admits configstring blocks, entity baselines and client
identities through EOF. Its trailer is config-data sequence, checked own-client
index and checksum-feed bits. The schema follows locked `src/cl_main.c`,
`CL_ParseGamestateX`, without the optional, undefined `FALLBACK_SIGNALING` build path.
Protocol 6 remains explicitly unsupported here pending its map-center coordinates.

This is a constituent of an ordinary service message or reliable application
command. The enclosing stream/envelope still needs complete admission before
session acknowledgement or engine application. Parsing stops exactly after the
trailer, preserving following service bytes and cached bits. Sequence/checksum
words retain all 32 wire bits and grant no ordering, authentication or file authority.

The caller supplies final text and opaque configstring/entity/client slots, private
string scratch and independent service, configstring, text, entity and client-record
budgets. Service work includes EOF. Native identifiers/field capacities remain wire
constraints; they do not allocate fixed engine player/entity tables or native global
string arenas. Repeated client IDs and configstring IDs across separate blocks remain
in original order. Ordering within each configstring block and the entity baseline
stream remains checked by the existing decoders.

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
defaults live in the original executable and require a separately verified content
import. Applying sequences, defaults, resources or baselines to a running engine
remains separate from byte validation. Unknown inner opcodes reject without skipping
or executing their bodies. Client records omit the ordinary update's sequence prefix.

Nine tests cover independent minimal framing, source-independent output lifetimes,
mixed/interleaved records, global text offsets across blocks, repeated IDs, exact
following byte/bit fields, all 4884 configstrings and 1024 baseline entities, more
than 64 retained client records, a complete gamestate above 128 KiB, every fixture
truncation, invalid nested records/opcodes/trailers, unsupported profiles, dirty
output rollback, existing cached bits, every short capacity and every host-budget
boundary. No live ordinary-server session or playable client is claimed.
