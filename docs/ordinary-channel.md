# Ordinary channel ownership

The client channel consumes an acknowledged socket and preserves the original
challenge cookie, protocol and qport for the later message codec. Its sender and
receiver reuse `cod4x-transport` from the same immutable Git revision as the other
shared libraries. This does not edit server source or create a second channel codec.

The source-grounded envelope is little endian: sequence (four bytes), client-to-
server qport (two bytes), and optional fragment offset (four bytes) and length
(two bytes). A full fragment carries 1300 bytes; exact multiples require a final
zero-length fragment. Incoming sequence begins at zero; outgoing begins at one.
Ordinary server packets do not contain qport. The caller supplies bounded fragment
storage, up to the existing 128 KiB profile, before creating the channel.

The channel can split into borrowed send/receive handles. One outbound message
borrows the sender and caller bytes; each operation sends at most one datagram
under an absolute deadline. The host owns pacing and may service receives between
fragments. Cancellation keeps its reserved sequence, so a later message cannot
reuse it with different contents. After an I/O failure that transmission terminates;
the next message uses a fresh sequence. No automatic fragment retry is invented.

Each receive operation inspects at most one complete UDP datagram. The host supplies
an unchanged absolute deadline and processes the event in a scoped callback; the
borrowed bytes cannot outlive receive/reassembly storage or escape through the
callback result. Duplicate/out-of-order packets, capacity failures and malformed
envelopes produce fixed drop categories. Partial storage can be discarded explicitly
when the host's reassembly deadline expires. There is no hidden receive loop that
extends a deadline under traffic. Debug output contains counters and lengths only.

Connectionless, update-proxy and separate reliable markers are classified before
the fragment bit. Reliable auxiliary packets must retain the selected qport.
Their bytes remain opaque; classifying them is not implementing that transport or
executing OOB commands. Ordinary message bodies remain opaque too: reliable command
acknowledgements, profile-specific headers, obfuscation, compression, gamestate and
snapshots need their own checked message layer. The challenge cookie is kept out
of Debug/log output, but remains owned context for that future codec.

[Control-message planning](control-messages.md) now supplies owned outgoing idle/
command bodies for explicit legacy/modern headers. `begin_control` checks protocol
and challenge before reservation; raw `begin` remains an explicit low-level entry
for already encoded bodies. Incoming messages and auxiliary reliable payloads
remain opaque at this channel boundary.

No public-server join or playable session is claimed. Acceptance for this boundary
uses independent UDP packets to establish original-socket handoff, signed-qport bits,
sequence reservation, fragment lengths/terminators, receive ordering/capacity,
auxiliary classification, absolute timeouts and private-byte Debug behavior.
