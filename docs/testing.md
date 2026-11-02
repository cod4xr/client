# Test organization

The deterministic core has one explicit integration target, `core`. Its component
modules and case files stay separate; the harness only declares modules. Shared
fixtures are loaded once at the harness root and reused by components. Unit tests
and compile-fail documentation tests remain separate Cargo targets.

This replaces 31 integration executables with one, reducing repeated linking and
per-executable overhead as protocol code grows. It uses the same existing Cargo
target directory and release profile. It does not remove tests or relax validation.
The before/after inventory comparison preserved all 289 integration cases and nine
core unit cases at the consolidation checkpoint; [evidence](core-test-inventory.json)
records the normalized case-identity hash. Future checkpoints add their own cases.

Run a component or individual case with its namespace filter:

```powershell
cargo test -p cod4xr-client-core --test core player_arrays:: --locked -j 2
cargo test -p cod4xr-client-core --test core snapshot_entities:: --locked -j 2
cargo test -p cod4xr-client-core --lib --locked -j 2
cargo test --workspace --locked -j 2
cargo test --workspace --release --locked -j 2
```

Create each new integration component as a top-level file in
`crates/client-core/tests` and register its module in `tests/core.rs`. Keep its
fixtures/cases in focused sibling folders. Shared fixture directories ending in
`_support` are registered once at the harness root. Component-local fixture code
can stay under its owning component without becoming a separate integration binary.

`tools/check_repository.py` verifies that the single Cargo test target is explicit,
all component files and shared fixtures are registered exactly once, and the harness
contains only unconditional module declarations/comments. Missing, duplicate,
unknown and conditionally hidden harness modules were checked to fail policy.
This guard prevents disabling Cargo auto-discovery from silently omitting a new test
file. Keep all test logic and data outside the small registry file.

Workspace quality gates remain formatting, all-target Clippy with warnings denied,
debug/release tests, rustdoc with warnings denied, repository policy and locked
schema/source reproduction. Reuse the existing local target; never create a target
directory to bypass a lock. Stop only this client's exact development process with
`tools/stop-dev-client.ps1`, leaving unrelated builds alone.
