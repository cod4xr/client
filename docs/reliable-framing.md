# Reliable application framing

The locked public client's `CL_SendReliableClientCommand` and
`CL_ReceiveReliableMessages` use a signed little-endian 32-bit body length. The
length excludes its own four-byte prefix. The body begins with a separate
application opcode; framing alone grants no opcode, download, authentication,
filesystem or execution authority.

`cod4xr_reliable::framing` owns this previously absent format.
`ReliableFrame::parse` requires one exact whole frame; missing or trailing bytes
fail. `encode` checks every bound before changing output. Bodies are borrowed,
including zero-length bodies that application validation must assess separately.
Debug output contains lengths only.

`FrameAssembler` incrementally copies admitted stream bytes into caller-provided
body storage. Its storage length is the explicit resource budget. No remote length
causes allocation. The native client's whole-message scratch limit is 1 MiB,
including the four-byte prefix; exported constants distinguish that historical
budget from the signed wire-length domain. Larger host storage does not imply a
peer supports larger messages or activate an extended profile.

One to three partial prefix bytes persist across pushes. Incomplete body bytes
remain private. A push admits at most one complete frame and reports exactly how
much input it consumed; the caller must preserve the remaining suffix and enforce
its own frame/work quota. A retained complete frame consumes no more bytes until
`consume_ready` releases it. Borrowing the completed frame prevents storage reuse
while inspecting its bytes.

Negative, unrepresentable or over-budget lengths permanently fail the assembler.
It exposes no partial frame and never guesses a resynchronization point. Later
pushes fail, and consuming a frame cannot clear the failure. A new connection and
fresh assembler are required.

Receive-window readers should drain incrementally into this assembler, respecting
both fragment and message quotas. Waiting for a whole application message inside
the transport window can deadlock on a message larger than that window. Tests
cover every three-chunk partition of an independent frame, concatenated and empty
frames, bounded consumption, dirty-storage reuse, terminal length errors, output
atomicity and a complete native-size message through three reordered slots.

This implements framing, not typed application dispatch or a complete auxiliary
transport session. [Send windows](reliable-send.md) now provide outgoing ACK
admission, and [auxiliary UDP sends](reliable-udp.md) now record complete OS I/O.
[Scheduling](reliable-schedule.md) now paces original retries and ACKs. Complete
session orchestration, gamestate and ordinary-peer interoperability remain separate gates.
