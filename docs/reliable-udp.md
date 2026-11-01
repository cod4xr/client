# Auxiliary reliable UDP sends

The acknowledged channel's existing `ClientSender` now sends explicit reliable
fragments and ACK-only packets on the original selected UDP socket. Both operations
reuse the pinned shared `Packet::Reliable` envelope with the original qport. No
socket, routing identity or ordinary-channel sequence is recreated or reserved.
The exclusive sender serializes ordinary and auxiliary I/O; finish or drop an
ordinary `OutboundMessage` before borrowing it for an auxiliary send. The separate
receive half remains usable between sends.

Emission requires the originally selected, reviewed protocol 21 profile. Legacy
protocol 6 and unreviewed profiles fail before encoding, I/O or transmit admission.
The challenge's protocol-18 hint and discovery's protocol-6 advertisement do not
enable auxiliary emission. Body codecs remain available for bounded inspection
without granting feature activation or application-command authority.

`send_reliable_fragment` consumes a prepared outgoing fragment and a receive ACK
snapshot. It encodes the original sequence/bytes with baseline flags zero, then
the shared outer envelope, into fixed stack buffers. One call sends at most one
datagram under the supplied absolute deadline. Only after complete successful OS
I/O does it record transmission. Deadline, codec and socket failures drop the
preparation unmarked; the original queued bytes remain available for an explicitly
scheduled retry. A successful retry preserves the original sequence and reports
whether this is its first successful OS admission. None of these results is a
peer acknowledgement.

`send_reliable_ack` uses the supplied receive cumulative/selective snapshot and
the validated outgoing window's sender hint. Its sequence is the native ACK-only
sentinel, its flags are zero and its payload is empty. It changes neither window.
Receive snapshots must belong to the same current connection and follow whole
incoming packet validation. New connections require fresh window state; these
library calls do not manage a session lifetime or reconnect reused queues.

`SentReliableDatagram` separates complete wire bytes from optional data-fragment
admission metadata. Send usage counts ordinary and auxiliary datagrams/bytes in
the totals, with reliable sent/failed subset counters. Protocol and encoding errors
before I/O charge no datagram attempt. Deadline/socket failures charge failed I/O
without marking a fragment sent. Debug values and errors exclude payloads,
qports, cookies and user strings. [Scheduled sends](reliable-schedule.md) additionally
hold the selected fragment and pacing lease until complete I/O, then commit both.

Four real UDP tests check exact independent data/ACK vectors, original socket and
qport, deadline failure followed by stable retransmission, transmit admission only
after successful I/O, validated peer consumption, selective ACKs for a partial
fragment, host window hints, unsupported original profiles and invalid receive
metadata before I/O. Incoming and outgoing updates are preflighted before either
commit in these fixtures. No ordinary gameplay peer was used. The scheduler now
supplies retry/ACK timing; continuous session orchestration and typed
gamestate/application dispatch remain separate acceptance gates.
