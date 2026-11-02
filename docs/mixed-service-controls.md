# Mixed ordinary service controls

`LegacyServicePlan::parse_with_client_data` validates protocol-21 ordinary streams
containing NOP (0), server commands (4), sequenced client data (11) and logical EOF
(7). The original `parse` stays strict and rejects client data. Both paths share
one scanner and shared wire readers. Unsupported services reject the whole stream;
the parser never skips an unknown body. Ordinary decompression remains bounded by
the existing 128 KiB profile, and native padding after logical EOF remains inert.

The caller supplies a total-record budget, identity-update budget, private text
and update scratch, and separate final command bytes and identity cells. Command
descriptors retain their existing per-message bound of 128. Identity counts have
independent host budgets and capacities. A synthetic 200-update stream proves that
the command window does not become an identity-update limit. Native wire client
indices and text widths remain unchanged.

Parsing validates the complete stream before publishing either final output. A late
bad field, unsupported opcode, missing EOF or exhausted budget leaves both complete
final buffers unchanged, including unused tails. Private scratch is unspecified on
failure. A successful plan borrows only final output, retaining owned client records
and normalized command bytes independently of the encrypted/decompressed message,
history borrow and private workspace. Debug/error output exposes counts/categories,
not names, tags or command text.

`ServerCommandHistory::prepare` and `ClientIdentityState::prepare_service_plan`
consume the same plan. Identity staging checks the profile and requires a committed
gamestate baseline, then admits only each exact next wrapping sequence. It does not
retain a plan borrow after staging. The host preflights both prepared owners and
sibling session resources before committing the checked client-command ACK. The
remaining owner commits are infallible. Dropping either preparation cancels it;
parsing alone advances no history, identity sequence or acknowledgement.

Twelve core tests cover mixed ordering, source/workspace release, independent
capacities and budgets, all truncations, malformed late identities, EOF padding,
unchanged final tails, strict old/profile APIs, more than 128 updates, cancellation,
late identity-stage failure and whole-plan application. One bidirectional UDP test
first rejects a late unsupported service without releasing a pending client
command. A subsequent valid stream stages command sequence 3 and identity sequences
10/11 while ignoring forward gap 12. After commit, the peer decodes the next actual
client datagram using message ACK 8, server-command ACK 3, config-data ACK 11 and the
new retained command key. Input/private/final buffer clearing checks retained state
ownership. This is loopback evidence, not a native gameplay interoperability claim.

Typed command execution, snapshots, downloads, legacy identity adapters and sibling
session/content application remain separate gates. No raw peer strings are executed,
no remote CVars are applied and no gameplay-ready state is inferred by these APIs.
