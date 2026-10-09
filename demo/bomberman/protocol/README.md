# Bomberman protocol

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

State hashes use canonical fixed-width little-endian state bytes and FNV-1a 64. The hash includes protocol version, ruleset ID, tick, arena dimensions, typed ruleset values, sorted player state, sorted bomb state, sorted indestructible-wall state, sorted destructible-wall state, and the next bomb ID.

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
| Authored gameplay description | JSON | Parsed and normalized by `bevy_gamesim_ast`; the core build emits one generated definition |
| Arena maps and asset manifests | JSON | Cross-engine, versioned, validated before use |
| Authored input fixtures | JSON | Human-readable source converted to canonical `TickInputFrame` bytes |
| Tick inputs and replay streams | Canonical binary | Fixed-width little-endian fields; no parser-dependent representation |
| Authoritative state hashes | Canonical binary state bytes | Hash the normalized state, never raw JSON |
| Definition identity | SHA-256 of normalized IR | Binds replay and fixture compatibility to gameplay data |
| Documentation and design notes | Markdown | Human-readable |
| Presentation assets | Native asset formats | PNG, OGG, GLTF, and engine-imported derivatives |

YAML MUST NOT be used for authoritative maps, rules, inputs, replays, or state. Its implicit typing, aliases, parser differences, and multiple equivalent representations make cross-language canonicalization needlessly fragile.

### Authored JSON rules

`demo/bomberman/bomberman.json` remains the gameplay source. `bevy_gamesim_ast` parses it into a typed syntax tree, validates schema, ruleset, limits, IDs, coordinates, and duplicate cells, then normalizes wall/player ordering and computes a `definition_hash`. The `bevy_pipe_core` build script emits this data into `BOMBERMAN_DEFINITION`; `Simulation::from_definition` is the runtime boundary used by the fixture and Bevy client.

The presentation file contains colors, input bindings, movement delay, and UI text. It MUST NOT override gameplay spawns or bomb rules. The readable `.game` language is defined under `demo/bomberman/engine/bevy/lang/` and currently compiles to the same typed source model without replacing the JSON source.
