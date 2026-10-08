# Bevy client

Native Bevy presentation client. Consume the shared protocol and keep rendering, input, UI, audio, and effects outside authoritative simulation.

## Run

From the repository root:

```sh
cargo run -p pipe_arena_bevy
```

Player 1 uses `WASD` and `Space`; player 2 uses the arrow keys and `Enter`. The client loads the checked-in Stage 1 map, steps the shared `bevy_pipe_core` simulation at a fixed rate, renders authoritative state, and logs presentation events.