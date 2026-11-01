# Service-control validation

The decoder returns untrusted service bytes and a checked client-command
acknowledgement. `LegacyServicePlan` validates complete ordinary control streams,
using pinned byte opcodes for NOP (0), server command (4), and EOF (7). Gamestate,
snapshots, downloads, config-client data and other service records require their
own decoders; this boundary rejects unsupported records without skipping an
unknown variable-length body or committing acknowledgement.

A plan retains normalized command bytes in caller storage and bounded owned
descriptor metadata. It first validates all records and budgets through logical EOF,
using private scratch for shared legacy-string reading. Only after this preflight
does it populate final command output. Failure preserves final output and all
session/history state. Native Huffman padding after EOF is ignored as framing
padding, without executing or scanning it as another service stream.

The baseline has at most 128 server-command records and a host-selected total
record budget including NOP/EOF. Commands carry positive signed 32-bit sequence
numbers and a native big-string limit of 8191 bytes. Shared string reading maps
percent to period while retaining other raw bytes. The later retained obfuscation
key uses at most the first 1023 normalized bytes through `ServerCommand::key()`;
the full command view retains all accepted big-string bytes.

Successful plans identify the original profile, message sequence, checked client
acknowledgement, logical consumed length and normalized command records. They do
not execute commands, load content or mark a gameplay session active. The host
must apply them to the same connection after preflighting its own server-command/
gamestate resources. Duplicate command handling and session-level commit ownership
remain explicit responsibilities rather than side effects during byte parsing.

Nine core tests cover literal record vectors, empty and maximum strings, byte
normalization, every truncation/output limit, unsupported/missing EOF, positive
sequence bounds, record/command quotas, malformed suffixes without partial output,
padding isolation and privacy checks. A full UDP send/decode/validate test proves
that a malformed stream does not release pending client commands or publish partial
normalized command output. Raw transport sequence 7 is distinguished from the host's
still-zero validated message sequence. Complete EOF on the following sequence 8
allows explicit host acknowledgement commit. No command execution or ordinary
gameplay-server session was tested.
