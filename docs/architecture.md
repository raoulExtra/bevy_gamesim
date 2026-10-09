# Architecture

## Architectural shape

```text
external input / network messages / timers
                    |
                    v
             InputNormalizer
                    |
                    v
              TickInputFrame
                    |
                    v
        +---------------------------+
        | deterministic simulation  |
        |                           |
        |  read-only WorldView      |
        |          |                |
        |          v                |
        |   interpreted parts       |
        |          |                |
        |          v                |
        |   typed IntentStream      |
        |          |                |
        |          v                |
        |   validation + authority  |
        |          |                |
        |          v                |
        |   CommandStream           |
        |          |                |
        |          v                |
        |   apply to ECS world      |
        +---------------------------+
                    |
                    v
       StateHash / Snapshot / Events
          |          |          |
          v          v          v
       replay      rollback    clients
```

The same typed stream can be recorded, replayed, transmitted, inspected, or fed into another pipe stage.

## Core invariants

### 1. Tick ownership

The simulation advances only through explicit integer ticks. A tick has a stable input frame, execution order, resulting state hash, and emitted event set.

```text
Tick N input
    -> read state at N
    -> run systems and interpreters
    -> validate and apply commands
    -> produce state N+1
    -> hash state N+1
```

Wall-clock time, rendering frames, audio callbacks, thread scheduling, and packet arrival order MUST NOT change authoritative results.

### 2. Read before write

Interpreted parts MUST NOT receive unrestricted mutable access to `World`, `Commands`, assets, files, sockets, or operating-system APIs.

They receive:

- a read-only, tick-stamped `WorldView`;
- the current `TickInputFrame`;
- deterministic random streams scoped to the entity or system;
- previously emitted simulation events; and
- an explicit capability set.

They produce typed intents into a bounded output buffer.

### 3. Commands are the only mutation path

All state mutation crosses a command boundary. Commands MUST be:

- typed;
- versioned;
- attributable to a source and tick;
- validated by the authority that applies them;
- deterministic to serialize; and
- bounded in count and payload size.

A script cannot bypass validation by retaining an entity reference or calling a Bevy system directly.

### 4. Pipe stages have explicit contracts

A pipe is a deterministic stage with explicit input and output types:

```rust
pub trait Pipe {
    type Input;
    type Output;
    type Fault;

    fn run(
        &mut self,
        input: &Self::Input,
        output: &mut BoundedBuffer<Self::Output>,
    ) -> Result<(), Self::Fault>;
}
```

The first production pipe is the gameplay boundary:

```text
TickInputFrame + WorldView + Events
        -> InterpretedPart
        -> IntentStream
        -> Authority
        -> CommandStream
        -> ECS application
```

A pipe stage MUST declare:

- input schema and schema version;
- output schema and schema version;
- read capabilities;
- write capabilities;
- instruction, time, and output budgets;
- deterministic ordering rules; and
- fault behavior.

Pipe stages MUST NOT communicate through hidden global state.

## Stage 1 data contracts

The first concrete implementation is in `crates/bevy_pipe_core`. It uses fixed-width integer fields and a canonical little-endian encoding for tick input and state hashing. The full protocol summary is in `demo/bomberman/protocol/README.md`.

```rust
pub struct TickInputFrame {
    // Private fields. Construct with `TickInputFrame::new`, or validate
    // decoded data at the simulation boundary.
}

impl TickInputFrame {
    pub fn tick(&self) -> u64;
    pub fn players(&self) -> &[PlayerInput];
}

pub struct WorldView<'a> {
    pub tick: u64,
    // Exposes only approved, stable queries.
    // It does not expose &mut World or arbitrary Entity access.
    _private: std::marker::PhantomData<&'a ()>,
}

pub enum Intent {
    Move { actor: u8, direction: Direction },
    PlaceBomb { actor: u8 },
}

pub enum AuthoritativeCommand {
    Move { actor: u8, from: Cell, to: Cell },
    PlaceBomb { actor: u8, bomb: u32, cell: Cell, fuse_ticks: u16 },
    DetonateBomb { bomb: u32, cell: Cell },
    DestroyWall { cell: Cell },
    EliminatePlayer { actor: u8 },
}
```

This API is illustrative at the architectural level; `bevy_pipe_core` is the executable Stage 1 contract. Protocol changes require updated consumers, fixtures, documentation, and a version or migration decision.

## Authoritative state and rollback ownership

Authoritative state is held by the simulation core. Bevy presentation entities, engine objects, audio handles, particles, and UI state are not part of the authoritative state hash.

Pipe stages participating in authoritative simulation MUST be either stateless or provide deterministic snapshot/restore state. Mutable interpreter state MUST be included in rollback snapshots, reset at the restored tick, and serialized when it can affect future output. A pipe MUST NOT retain engine pointers, wall-clock values, packet order, or process-global randomness.

The authoritative mutation boundary is the simulation schedule. Presentation and adapter systems receive published state and events but do not receive mutable access to authoritative state.

## Bevy integration direction

The design should be delivered as composable Bevy plugins rather than a replacement engine:

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

Rendering, audio, UI, and editor systems consume published presentation state and confirmed events. They do not become part of the authoritative state hash unless explicitly declared simulation state.
