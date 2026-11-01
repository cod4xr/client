# Legacy client-command history

Ordinary reliable client commands are distinct from CoD4X's separate reliable
transport. The pinned client has a 128-entry command window and 1024-byte command
slots. Its queue canonicalizes byte 0x92 to an ASCII apostrophe and percent to a
period before transmitting or using the acknowledged command as an obfuscation key.
Other bytes remain unchanged. The rewrite rejects overlong or NUL-containing
inputs atomically instead of silently truncating an explicit Rust byte slice.

The deterministic core borrows caller-owned storage. A host chooses 1–128 pending
slots within this baseline profile and supplies one additional slot to retain the
last acknowledged key independently of ring reuse. No allocator, clock, socket,
logging subscriber or command execution is involved. Empty commands retain their
native empty-string semantics. Only initialized byte prefixes are exposed.

Queued, transmitted and acknowledged sequences are separate. A host may mark only
a queued prefix as transmitted, after admitting it to actual outgoing message I/O.
A peer acknowledgement cannot select a command that was never transmitted, move
the acknowledgement backwards or refer to unavailable/reused storage. A full
window rejects enqueue without overwriting an unacknowledged command. Sequence
space ends at the positive signed 32-bit bound; reconnect rather than wrap.

Checking an acknowledgement returns a borrowed private command view without
advancing state. The subsequent message layer must decode and validate the entire
message before committing acknowledgement. Commit revalidates the number, copies
the acknowledged key to its dedicated slot, and releases the covered pending
prefix. Malformed messages therefore cannot free history slots or select arbitrary
ring contents as keys. Private commands/keys never appear in Debug or fixed errors.

This is an explicit baseline resource profile, not an engine-wide scripting/string
or custom-protocol limit. Larger future command windows need a negotiated profile
and a separate storage/sequence contract. Message encoding, compression, obfuscation,
server-command dispatch and a clean disconnect lifecycle remain subsequent steps.
