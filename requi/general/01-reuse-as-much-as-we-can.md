# Requirement: Reuse as much as we can

We MUST reuse existing, proven work wherever it fits the Bevy GameSim goals.

## Rules

- Inspect the repository, Bevy APIs, Rust crates, existing protocols, and existing tools before creating new mechanisms.
- Prefer composing, extending, or adapting an existing implementation over introducing a parallel abstraction.
- Keep one authoritative implementation for each gameplay rule and simulation concern.
- Reuse existing conventions for naming, serialization, errors, testing, fixtures, and documentation.
- Add new infrastructure only when the existing option cannot satisfy the required contract, determinism, safety, or performance.
- When new infrastructure is necessary, record the gap and explain why reuse was rejected.
- Remove obsolete duplicate paths after a replacement is adopted.

## Acceptance

A change satisfies this requirement when its implementation identifies the existing work it reuses, or documents the concrete limitation that requires new work. No second gameplay authority or competing protocol is introduced.
