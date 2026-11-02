# Unified reliable endpoint

`cod4xr_reliable::endpoint::ReliableEndpoint` owns outgoing/receive windows,
application framing and scheduling over exclusive caller-provided buffers. It
allocates nothing and owns no socket, OS clock, gameplay decoder or command
execution. Construct fresh state for each host connection after peer/profile/qport
admission. The network layer still checks those connection properties.

Construction validates every byte/metadata geometry, body budget and initial
timer before resetting either metadata array. Failed construction preserves both
arrays and dirty bytes. The explicit send span and timing policy remain host
decisions. Receive capacity and complete-message capacity come from the funded
buffers, never remote length fields or sender-window hints.

`enqueue` atomically queues a complete body with its four-byte native length
prefix. The queue copies the prefix and body directly into its fragment slots,
without allocating a full-message temporary. Capacity and sequence checks precede
every write, so a failed enqueue cannot leave a partial prefix in the stream.
Exact fragment multiples still have no auxiliary empty terminator.

`ingest` takes a completely parsed reliable body and explicit host receipt time.
It prepares clock/force-ACK effects, incoming data admission and the entire outgoing
ACK update before committing any of them. Every commit is then infallible under
its exclusive borrow. An invalid future fragment, unsent ACK, late invalid range
or regressing clock changes none of those owners. In particular, a rejected
force-ACK request cannot move the clock or accelerate ACK scheduling.

`poll` returns the existing wait point or exclusive scheduled-datagram lease over
this endpoint's actual windows. `ClientSender::send_scheduled_reliable` can consume
that lease on the acknowledged socket. Failed I/O cancels both data admission and
pacing. Core state remains deterministic and independently testable.

`drain` moves received stream bytes toward at most one complete application frame.
Every read is capped by scratch space, remaining byte/inspection budgets and the
exact next prefix/body need. It never consumes bytes belonging to the next frame.
Ready frames stay available until explicit host inspection and `consume_frame`.
Partial reads retain their fragment; empty fragments charge inspection quota.
Zero scratch or quota performs no work. A returned usage record includes work
already consumed even when a bad length permanently fails the assembler.

Transport consumption is flow control, not acceptance of a game message. Draining
must proceed incrementally for bodies larger than the receive window. The complete
body remains opaque and must pass typed application validation before use. An
invalid frame prefix poisons the endpoint: subsequent ingest/enqueue/poll operations
fail, and a host owning the socket must terminate that connection. No stream
resynchronization or reset clears the failure.

State/debug reports contain bounded counts, sequence positions and readiness only.
They exclude payloads, routing identifiers and private connection strings.

Tests cover constructor atomicity, complete framed enqueue boundaries, paired
receive/ACK/timer cancellation, terminal framing failures and retained failure-work
counts, gaps/empty-fragment quotas, partial headers and concatenated frames. Two
endpoints exchange a 5,000-byte frame and a reverse reply with loss, duplicates,
retry and backpressure from three receive slots. A real UDP fixture rejects a
combined invalid ACK/data/force request, then validates the next complete packet,
delivers its frame and emits the correct consumed ACK on the original socket.

This supplies endpoint orchestration. [Connection owners](reliable-connection.md)
now bind the original socket, internal clock and endpoint to bounded I/O operations.
An ongoing game loop, typed gamestate/application dispatch and ordinary-peer gameplay
evidence remain open. The CLI still exposes discovery/challenge probes.
