# Bevy client

Native Bevy presentation client. Consume the shared protocol and keep rendering, input, UI, audio, and effects outside authoritative simulation.

## Run

From the repository root:

```sh
cargo run -p bomberman_bevy
```


## Configuration

Gameplay and presentation configuration are intentionally separate:

- `demo/bomberman/bomberman.json` remains the gameplay source for the core build. The readable language grammar and parallel `.game` example live in `lang/`; both map to the same typed gameplay model;
- `demo/bomberman/config/stage1.json` owns presentation-only settings:
  - `movement_delay_ms`: repeat delay for held movement;
  - `help_hint`: lower-left text shown during play;
  - `win_format`: lower-left text shown when exactly one player remains; `{score1}` and `{score2}` are replaced with the round score (`1:0` or `0:1`);
  - each player's `color` and `keys`: `KeyW`, `KeyA`, `KeyS`, `KeyD`, `ArrowUp`, `ArrowRight`, `ArrowDown`, `ArrowLeft`, `Space`, or `Enter`.

The presentation file may select colors and input bindings only for actors defined by the generated gameplay definition. It MUST NOT redefine gameplay rules or spawns.