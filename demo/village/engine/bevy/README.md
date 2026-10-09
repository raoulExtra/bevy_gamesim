# Village traffic sandbox

Native Bevy presentation client for the compiled village definition.

The client loads `demo/bomberman/engine/bevy/lang/village.json`, compiles its seeded local streets through `bevy_gamesim_ast`, and renders the village map, center traffic signal, connected local roads, and twelve moving emoji agents.

## Run

From the repository root:

```sh
cargo run -p village_bevy
```

The window is titled `Village traffic sandbox`.
