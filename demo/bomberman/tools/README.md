# Tools

The source compiler is the reusable `bevy_gamesim_ast` package. Bomberman is compiled by the `bevy_pipe_core` build script so the client and fixture share one generated definition:

```text
Bomberman JSON
    -> parse_bomberman
    -> compile_bomberman
    -> normalized BombermanIr + definition_hash
    -> generate_bomberman_rust
    -> BOMBERMAN_DEFINITION
    -> Simulation::from_definition
```

Validate the checked-in source and replay fixture with:

```sh
cargo run -p bevy_pipe_core --example stage1
```

Compile the village JSON source independently with:

```sh
cargo run -p bevy_gamesim_ast --example village_compiler
```

The village compiler emits a normalized generic `VillageIr`, including seeded random local streets with higher probability near the center, and a definition hash; it does not replace the Bomberman build input or simulation.


Tools MUST NOT become an alternate gameplay implementation; authoritative behavior remains in the Rust simulation. The readable `.game` language under `demo/bomberman/engine/bevy/lang/` is an additional parser path to the same typed AST; JSON remains the current build source.
