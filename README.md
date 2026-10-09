# Bevy GameSim

A Rust-owned, deterministic game-simulation architecture built as composable Bevy plugins.

> **Status:** design and Stage 1 implementation, with a versioned map, file-driven input/output fixtures, divergence reporting, and a Bevy presentation client. The repository is proving the deterministic local loop before expanding to rollback, multiplayer, or external engine adapters.

Bevy GameSim adds explicit simulation ticks, typed pipe contracts, authoritative command application, replay, and state-hash diagnostics. Rollback, multiplayer correction, and protected interpreters remain staged extensions without replacing Bevy's ECS or renderer.

## Core idea

```text
input / network
      |
      v
TickInputFrame
      |
      v
read-only WorldView -> interpreted part -> typed IntentStream
                                                  |
                                                  v
                                      validation + authority
                                                  |
                                                  v
                                          CommandStream
                                                  |
                                                  v
                                           ECS application
                                                  |
                                                  v
                              state hash / snapshot / events
```

The authoritative simulation is the single source of truth. Local play, replay, rollback, and multiplayer use the same simulation path. Rendering, audio, UI, and engine-specific presentation consume published state and events; they do not independently decide gameplay outcomes.

## Project layout

```text
bevy_gamesim/
├── README.md             project overview
├── vision.md             overall vision and design decisions
├── docs/                 architecture, determinism, replay, authority, and roadmap
├── requi/                project requirements
├── crates/               Rust simulation and protocol implementation
└── demo/
    └── bomberman/       Bomberman map, fixtures, protocol, and Bevy client
```

The demo is an original Bomberman-inspired grid arena. It is a technical demonstration, not a use of commercial Bomberman assets, names, sounds, or branding.

## Documentation map

- [Vision](vision.md) — purpose, scope, guiding decisions, and success criteria.
- [Architecture](docs/architecture.md) — pipe stages, contracts, invariants, and Bevy integration.
- [Determinism](docs/determinism.md) — tick ownership, ordering, numeric policy, and state hashes.
- [Replay and rollback](docs/replay-rollback.md) — recordings, snapshots, resimulation, and presentation effects.
- [Multiplayer](docs/multiplayer.md) — authority, prediction, correction, and reconciliation.
- [Interpreters](docs/interpreters.md) — Rust/WASM execution, capabilities, budgets, and fault containment.
- [Adapter vision](docs/adapter-vision.md) — integration direction for Bevy, Godot, Unity, and Unreal.
- [Roadmap](docs/roadmap.md) — staged delivery and acceptance criteria.
- [Requirements](requi/README.md) — project rules for reuse and explicit contracts.
- [Bomberman demo](demo/bomberman/README.md) — assets, protocol, replays, tools, and engine clients.

## Planned plugin boundaries

```text
bevy_pipe_core       schemas, tick identity, buffers, ordering, faults
bevy_deterministic   fixed ticks, deterministic schedules, random streams, hashes
bevy_replay          recording, playback, seeking, divergence reports
bevy_rollback        snapshots, restore, resimulation, confirmation tracking
bevy_authority       validation, ownership, permissions, command application
bevy_interpreter     Rust and WASM interpreter adapters
bevy_pipe_net        transport-independent replication and correction
bevy_pipe_debug      tick inspection, traces, hashes, and budget diagnostics
```

These are design boundaries implemented incrementally. The current repository proves the deterministic local loop first; later stages must not introduce a second gameplay authority.

## Non-goals

Bevy GameSim is not intended to replace:

- Bevy's ECS, renderer, scene system, or editor direction;
- ordinary direct ECS APIs for non-authoritative tools;
- rendering, audio, or UI systems; or
- the need for WASM or process isolation when logic is untrusted.

## Design invariant

> Interpreted parts read a bounded view of the world and emit typed intents. Only the authoritative simulation applies validated commands.

Every authoritative field has one owner, every pipe has explicit versioned input and output contracts, and every deterministic result must be reproducible from its initial state and input stream.
