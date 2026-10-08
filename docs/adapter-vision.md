# Adapter vision: Unity, Unreal, and Godot

## Purpose

Enable existing Unity, Unreal, and Godot games to adopt the Rust pipe architecture without requiring an immediate rewrite of their renderer, editor, scenes, assets, animation, audio, or UI.

The practical target is not an automatic conversion of an arbitrary game. The target is a staged extraction of the authoritative gameplay layer:

```text
Unity / Unreal / Godot presentation client
        input, camera, animation, rendering, audio
                         |
                         | TickInputFrame
                         v
                 Rust simulation authority
                         |
                         | StateDelta + PresentationEvents
                         v
Unity / Unreal / Godot presentation client
```

Rust owns authoritative gameplay state and decisions. The existing engine remains a presentation and content-production environment until individual systems are migrated.

## Core rule

There MUST be exactly one authority for each authoritative field.

Example ownership:

```text
health, inventory, abilities, score, match state  -> Rust
server tick and accepted commands                  -> Rust
camera, animation playback, particles, audio      -> engine
editor state and visual layout                     -> engine
```

Two systems MUST NOT independently decide the same gameplay result. In particular, an engine's built-in replication or physics result must not silently compete with the Rust authority.

## Migration boundary

### Portable gameplay

The following systems are good candidates for migration:

- player rules;
- abilities;
- damage and health;
- inventory;
- quests;
- AI decisions;
- timers;
- scoring;
- economy;
- match state;
- deterministic movement; and
- command validation.

### Engine-specific presentation

The following systems usually remain in the host engine initially:

- scene graphs;
- rendering materials and shaders;
- cameras;
- animation graphs;
- particles;
- audio;
- UI layout;
- editor metadata; and
- visual effects.

Assets can usually remain in the original engine. Gameplay semantics, hidden state, callbacks, coroutines, object references, and physics assumptions generally require explicit extraction or rewriting.

## Adapter protocol

The first compatibility pipe is:

```text
EngineInputSnapshot
    -> Rust TickInputFrame
    -> Rust StateDelta + PresentationEvents
    -> engine adapter
```

Example protocol types:

```rust
struct TickInputFrame {
    tick: u64,
    players: Vec<PlayerInput>,
}

struct StateDelta {
    tick: u64,
    entities: Vec<EntityUpdate>,
}

enum PresentationEvent {
    PlayAnimation {
        entity: EntityId,
        animation: AnimationId,
    },
    PlaySound {
        sound: SoundId,
        position: FixedVec3,
    },
    SpawnEffect {
        effect: EffectId,
        position: FixedVec3,
    },
}
```

The protocol MUST define:

- tick and protocol versions;
- entity identity and lifecycle;
- input normalization;
- state ownership;
- fixed-width or canonical serialization;
- correction and snapshot behavior;
- event confirmation status; and
- error and rejection messages.

The engine adapter translates `StateDelta` into engine-specific component, actor, or node updates. It translates `PresentationEvent` into animation, audio, UI, and visual effects.

## Migration levels

### Level 1: shadow simulation

The existing game remains authoritative. The adapter sends normalized inputs to Rust and runs the Rust simulation in parallel:

```text
engine input
    ├──> existing gameplay
    └──> Rust shadow simulation
```

Compare state hashes, positions, health, commands, events, and timing. This exposes nondeterminism without changing player-visible behavior.

### Level 2: subsystem authority

Move one bounded gameplay subsystem to Rust, such as health, damage, inventory, abilities, or match scoring.

The engine stops mutating that subsystem directly and applies Rust state deltas instead. All other systems remain unchanged.

### Level 3: full local authority

The engine becomes a local client:

```text
engine input
    -> TickInputFrame
    -> Rust simulation
    -> StateDelta
    -> engine presentation
```

Existing engine scripts stop mutating authoritative gameplay state. They may still control presentation through confirmed or speculative events.

### Level 4: authoritative server

```text
Unity / Unreal / Godot clients
            | input
            v
      Rust authority server
            | state and corrections
            v
Unity / Unreal / Godot clients
```

Clients may predict local input, but the Rust server accepts commands and decides authoritative state.

### Level 5: rollback and replay

Add snapshots, input history, state hashes, correction, replay playback, and confirmation-aware presentation events. The same Rust simulation must run locally, in replay, and on the server.

## Engine adapter responsibilities

Every adapter should provide:

