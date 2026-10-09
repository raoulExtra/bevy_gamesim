# Delivery roadmap

The system should be delivered incrementally as composable Bevy plugins rather than as a replacement engine.

## Proposed plugin boundaries

```text
bevy_pipe_core
bevy_deterministic
bevy_replay
bevy_rollback
bevy_authority
bevy_interpreter
bevy_pipe_net
bevy_pipe_debug
```

Responsibilities:

- `bevy_pipe_core`: schemas, tick identity, bounded buffers, ordering, faults;
- `bevy_deterministic`: fixed tick, deterministic schedules, random streams, state hashes;
- `bevy_replay`: recording, playback, seeking, divergence reports;
- `bevy_rollback`: snapshots, restore, resimulation, confirmation tracking;
- `bevy_authority`: validation, ownership, permissions, authoritative application;
- `bevy_interpreter`: Rust and WASM interpreter adapters;
- `bevy_pipe_net`: transport-independent replication and correction protocol;
- `bevy_pipe_debug`: tick inspection, command traces, hashes, and budget diagnostics.

## Stage 1: deterministic local loop

Implemented in `crates/bevy_pipe_core` and `demo/bomberman`:

- explicit simulation tick;
- deterministic, versioned input frames;
- versioned JSON map loading and validation;
- bounded typed intent and command buffers;
- a read-only `WorldView` and Rust gameplay interpreter;
- authority validation and stable command ordering;
- canonical state bytes and FNV-1a 64 state hashes;
- file-driven expected command/event/hash fixtures with first-divergence reporting; and
- a Bevy presentation client consuming authoritative state and events.

Acceptance: identical initial state and input stream produce identical per-tick hashes in repeated runs, and the checked-in fixture matches commands, events, rejections, and hashes. The executable check is `cargo run -p bevy_pipe_core --example stage1`.

## Stage 2: replay diagnostics

Build on the Stage 1 replay contract with:

- persistent replay file format and metadata;
- seeking and checkpoints;
- command/event trace inspection; and
- deterministic random streams.

Acceptance: a recorded session reproduces the same state hashes without using live input or wall-clock time.

## Stage 3: rollback

Deliver:

- bounded snapshot ring;
- restore and resimulation;
- presentation confirmation rules; and
- late-input correction tests.

Acceptance: correcting an input at tick `K` produces the same final state as a clean run that received the corrected input at tick `K`.

## Stage 4: authoritative multiplayer

Deliver:

- server-owned tick and validation;
- client prediction;
- authoritative corrections;
- reconciliation and rollback; and
- transport-independent protocol types.

Acceptance: a client cannot create authoritative entities or state changes without an accepted command, and divergent clients converge after correction.

## Stage 5: protected interpretation

Deliver:

- capability-scoped `WorldView`;
- execution budgets;
- WASM or process isolation for untrusted parts;
- schema-versioned pipe messages;
- fault containment; and
- deterministic interpreter tests.

Acceptance: an interpreter can produce gameplay behavior through commands but cannot directly mutate the world or access undeclared services.

## Cross-stage quality gates

Every stage must preserve:

- one authoritative simulation implementation;
- explicit protocol and schema versions;
- bounded memory and output behavior;
- deterministic test fixtures;
- first-divergence diagnostics; and
- separation between simulation and presentation.
