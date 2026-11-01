# Auxiliary reliable transport

`cod4xr-reliable` owns the previously absent auxiliary reliable body codec. It is
a dependency-free crate in the client repository, with no OS, game-runtime or
server-type dependencies. The existing pinned `cod4x-protocol` implementation
continues to own the outer marker/qport envelope. Ordinary channel fragmentation
and reliable client/server commands are separate protocols.

The source baseline is the locked public client's `net_reliabletransport.c` and
`.h`. The body contains signed little-endian sequence and cumulative ACK fields,
flags, up to three relative selective ranges, a signed sender-window hint, a
signed fragment length and the exact payload. Sequence -1 is ACK-only; data starts
at zero. Nonnegative sequence/ACK values must fit signed 32-bit fields. Range
offsets are nonnegative and lengths positive signed-short values.

`ReliablePacket::parse` borrows a complete body without allocation. It rejects
incomplete headers, impossible counts, negative fields, overlong payloads, hidden
suffixes and ACK-only payload data. The native fragment maximum is 1200 bytes;
the body maximum is 1226 bytes, excluding the shared six-byte envelope. A
zero-length data fragment remains representable. `encode` preflights and stages
the header before changing caller output. Debug output contains metadata and
lengths, never payload bytes.

All flag bits and the signed sender-window hint are retained. Only bit zero has
the known force-ACK interpretation. Parsing activates no feature, allocation,
send, timer or acknowledgement. The native implementation ignores the incoming
window hint; it must not become a peer-selected resource budget. Selective range
order and overlap are retained for later temporal validation against transmitted
data, rather than being accepted as acknowledgement authority by the codec.

The network receiver first checks the selected peer, shared envelope family and
original qport. `AuxiliaryMessage::reliable_packet` then parses only a reliable
body. Other auxiliary families cannot be cast into this protocol. Malformed
bodies are errors without channel sequence reservation or automatic responses.

Receive/send windows, complete ACK admission, pacing, retransmission, stream
assembly and application message framing remain to be implemented. No native
reliable peer exchange, gamestate delivery or playable connection is claimed.
