# Pipe Arena protocol

Versioned contracts shared by the Rust simulation and all engine adapters.

## Stage 1 contract

The first implementation lives in [`bevy_pipe_core`](../../../crates/bevy_pipe_core/). It defines:

- `TickInputFrame`;
- `PlayerInput` and `Direction`;
- `Intent`;
- validated `AuthoritativeCommand`;
- `PresentationEvent`;
- deterministic state bytes and `StateHash`;
- replay metadata; and
- bounded pipe buffers.

The binary `TickInputFrame` encoding is:

```text
u16 protocol_version, little-endian
u64 tick, little-endian
u8 player_count
repeat player_count:
    u8 player_id
    u8 direction (0 none, 1 up, 2 right, 3 down, 4 left)
    u8 place_bomb (0 false, 1 true)
```

Frames are sorted by player ID, reject duplicate IDs, reject unknown direction values, reject invalid booleans, and reject trailing bytes. `PROTOCOL_VERSION` is currently `1`; incompatible changes require a version change or migration.

State hashes use canonical fixed-width little-endian state bytes and FNV-1a 64. The hash includes protocol version, ruleset ID, tick, arena dimensions, sorted player state, sorted bomb state, sorted destructible-wall state, and the next bomb ID.

## Boundary flow

```text
engine input
    → TickInputFrame
    → input pipe
    → Intent
    → authority validation
    → AuthoritativeCommand
    → state application
    → StateHash + PresentationEvent
    → engine adapter
```

The protocol contains stable numeric IDs and engine-neutral values only. It must not contain engine object pointers, wall-clock time, packet-arrival order, or presentation-only object state.
