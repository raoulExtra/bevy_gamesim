# Bomberman

An original Bomberman-inspired arena for proving the Bevy GameSim pipe architecture. Stage 1 runs as an engine-neutral Rust simulation with a file-driven map/input/output fixture pipeline and a Bevy presentation client.

The demo is an original specification. Do not add commercial Bomberman assets, names, sounds, or branding.

## Goals

- deterministic fixed-tick simulation;
- typed input, intent, and command pipes;
- replay and rollback;
- authoritative multiplayer;
- protected Rust/WASM interpretation; and
- interchangeable engine presentation clients.

- `bomberman.json` remains the authored gameplay description; `bevy_gamesim_ast` parses, validates, normalizes, and hashes it during the core build, then emits one generated runtime definition;
- `engine/bevy/lang/bomberman.game` defines the readable game-language syntax and parses to the same typed model without replacing JSON;
- Stage 1 includes the deterministic simulation, JSON description/input/output fixtures, file-driven divergence reporting, and the Bevy presentation client.

## Layout

```text
bomberman/
├── engine/
│   ├── bevy/
│   │   ├── lang/       gameplay language grammar and source
│   │   └── ...         Bevy presentation client
│   ├── godot/          Godot adapter/client
│   ├── unity/          Unity adapter/client
│   └── unreal/         Unreal adapter/client
├── assets/             source art, audio, fonts, maps, and effects
├── config/             versioned client colors, controls, delays, and win text
├── protocol/           versioned wire and replay schemas
├── replays/            recorded input streams and test fixtures
└── tools/              import, validation, and asset-processing tools
```

## Initial game scope

Stage 1 implements:

- 500×500 authoritative grid arena;
- two players;
- fixed tick identity;
- four-direction grid movement;
- bomb placement and ruleset-configured explosion delay;
- 100 deterministic random bombs at ticks 0, 100, 200, and subsequent intervals;
- indestructible and destructible walls;
- player elimination;
- lower-left winner and round-score status;
- deterministic hashes and replay verification; and
- a Bevy client that renders authoritative state and logs presentation events.

Later stages add persistent replay files, rollback, persistent score/round rules, power-ups, network correction, and additional engine adapters.

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

See `engine/bevy/lang/README.md`, `engine/bevy/lang/bomberman.game`, `assets/README.md`, `config/stage1.json`, `protocol/README.md`, and `docs/test-scenarios.md` before adding files. Run the description-to-runtime fixture with `cargo run -p bevy_pipe_core --example stage1`.
