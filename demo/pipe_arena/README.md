# Pipe Arena

An original Bomberman-inspired arena for proving the Bevy GameSim pipe architecture. Stage 1 runs as an engine-neutral Rust simulation with a file-driven map/input/output fixture pipeline and a Bevy presentation client.

The demo is an original specification. Do not add commercial Bomberman assets, names, sounds, or branding.

## Goals

- deterministic fixed-tick simulation;
- typed input, intent, and command pipes;
- replay and rollback;
- authoritative multiplayer;
- protected Rust/WASM interpretation; and
- interchangeable engine presentation clients.

Stage 1 includes the deterministic simulation, JSON map/input/output fixtures, file-driven divergence reporting, and the Bevy presentation client.

## Layout

pipe_arena/
├── assets/              source art, audio, fonts, maps, and effects
├── config/              versioned client controls, colors, keys, and positions
├── docs/                demo-specific design and production notes
├── engine/
│   ├── bevy/            Bevy presentation client
│   ├── godot/           Godot adapter/client
│   ├── unity/           Unity adapter/client
│   └── unreal/          Unreal adapter/client
├── protocol/            versioned wire and replay schemas
├── replays/             recorded input streams and test fixtures
└── tools/               import, validation, and asset-processing tools
```

## Initial game scope

Stage 1 implements:

- 13×11 grid arena;
- two players;
- fixed tick identity;
- four-direction grid movement;
- bomb placement;
- fixed fuse and blast rules;
- indestructible and destructible walls;
- player elimination;
- deterministic hashes and replay verification; and
- a Bevy client that renders authoritative state and logs presentation events.

Later stages add persistent replay files, rollback, score/round rules, power-ups, network correction, and additional engine adapters.

## Authority split

```text
Rust simulation:  state, rules, collision, bombs, damage, replay
Engine client:    input capture, rendering, camera, animation, audio, UI
```

The Bevy client consumes authoritative state and presentation events; it does not independently decide gameplay outcomes.

## Asset rules

- Prefer original or clearly licensed assets.
- Keep source files separate from processed/runtime files.
- Give every presentation asset a stable identifier.
- Do not pass engine object pointers through the simulation protocol.
- Put attribution and license information beside third-party assets.

See `assets/README.md`, `config/stage1.json`, `protocol/README.md`, and `docs/test-scenarios.md` before adding files. Run the file-driven fixture with `cargo run -p bevy_pipe_core --example stage1`.
