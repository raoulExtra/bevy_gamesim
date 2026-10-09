# Rail Dispatch Bevy client

Native Bevy presentation client for the 3D railway-control demo. Rendering, input, UI, camera, animation, and effects remain outside the authoritative simulation.

## Run

From the repository root:

```sh
cargo run -p rail_dispatch_bevy --bin rail_dispatch
```

The client renders a closed railway loop with four quarter-turn curves, an inner siding, five animated signals, three moving GLB trains including the Munich-inspired red S-Bahn consist, a procedural station, directional lighting, and an orbit camera. Runtime scenes are loaded from `demo/rail_dispatch/assets/rail_stock/runtime/`.

Arrow keys orbit the camera, `+`/`-` zoom, and `R` resets the view.

Player controls:

- `1`, `2`, `3` or `Tab`: select a train;
- `Q` / `E`: decrease or increase its speed by 10;
- `F1`–`F5`: select named signals `S1`–`S5`;
- `S`: manual stop;
- `G`: manual clear; and
- `A`: automatic signal protection.

## Configuration

The client reads `demo/rail_dispatch/config/rail_dispatch.json`:

- `signal_block_length`: track distance protected after each signal;
- `signal_stop_distance`: distance before a red signal where a vehicle stops; and
- `green_delay_ms`: hold time before an unoccupied red signal returns to green.
