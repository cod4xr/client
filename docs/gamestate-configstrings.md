# Gamestate configstring blocks

`gamestate::configstrings::ConfigStringBlockPlan` validates explicit patches in
one gamestate `svc_configstring` block. The reader starts immediately after that
opcode. This constituent does not parse an entire gamestate, synthesize native
constant defaults, apply engine state or grant asset/path/command authority.

The locked native client provides two reviewed formats in `src/cl_main.c`,
`CL_ParseGamestateLegacy` and `CL_ParseGamestateX`; `src/q_shared.h` defines the
legacy namespace. Protocol 6 has a signed 16-bit positive count, then either an
increment bit or an absolute 12-bit index for each entry. Protocol 21 has signed
32-bit counts and indices. Both require strictly increasing identifiers within a
block, bounded respectively by 2442 and 4884 slots. Order starts anew per block.

Strings use the pinned shared big-string reader, preserving non-UTF-8 bytes and
normalizing percent to period. The additional native gamestate check admits at
most 8190 bytes excluding NUL, one fewer than the general big-string field bound.
Empty values are valid. Packed output omits terminators and can share offsets for
empty entries. Each view also retains whether it starts its original native block.
Public views retain wire identifiers as `u32` and expose only used
records/bytes. Debug and error formatting never includes private string contents.

The caller supplies a record/work budget, combined text budget, reusable private
string scratch, final text storage and opaque metadata slots. Count/profile/work
and metadata capacity checks occur before reading entries. A cloned reader then
validates the entire block, including ordering, field limits, all text extents and
final capacities. Only after success does a deterministic second pass over the
same immutable input populate final storage. It uses the same shared string codec
and scratch capacity, with every read and destination extent already proven.

On failure, final text, metadata and both original reader cursors remain unchanged;
scratch is unspecified. On success, the reader advances exactly past the block,
preserving legacy cached bits for subsequent fields. The returned plan borrows
only cooked storage, allowing the source frame and its reader to be released.
Whole-gamestate application must still wait for the enclosing message to validate.

These namespaces and field lengths are compatibility limits of a selected baseline,
not engine-wide memory/asset limits. The decoder inherits no native global string
arena and can read blocks larger than 128 KiB through `BodyReader`. Raising limits
for future custom peers requires an explicitly negotiated wire/content profile.

Ten focused tests cover independent native vectors, mixed bit/byte cursor state,
raw/normalized/empty values, source-independent plan lifetimes, full namespaces,
a 160 KiB block, maximum strings, invalid counts/order/IDs, every input truncation
and short caller capacity, budgets, dirty metadata and rollback of existing cache
state. Protocol-21 initial entity baselines now have a separate decoder. Default
merging, legacy coordinates, full gamestate dispatch and real ordinary-server
interoperability remain open.
