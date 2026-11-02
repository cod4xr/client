# Service-control validation

The decoder returns untrusted service bytes and a checked client-command
acknowledgement. `LegacyServicePlan` validates complete ordinary control streams,
using pinned byte opcodes for NOP (0), server command (4), and EOF (7). Gamestate,
snapshots, downloads, config-client data and other service records require their
own decoders; this boundary rejects unsupported records without skipping an
unknown variable-length body or committing acknowledgement.

The opt-in protocol-21 `parse_with_client_data` path also accepts sequenced client
data (11) mixed with those records. It shares the same scanner and publishes final
command bytes and owned identity cells only after the complete stream validates.
`ClientServiceBudget` bounds total records and identity updates independently;
`ClientServiceStorage` separates private update scratch from final cells and text.
Neither private scratch buffer is promised unchanged on failure. Both final outputs,
including unused tails, are unchanged on failure. The original `parse` API continues
to reject opcode 11. See [mixed controls](mixed-service-controls.md) for application
ordering and UDP evidence.

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

The 128-record command bound does not limit the number of identity updates in a
mixed plan. Ordinary payload decompression still uses the existing 128 KiB profile;
this API does not extend wire sizes or imply a larger native player namespace.

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

[Server-command history](server-command-history.md) now supplies the retention and
monotonic duplicate handling for these plans, with prepared updates that can be
cancelled before state changes. Typed execution and gamestate remain separate.
