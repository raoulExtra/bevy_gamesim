# Pipe Arena

An original Bomberman-inspired arena for proving the Bevy GameSim pipe architecture. Stage 1 currently runs as an engine-neutral Rust simulation; Bevy, Godot, Unity, and Unreal clients are later presentation targets.

The demo is an original specification. Do not add commercial Bomberman assets, names, sounds, or branding.

## Goals

- deterministic fixed-tick simulation;
- typed input, intent, and command pipes;
- replay and rollback;
- authoritative multiplayer;
- protected Rust/WASM interpretation; and
- interchangeable engine presentation clients.

Only the first two goals plus basic replay verification are implemented in Stage 1.

## Layout

pipe_arena/
├── assets/              source art, audio, fonts, maps, and effects
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
- destructible walls;
- player elimination; and
- deterministic hashes and replay verification.

Later stages add persistent replay files, rollback, score/round rules, power-ups, network correction, and presentation clients.

## Authority split

```text
Rust simulation:  state, rules, collision, bombs, damage, replay
Engine client:    input capture, rendering, camera, animation, audio, UI
```

Future engine clients will consume versioned state snapshots/deltas and presentation events; they do not independently decide gameplay outcomes.

## Asset rules

- Prefer original or clearly licensed assets.
- Keep source files separate from processed/runtime files.
- Give every presentation asset a stable identifier.
- Do not pass engine object pointers through the simulation protocol.
- Put attribution and license information beside third-party assets.

See `assets/README.md`, `protocol/README.md`, and `docs/test-scenarios.md` before adding files.
