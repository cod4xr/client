# Owned network phase boundary

The implemented network boundary moves reusable blocking socket ownership out of the
CLI application. `client-core` remains deterministic state and codecs. A client
network crate owns UDP sockets, challenge entropy, buffers and monotonic I/O timers;
the application retains argument parsing, report output and logging setup.

This boundary is justified by the actual challenge-to-connect handoff: both phases
must use one selected peer and the same bound socket. Returning only a challenge
cookie from a one-shot probe loses that resource lifetime. The connection path
must consume an owned challenged socket into an owned pending-connect socket,
then hand an acknowledged socket and original protocol/qport metadata to channel
initialization. Standalone probes may intentionally close after reporting.

The API owns one complete UDP receive buffer and prepaid terminal-reply
storage. It does not allocate from untrusted length fields or expose private raw
userinfo/cookies in Debug/errors. Core attempt counters and absolute phase deadlines
remain authoritative. Notices consume work without restarting timers or activating
server markup. Terminal payload bytes remain data for a future display/asset layer.

Public operations are blocking and belong on an I/O worker, not a future rendering
thread. The accepted transport is still pre-channel state, not a playable session.
This step creates no renderer/assets/plugin abstractions, auth services, custom
wire identifiers or new local Cargo target directory. All edits remain client-only.

Eight independent real UDP tests prove one source socket across
both phases, identical retries, correct cookie/nonce fields, refusal/notice handling,
deadline/work exhaustion and ownership-preserving handoff. They use independent
source-shaped packets, not the production encoders as the simulated peer.

`challenge(peer, budget)` returns aggregate work even on failure. Its successful
`ChallengedSocket` can be consumed through `into_pending(settings, budget)`, then
`PendingConnect::run()` returns an acknowledgement owning the original socket or
an owned refusal closing it. Pending deadlines start on construction, including
any caller delay before `run`. Dropping any owning phase closes its socket.

Acknowledged transport exposes an explicit `into_parts` handoff for channel
ownership. The transferred socket is still blocking with finite timeouts; the new
owner must set its own I/O policy. Receive scratch storage is released at this
handoff. The original requested protocol/qport and inert mod-label bytes remain
owned context, together with the original private challenge needed for message
obfuscation. No state here claims a negotiated protocol or active gamestate.

`into_channel(storage)` is the integrated ordinary-channel handoff. It retains the
receive scratch allocation and borrows the caller's bounded reassembly storage.
The channel reuses shared sequencing/fragmentation and can split into independently
borrowed send/receive halves for pacing; see [ordinary channel](ordinary-channel.md).

The standalone CLI challenge probe deliberately copies its receipt and closes its
socket after reporting. Discovery uses the same datagram primitives but retains
its separate one-query correlation and report behavior. No connect CLI is exposed
before channel ownership and a clean disconnect lifecycle are implemented.
