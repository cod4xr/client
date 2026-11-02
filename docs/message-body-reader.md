# Complete-body reader views

Native reliable application messages can approach 1 MiB, while the pinned shared
primitive reader accepts backing views of at most 128 KiB. `BodyReader` in the
deterministic client core adapts a complete borrowed body to those bounded views.
The caller supplies the whole-body byte budget. No remote length allocates memory,
and the adapter changes no ordinary-channel or legacy field limits.

All byte/integer/float/bit/string/index decoding remains in the existing immutable
`cod4x-protocol` implementation. Byte-only operations use fresh bounded views at
the current byte cursor. Complete byte-span reads preflight the entire source
before copying through shared views. Strings retain the shared normal/big budgets,
percent normalization and raw-byte behavior without assuming UTF-8.

Legacy bit reads require a separate cursor: an old partial bit byte stays available
after intervening byte reads. For bit/mixed fields, the adapter presents a synthetic
one-byte cache slice followed by the fresh byte-cursor slice to the shared split
reader. A delegated seed read positions that cache. After the field, at most three
cloned, non-consuming probes recover the remaining cache length; another shared
read retains its residual value. The adapter owns view/cache bookkeeping, never a
second implementation of field bit extraction or legacy index forms.

Shared concrete `LegacyIndexDecoder` operations therefore work across rebasing as
well. A copied decoder candidate commits only after the complete view operation,
preserving both reader and ordering state on failure. No arbitrary reader callback
or rewind is exposed to a caller. Primitive errors carry bounded sizes/categories,
and debug output includes only input length, byte position and cache-bit count.

Seven tests cover an independent cached-bit/byte vector at offsets around 128 KiB
and beyond 1 MiB, a differential mixed primitive trace against the pinned reader,
a megabyte byte span followed by maximum shared big-string normalization, input/
output bounds, invalid widths, missing terminators, concrete entity/client index
forms and rollback of both index and cache state. Full-body size is an explicit
host budget; legacy string and index namespaces remain compatibility constraints.

This is a prerequisite for typed gamestate/application decoding. It does not itself
parse a complete gamestate, apply configstrings, allocate assets or activate an
extended protocol. Wider engine storage and negotiated extensions remain separate
from the selected peer's legacy field representation.
