# Ordinary server-payload decoding

`ServerPayloadDecoder` uses a reviewed `BaselineProtocol` and the original private
challenge. `ClientChannel::server_decoder` obtains these from acknowledged socket
context; unknown connect versions reject without treating the challenge suffix or
discovery advertisement as a message-profile negotiation.

Both selected profiles retain the pinned ordinary server body: a four-byte plaintext
client-command acknowledgement, followed by transformed static-Huffman bytes. The
host supplies one complete message from the selected peer's ordinary channel and
its validated positive 31-bit sequence. Socket routing, sequencing and reassembly
precede this decoder. The separate reliable transport is outside this boundary.

Before transforming bytes, decoding checks the peer number against transmitted
and retained command history. Queued-but-unsent, future and backwards references
cannot select arbitrary ring contents. Shared server-to-client obfuscation uses
the exact canonical retained command and original cookie/channel sequence. Shared
decompression preserves native byte-framed padding symbols after logical EOF.

All storage belongs to the caller: one private scratch slice for transformed bytes
and one decoded destination. There is no allocation based on a remote length field.
The baseline input and decompression budgets remain 128 KiB; these are protocol
profiles, not engine memory limits. Every failure preserves decoded output and
history. Scratch is private work space and may contain transformed data after an
error; discard/reuse it without consuming or logging its contents.

The result is `DecodedServerPayload`, deliberately before service-opcode validation.
It retains a history borrow while the view/body is inspected and cannot release
its command key through a concurrent mutable history operation. It does not commit
acknowledgement, execute server commands, select paths or initialize gamestate.
The message layer must validate all service records through logical EOF before
explicitly committing the checked acknowledgement. Huffman padding alone cannot
establish integrity or authority, and this transform provides no authentication.

Eight core tests establish independent initial/keyed vectors for both profiles,
native padding, exact key selection without commit, opaque/private bytes, unsent/
future/regressing numbers, every short scratch/output capacity, incomplete/empty/
oversized inputs, sequence bounds and reserved Huffman symbols. Two independent UDP
tests establish original cookie/profile use after socket handoff and unknown-profile
rejection without fallback. A compile-fail documentation example checks the history
borrow during decoded-view inspection. No ordinary gameplay peer was executed.
