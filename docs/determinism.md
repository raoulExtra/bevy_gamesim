# Deterministic simulation

## Objective

The same initial state and the same tick-indexed input stream MUST produce the same authoritative state, commands, events, and state hashes. This property is required for replay, rollback, prediction, and server authority.

## Tick model

The simulation advances through explicit integer ticks:

```text
Tick N input
    -> read state at N
    -> run systems and interpreters
    -> validate and apply commands
    -> produce state N+1
    -> hash state N+1
```

Each tick owns:

- one normalized input frame;
- a fixed system and pipe order;
- deterministic random streams;
- a bounded event and command set;
- the resulting authoritative state; and
- a state hash.

Wall-clock time, rendering frames, audio callbacks, thread scheduling, and packet arrival order MUST NOT change authoritative results.

## Deterministic core

The deterministic core should provide:

- a fixed simulation tick rate;
- stable system and pipe ordering;
- deterministic entity and command ordering;
- fixed-point or explicitly controlled numeric types where required;
- seeded random streams;
- deterministic timers and event queues;
- stable serialization of authoritative state;
- state hashing; and
- diagnostics for nondeterministic access.

Bevy systems that only affect presentation MAY remain frame-driven. Simulation systems MUST be separated from rendering, audio, UI, asset loading, and other nondeterministic services.

## Recommended schedule

```text
CollectExternalInput
NormalizeInput
BuildWorldView
RunDeterministicSystems
RunInterpretedPipes
ValidateIntents
ApplyAuthoritativeCommands
ResolvePhysics
EmitSimulationEvents
SnapshotAndHash
PublishPresentationState
```

The schedule order is part of the simulation protocol. Changes to it require a protocol version change or a migration strategy.

## Nondeterminism boundary

Nondeterministic services MUST remain outside the deterministic core:

- wall-clock time;
- OS randomness;
- filesystem access;
- network access;
- GPU results;
- uncontrolled threads; and
- frame-rate-dependent presentation code.

If a nondeterministic value affects gameplay, the authority MUST inject it as an explicit input or recorded command.

## Numeric policy

Values affecting authoritative state MUST use a documented numeric policy. Fixed-point or integer representations are preferred for cross-platform authority. Floating-point use is permitted only when the supported platform set, compiler settings, operation set, and reproducibility limits are explicit.

The policy must cover:

- position and velocity;
- collision and physics calculations;
- timers and interpolation;
- random generation;
- serialization; and
- state hashing.

## Ordering

The implementation MUST define stable ordering for:

- entities participating in authoritative queries;
- system execution;
- pipe execution;
- input frames;
- commands emitted in one tick;
- event delivery; and
- snapshot/hash traversal.

Ordering MUST NOT depend on hash-map iteration, packet arrival, thread completion, or allocator behavior.

## Stage 1 policy

`bevy_pipe_core` makes the following decisions:

- `PROTOCOL_VERSION = 1` and ruleset ID `pipe_arena_stage1`;
- player IDs, bomb IDs, cells, ticks, fuses, and counts use fixed-width integers;
- input players are sorted by ascending player ID;
- commands are applied in intent order, followed by generated fuse and blast commands in deterministic traversal order;
- state vectors are traversed by stable actor, bomb, and cell ordering;
- no floating-point values or random source are used;
- input, intent, command, event, and rejection buffers are bounded; and
- the state hash is FNV-1a 64 over canonical little-endian state bytes.

The Stage 1 wire layout and hash inclusion list are specified in `demo/pipe_arena/protocol/README.md`.

## State hashing

A state hash is a diagnostic and authority tool. It should cover all gameplay state that affects future simulation and exclude presentation-only state.

The hash input must have:

- a stable schema version;
- canonical field ordering;
- explicit treatment of absent/default values;
- fixed-width serialization; and
- a documented inclusion/exclusion list.

Hash checkpoints should be emitted periodically and at replay or network correction boundaries.

## Determinism diagnostics

The system should report:

- the first divergent tick;
- the previous matching hash;
- input frame at divergence;
- pipe outputs and accepted/rejected commands;
- event sequence;
- relevant component diffs; and
- protocol and ruleset versions.

A divergence report should be reproducible as a small artifact without requiring a full session capture.
