# Auxiliary reliable receive window

`cod4xr_reliable::receive` adds deterministic receive storage and stream reads over
caller-owned byte and metadata buffers. Construction checks their complete exact
geometry before clearing metadata; dirty backing bytes are never exposed. The
native receive span is 20 fragments. Hosts can explicitly fund other spans up to
32767, the signed-short selective-field limit. Incoming sender-window hints never
select this capacity or allocate memory. Larger internal capacity does not change
the baseline wire profile or permit sending unsupported ranges to ordinary peers.

`ReceiveWindow::prepare` accepts a fully parsed `ReliablePacket` and preflights
only its incoming data sequence. It does not validate or apply the packet's
outgoing cumulative/selective ACKs. The host must validate those separately and
then commit both updates. Preparation holds exclusive receive storage plus the
borrowed immutable payload; dropping cancels it without changing bytes, tags or
counters. `PreparedFragment::commit` is infallible under these borrows.

First fragments in the current span own their sequence-tagged slots. Later
duplicates retain the original bytes, including a partially read head; old
fragments cannot reintroduce consumed data. Future out-of-window fragments fail
without mutation. The last consumable sequence is signed maximum minus one, so
the cumulative ACK remains representable; further data requires reconnecting
rather than wrapping into the ACK-only sentinel.

`read(output, max_fragments)` copies contiguous stream bytes, stopping at a gap,
output capacity or the explicit work quota. Empty fragments consume quota. Empty
output or zero quota does no work. One call examines at most one funded window
span even when the caller supplies an unlimited quota. Progress reports copied
bytes, freed fragments and examined fragments without logging payloads.

A partially consumed fragment stays in its original slot. Its cumulative ACK is
delayed until fully consumed. This is conservative backpressure: the native
implementation instead copies unread tails into a separate fragment cache and
advances the base earlier. Complete original bytes remain retained and may still
be selectively acknowledged. Stream readers must drain incrementally, rather
than waiting for a complete application message to fit the receive window.

`acknowledgement(start_offset)` returns the consumed cumulative base and up to
three sorted disjoint received ranges relative to that base. `next_offset` allows
the host to inspect later pages without starving ranges beyond the first three;
zero marks the end. Reset pagination to zero when the base changes. Out-of-span
start offsets restart at zero. Generating metadata neither sends a packet nor
changes receive state or host pacing policy.

Tests cover independent concatenated streams under reverse arrival and repeated
slot reuse, binary maximum fragments, gaps, partial reads, conflicting duplicates,
cancellation, dirty storage, explicit work quotas, selective pagination and signed
sequence/range boundaries. A real UDP test validates exact body framing before
admitting reordered bytes, rejects a malformed body and ignores a conflicting
duplicate. Its outgoing side is empty, with ACK-zero/no-SACK admission checked
explicitly. A retransmission runtime, application dispatch and native-peer
interoperability remain separate acceptance gates. [Send windows](reliable-send.md)
now provide whole outgoing ACK preflight for combined receive/send transactions.

[Application framing](reliable-framing.md) now consumes incrementally drained
stream bytes with fixed caller storage, complete-frame borrows and no execution.
