# Bevy client

Native Bevy presentation client. Consume the shared protocol and keep rendering, input, UI, audio, and effects outside authoritative simulation.

## Run

From the repository root:

```sh
cargo run -p pipe_arena_bevy
```

The client loads `demo/pipe_arena/config/stage1.json` for the presentation settings and `demo/pipe_arena/assets/maps/stage1.json` for arena geometry. `F1 = ?` in the configured lower-left hint toggles the help panel. The config controls movement delay, bomb explosion delay, help hint text, win text, player colors, keyboard bindings, and initial player positions. The configured positions are applied to the validated map before the shared `bevy_pipe_core` simulation starts.

## Configuration

Edit `demo/pipe_arena/config/stage1.json`:

- `movement_delay_ms`: repeat delay for held movement;
- `bomb_explosion_delay_ms`: delay from bomb placement until its explosion; it is rounded up to 64 Hz simulation ticks;
- `help_hint`: lower-left text shown during play;
- `win_format`: lower-left text shown when exactly one player remains; `{score1}` and `{score2}` are replaced with the round score (`1:0` or `0:1`);
- each player's `color`: RGB values from `0` to `255`;
- each player's `keys`: `KeyW`, `KeyA`, `KeyS`, `KeyD`, `ArrowUp`, `ArrowRight`, `ArrowDown`, `ArrowLeft`, `Space`, or `Enter`; and
- each player's `initial_position`: validated arena cell coordinates.