- input capture and normalization;
- tick synchronization;
- protocol serialization and transport;
- entity identity mapping;
- state-delta application;
- presentation-event dispatch;
- snapshot/correction application;
- connection and authority status;
- protocol version checks; and
- divergence diagnostics.

The adapter MUST NOT contain a second implementation of gameplay rules. It is a translation layer, not an alternate authority.

## Engine integration options

### In-process adapter

The Rust simulation runs as a native plugin or extension inside the engine process.

Advantages:

- low latency;
- direct memory access where safe;
- simple local deployment.

Risks:

- no process isolation;
- FFI boundaries can be unsafe;
- engine callbacks may introduce nondeterminism;
- a crash can take down both simulation and presentation.

### Out-of-process adapter

The engine communicates with a Rust simulation process using a typed IPC or network protocol.

Advantages:

- stronger fault containment;
- clear ownership boundary;
- the same authority can serve local and remote clients;
- suitable for untrusted or user-authored interpreted parts.

Costs:

- serialization and transport overhead;
- lifecycle and deployment complexity;
- explicit synchronization required.

Untrusted logic MUST use WASM or process isolation. A Rust native plugin is not automatically a sandbox.

## Engine-specific paths

### Godot

Use a GDExtension, Rust plugin, or external Rust process. Map `InputEvent` and selected Node state into `TickInputFrame`; apply `StateDelta` back to Nodes and dispatch `PresentationEvent` through animation, audio, and effect systems.

Godot scripts and Nodes commonly mutate state directly. Migration therefore begins by assigning ownership per component and replacing direct gameplay writes with adapter-applied state.

### Unity

Use a C# native-plugin boundary, a Rust process, or a network client adapter. Normalize Unity input and convert Rust deltas into component or GameObject updates.

Existing C# gameplay, coroutines, and Unity physics should be treated as migration targets, not assumed to be deterministic. Unity replication must be disabled for fields owned by the Rust authority or explicitly bridged to it.

### Unreal

Use a C++ module/plugin, a Rust process, or a network client adapter. Map input and Actor identity into the protocol; apply authoritative deltas to Actors and components.

Blueprints, gameplay components, Chaos physics, and Unreal replication require explicit ownership decisions. Unreal's replication system must not compete with the Rust authority for the same fields.

## Physics boundary

Physics is usually the hardest system to migrate. Unity PhysX, Unreal Chaos, and Godot physics are not automatically deterministic across platforms, versions, thread counts, or floating-point environments.

Choose one authority strategy:

1. Rust-owned deterministic physics;
2. a dedicated deterministic physics library;
3. server-only authoritative physics with client corrections; or
4. engine physics for presentation, with important gameplay collisions validated in Rust.

The engine and Rust MUST NOT independently decide authoritative collision, damage, or movement results.

## Asset and content migration

The adapter should preserve existing content where possible:

- models and textures remain engine assets;
- animation clips remain engine assets;
- audio remains engine-managed;
- scenes remain editor-authored;
- Rust receives stable asset identifiers rather than engine object pointers.

Authoritative gameplay data should use versioned, engine-neutral identifiers and schemas. Presentation assets can be resolved by each adapter.

## Migration workflow

1. Inventory engine-owned and gameplay-owned state.
2. Define entity identity and authoritative field ownership.
3. Normalize engine input into `TickInputFrame`.
4. Implement Rust shadow simulation for a small subsystem.
5. Add state hashes and command/event traces.
6. Compare Rust and legacy behavior in recorded scenarios.
7. Transfer authority for the subsystem.
8. Add replay and correction before adding multiplayer.
9. Add client prediction and server authority.
10. Replace in-process interpretation with WASM or process isolation where untrusted logic is required.

## Success criteria

An adapter is successful when it can:

1. send normalized engine input into the Rust tick protocol;
2. apply Rust state deltas without duplicating gameplay rules;
3. dispatch presentation events without making them authoritative;
4. run shadow comparisons and identify the first divergence;
5. transfer ownership of a gameplay subsystem without two authorities;
6. support replay and rollback through the same Rust simulation;
7. support an authoritative Rust server; and
8. replace an in-process Rust module with an isolated process or WASM module without changing the game protocol.

The result is not a universal game converter. It is a stable migration path from engine-owned gameplay toward a Rust-owned deterministic simulation with Unity, Unreal, and Godot acting as interchangeable presentation clients.
