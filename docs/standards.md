# Engineering standards

Use Google's engineering principles of clear naming, small cohesive units,
explicit contracts, reviewable changes and evidence for behavior. Rust ownership,
error and API conventions take precedence over translating C++ formatting rules
into Rust. This is a project policy, not a claim of Google certification.

Workspace members inherit package metadata and strict all/pedantic Clippy lints.
Rustfmt owns formatting. Unsafe code, missing public documentation, ignored
must-use results, production unwrap/expect/panic, debug macros and unfinished
implementation macros are denied. Handwritten source aims for 250 lines and has
a hard ceiling of 400. Dependency rules describe implemented responsibilities;
new crates require a real ownership/dependency boundary.

Keep protocol parsing deterministic and independent of OS services. Preserve
remote byte encodings, preflight output capacities and return fixed errors without
including untrusted payloads. Product output can display escaped metadata, while
logs retain event names, counters and operator-selected addresses. A random query
challenge is correlation only; never label it authentication or server identity.

Tests should establish observable contracts: independent byte vectors, every
capacity prefix, malformed inputs, deadline/resource exhaustion and real process
I/O. Avoid tests that merely restate implementation details. Cross-peer evidence
must identify the tested program and distinguish development probes from ordinary
game interoperability. Hosted CI and locally executed checks are different records.

Use one existing local Cargo target instead of creating a new cache for each task
or executable lock. Stop only the verified development executable for this client.
Do not terminate unrelated editors, server work or Cargo processes. The local
target configuration and private runtime settings stay outside Git.
