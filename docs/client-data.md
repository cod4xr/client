# Owned client identity records

`client_data::ClientData` decodes the protocol-21 gamestate `svc_configclient`
fields after the opcode: one client-index byte, a NUL-terminated name and a
NUL-terminated clan tag. `ClientDataUpdate` decodes the ordinary service form,
which prefixes the same identity with a 32-bit config-data sequence. Both formats
come from the locked client `src/cl_main.c`, `CL_ParseGamestateX` and
`CL_ParseConfigClientData`; opcode 11 is declared in locked `src/qcommon.h`.

The baseline requires identifiers below 64, names of at most 32 bytes and tags of
at most 12 bytes excluding NUL. These are wire namespaces and field capacities,
not engine-wide player or string limits. Small owned arrays retain checked text
without allocations or dependence on an input frame. No fixed-size player table
is created. Other baseline profiles reject before any cursor changes.

Both parsers reuse `BodyReader` and the pinned shared string codec for NUL framing,
percent normalization and raw-byte preservation. They make no UTF-8 assumption.
Native display code truncates overlong text; this decoder deliberately rejects it
without publishing a truncated identity. This is strict malformed-input policy,
not a promise to accept arbitrary native truncation behavior.

All fields validate through a cloned reader into private local storage. Success
returns the complete owned record and advances the original byte cursor, retaining
any cached bits. Failure preserves both original cursors. Debug and errors expose
only identifiers, sequence bits, lengths and validation categories, never text.

Updates retain all native 32-bit sequence bits without choosing a wrapping,
ordering or acknowledgement policy. The separate [identity state](client-identity-state.md)
now provides exact-next batch admission and joint gamestate/frame commit. The host
must validate the whole enclosing ordinary message before applying update batches.
Neither parsing function mutates engine state,
dispatches commands or proves a player identity.

Six focused tests cover independent vectors, maximum/empty/raw fields, normalization,
owned data after input release, records beyond 128 KiB, every truncation, strict
length and identifier failures, unsupported profiles, all sequence high-bit cases,
cached-bit preservation and private debug output. The newer identity-state tests
cover sequence/table transactions separately. Ordinary service dispatch, remaining
session/engine state and live gameplay interoperability remain open.
