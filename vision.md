# Bevy GameSim: Pipeable Deterministic Simulation Vision

## Purpose

Enrich Bevy with a Rust-owned execution model for games that require:

- deterministic simulation;
- replay and rollback;
- authoritative multiplayer; and
- composable, pipeable gameplay and interpretation parts.

The system keeps Bevy's ECS and rendering strengths while adding a strict boundary between **observing simulation state** and **mutating simulation state**.

> An interpreted part reads a bounded view of the world and emits typed intents. Only the authoritative simulation applies validated commands.

## Document map

- [Architecture](docs/architecture.md) — execution model, pipe contracts, data schemas, and Bevy integration.
- [Deterministic simulation](docs/determinism.md) — tick ownership, ordering, numeric policy, state hashes, and nondeterminism boundaries.
- [Replay and rollback](docs/replay-rollback.md) — recordings, snapshots, resimulation, and presentation side effects.
- [Multiplayer authority](docs/multiplayer.md) — server authority, prediction, correction, and transport-independent protocols.
- [Protected interpreters](docs/interpreters.md) — Rust, WASM, capabilities, budgets, and fault containment.
- [Roadmap](docs/roadmap.md) — staged delivery and acceptance criteria.
- [Adapter vision](docs/adapter-vision.md) — migration path for Unity, Unreal, and Godot games.

## Scope

This vision covers the deterministic gameplay core and its boundaries to input, networking, replay, rendering, audio, and tools. It does not require replacing Bevy's renderer, ECS, scene system, or editor direction.

The design is suitable for:

- local deterministic games;
- client/server multiplayer;
- rollback netcode;
- authoritative dedicated servers;
- AI and gameplay interpreters;
- replay inspection and debugging; and
- user-authored logic with restricted capabilities.

## Guiding decisions

1. The authoritative simulation advances through explicit integer ticks.
2. Interpreted parts read a capability-scoped view and emit typed intents.
3. Commands are the only path to authoritative mutation.
4. Replay, rollback, and multiplayer use the same simulation pipe; none gets a second gameplay implementation.
5. Presentation systems remain outside the authoritative state unless explicitly declared otherwise.
6. Safe Rust provides memory safety, not a security sandbox. Untrusted logic uses WASM or process isolation.
7. The system is delivered as composable Bevy plugins, not as a replacement for Bevy's ECS or renderer.

## Non-goals

This vision does not require:

- making every Bevy system deterministic;
- replacing ordinary direct ECS APIs for non-authoritative tools;
- forcing rendering and UI through the rollback state;
- guaranteeing identical floating-point results on every CPU without a numeric policy;
- treating safe Rust as a security sandbox; or
- introducing serialization into every internal system.

The strict pipe applies to authoritative simulation and interpreted parts. Presentation and editor code may remain more flexible.

## Success criteria

The system is successful when a game can:

1. run the same authoritative simulation locally, in replay, and on a server;
2. record inputs and reproduce state hashes;
3. roll back and resimulate without duplicating irreversible effects;
4. reject invalid client or interpreter commands;
5. add gameplay interpreters without granting unrestricted world access;
6. replace an in-process Rust interpreter with WASM or a separate process without changing gameplay contracts; and
7. inspect the exact input, pipe output, command, event, and hash sequence that caused a divergence.
