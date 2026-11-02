# Owned control-message frames

`ControlMessagePlan` builds baseline idle/ordinary-client-command bodies before
channel packetization. It owns the resulting private bytes, so caller command/key
storage can change or disappear afterward. Construction never marks a command
transmitted or commits a received acknowledgement. The
[movement constructor](movement-packets.md) adds complete native input batches to
this same owner. Inbound decoding, gamestate and the auxiliary reliable transport
remain separate boundaries.

The host chooses `ClientControlHeader::Legacy6` or `Cod4x21` explicitly. Discovery
protocol 6 and challenge marker 18 never select a layout. The legacy header has
one server-ID byte and two four-byte acknowledgements (nine bytes); the pinned
modern header has four four-byte fields (sixteen bytes). A retained server-command
key supplies its own command acknowledgement, preventing an independently typed
number from disagreeing with that key's sequence. Server/config identifiers retain
their original bit patterns; an ordinary message acknowledgement must fit 31 bits.

The command prefix contains three-bit client-command opcodes, four-byte sequences
and canonical NUL-terminated strings, followed by a three-bit EOF. Shared message
writing preserves the unusual cached-bit-byte behavior across intervening sequence
and string writes. The owned plan reuses shared static Huffman compression, then
shared client-to-server obfuscation, exclusively after the plaintext header.
The transform is compatibility behavior and supplies no encryption/authentication.

The named baseline uses a 2048-byte logical and encoded control-frame budget.
The oldest complete pending prefix is selected. If Huffman expansion exceeds the
wire budget, construction rebuilds a shorter complete prefix, bounded by the
128-command history; it never clamps encoded bytes or skips an older command.
A single command that could not fit would fail explicitly instead of silently
deferring all work. Deferred commands remain pending for a subsequent frame.
The result reports included count and last sequence for later transmit admission.
Movement packets separately report their user-command count and may defer the last
reliable command when compression expansion requires it, preserving every movement input.

`ServerCommandKey::from_retained` validates representation only; the host must
obtain it from validated incoming history. Native incoming string reading maps
percent to period, then stores at most 1023 key bytes; it leaves 0x92 unchanged.
The initial key is empty at sequence zero. No raw key/cookie/command enters Debug
or fixed errors, and no command is executed by constructing a frame.

`ClientSender::begin_control` verifies the original protocol and challenge before
reserving a channel sequence. The host must also use its current validated game
state when constructing the header. Send pacing remains one datagram per operation,
and the included command prefix may be marked transmitted only after the complete
frame was admitted to outgoing I/O. Acknowledgement still requires a separately
validated inbound message. A standalone connect command is not exposed yet.

Eight core tests use independent plaintext/Huffman/XOR golden vectors for both
headers, keyed/scattered opcode bits, owned source lifetimes, exact output capacity,
full 128-command serialization, representation/sequence boundaries and compression
expansion. A real UDP test verifies context rejection before reservation and a
fragmented control frame on the original acknowledged socket. Structural boundary
tests additionally decode through the already verified shared primitives. These
are library/loopback checks, not ordinary gameplay-server interoperability evidence.
