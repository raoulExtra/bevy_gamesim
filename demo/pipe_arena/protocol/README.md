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

State hashes use canonical fixed-width little-endian state bytes and FNV-1a 64. The hash includes protocol version, ruleset ID, tick, arena dimensions, sorted player state, sorted bomb state, sorted indestructible-wall state, sorted destructible-wall state, and the next bomb ID.

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

## Data format decisions

The project uses different formats for authored data and authority-facing data:

| Data | Format | Rule |
| --- | --- | --- |
| Arena maps and asset manifests | JSON | Cross-engine, versioned, validated before use |
| Authored input fixtures | JSON | Human-readable source converted to canonical `TickInputFrame` bytes |
| Tick inputs and replay streams | Canonical binary | Fixed-width little-endian fields; no parser-dependent representation |
| Authoritative state hashes | Canonical binary state bytes | Hash the normalized state, never raw JSON |
| Rust-only tooling configuration | TOML | Not part of the simulation protocol |
| Documentation and design notes | Markdown | Human-readable |
| Presentation assets | Native asset formats | PNG, OGG, GLTF, and engine-imported derivatives |

YAML MUST NOT be used for authoritative maps, rules, inputs, replays, or state. Its implicit typing, aliases, parser differences, and multiple equivalent representations make cross-language canonicalization needlessly fragile.

### Authored JSON rules

Authored JSON is input data, not authority. The loader MUST:

1. validate `schema_version` and `ruleset_id`;
2. validate dimensions, coordinates, IDs, limits, and duplicate cells;
3. reject unsupported fields or incompatible versions;
4. normalize ordering into typed Rust structures; and
5. serialize the normalized structures using the canonical binary format before hashing or simulation.

Authoritative JSON values MUST use integer coordinates, IDs, timers, and counts. Floating-point values are excluded from Stage 1 authoritative data.

The checked-in Stage 1 map is [`assets/maps/stage1.json`](../assets/maps/stage1.json). The checked-in file-driven input and expected-output fixtures are [`replays/stage1-input.json`](../replays/stage1-input.json) and [`replays/stage1-expected.json`](../replays/stage1-expected.json).

```json
{
  "schema_version": 1,
  "ruleset_id": "pipe_arena_stage1",
  "width": 13,
  "height": 11,
  "indestructible_walls": [{ "x": 2, "y": 2 }],
  "destructible_walls": [{ "x": 3, "y": 3 }],
  "spawns": [
    { "actor": 1, "x": 1, "y": 1 },
    { "actor": 2, "x": 11, "y": 9 }
  ]
}
```
