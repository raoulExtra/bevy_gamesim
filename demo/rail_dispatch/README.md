# Rail Dispatch

A 3D railway-control demo for proving deterministic piping across multiple routes, signals, switches, and train actors. It is separate from the 2D `bomberman` demo while sharing the same Rust-owned simulation architecture.

## Run

From the repository root:

```sh
cargo run -p rail_dispatch_bevy --bin rail_dispatch
```

The current Bevy client renders a procedural railway yard with a closed loop, an inner siding, animated trains, signals, a station, and an orbit camera. Arrow keys orbit the camera, `+`/`-` zoom, and `R` resets the view.

## Player controls

Click the Rail Dispatch window to give it keyboard focus:

- `1`, `2`, `3` or `Tab`: select a train;
- `Q` / `E`: decrease or increase the selected train's speed by 10;
- `F1`–`F5`: select named signals `S1`–`S5`;
- `S`: set the selected signal to manual stop;
- `G`: set the selected signal to manual clear; and
- `A`: return the selected signal to automatic protection.

The in-game panel shows the selected train, speed, named signal aspects, and manual/automatic state.

## Layout

```text
rail_dispatch/
├── assets/       train models, source files, and provenance
├── config/       railway presentation and signal settings
├── engine/bevy/  native Bevy presentation client
├── docs/         demo-specific design notes
├── protocol/     versioned wire and replay schemas
├── replays/      recorded input streams and fixtures
└── tools/        import and validation tools
```

The simulation remains engine-neutral. Bevy owns presentation, input capture, camera, animation, and UI; authoritative state and rules belong in the shared Rust protocol/simulation layer.
