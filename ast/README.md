# GameSim AST compiler

`bevy_gamesim_ast` owns authored game-description parsing, schema validation, normalization, and definition identity. It is deliberately independent of `bevy_pipe_core` so future demos can add adapters without a dependency cycle.

The first backend is Bomberman:

```text
bomberman.json
  -> parse_bomberman
  -> compile_bomberman
  -> normalized IR + canonical bytes + SHA-256 definition_hash
  -> generate_bomberman_rust

bomberman.game
  -> parse_bomberman_lang
  -> the same BombermanSource/compiler path
```

The generated Rust contains data only. Gameplay mechanics remain handwritten in `bevy_pipe_core`; the core validates the generated map and constructs the runtime simulation.

Schema policy:

- `schema_version` is required and currently `1`.
- `ruleset_id` is required and must match the consumer's selected ruleset.
- Unknown fields are rejected.
- Wall and player ordering is normalized before hashing.
- `definition_hash` identifies the normalized gameplay definition and is separate from the per-state hash.

Adding a future demo should add a source schema and compiler in this crate, then add a consumer-specific build adapter. Do not make the AST crate depend on a runtime engine or on the core crate.
