# Reliable connection owner

`AcknowledgedSocket::into_reliable` transfers the original acknowledged socket into
`ReliableConnection`, together with one fresh endpoint and one internal monotonic
clock origin. The original peer, qport, protocol and private cookie remain attached
to that socket. Only the reviewed original protocol 21 profile can construct this
owner; discovery/challenge advertisements do not enable it. Ordinary-only channels
remain available separately for other reviewed baseline work.

The factory checks the original profile before touching endpoint buffers. Channel,
endpoint storage and timer failures preserve supplied endpoint contents. The
acknowledged socket is consumed and closes on construction failure. Successful
construction reuses its prepaid UDP receive buffer and caller-funded endpoint
storage; it creates no new socket, remote-selected allocation or background task.

`send_due` samples actual elapsed host time and sends at most one scheduled
datagram, or returns a wait point capped at the caller's absolute deadline.
Successful OS I/O commits both pacing and original fragment admission. Failed
sends retain both leases' prior state for an explicit retry. There is no internal
burst or retry loop.

`receive_until` inspects at most one selected-peer datagram. Shared envelope/qport
checks precede reliable-body parsing and endpoint admission. Receipt time is sampled
after blocking I/O and syntax parsing, then supplied internally to the endpoint.
Only whole admitted packets update the last accepted reliable timestamp. Invalid
body and receive/ACK effects return bounded rejection events without partial state.
Local clock/socket failures propagate as errors.

Ordinary messages, channel-drop events and other auxiliary families go through a
scoped host callback. Borrowed packet bytes cannot escape the receive buffer.
The wrapper's debug output hides the callback's result even if the host explicitly
copies private bytes. Unknown connectionless/update bodies gain no execution,
filesystem or feature authority.

The ordinary sender is a narrow serialized borrow exposing ordinary begin/control
operations, sequence position and copied usage only. It cannot accept external
auxiliary windows or schedulers. This preserves one socket writer and prevents
mixing another endpoint's state through that borrow. Finish or drop its transmission
before servicing another operation on the owner. The original ordinary decoder
remains available with its checked connection context.

Bounded drains and complete-frame inspection use the endpoint's existing body
budget. Host consumption is explicit and counted. A terminal frame-length error
retains charged work and prevents future receive, scheduled send, enqueue and
ordinary-sender operations before they perform I/O. Repeated inspection does not
increment the failure count again. Dropping the owner releases its socket and
buffer borrows; no failed stream reset or detached endpoint is exposed.

Copied statistics combine network totals, reliable admission/rejection counts,
drained bytes, fragment inspections, explicit frame releases, framing failures,
the last accepted receipt time and current endpoint resource state. Debug reports
exclude payloads, private strings, qports and cookies. The application owns log
subscriptions, stable event publication and any aggregate reporting cadence.

Six real UDP tests cover complete framed exchange, invalid routing/body/ACK
admission, scoped ordinary/connectionless callbacks, original ordinary sending,
receipt time after a delayed response, expired waits without charged work,
terminal failure before further I/O, unsupported profiles and late constructor
failures preserving buffers. Native gameplay sessions and typed gamestate dispatch
remain open. [Complete-body readers](message-body-reader.md) now provide the
shared-codec view adapter needed for bodies larger than 128 KiB. The CLI still
exposes discovery/challenge probes; these operations are library building blocks
for the engine's ongoing I/O loop.
