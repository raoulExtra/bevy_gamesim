# Game language and JSON sources

The `.game` language is an additional readable authoring syntax for the Bevy engine adapter. JSON remains the current Bomberman gameplay source and build input. The language parses into the same typed `BombermanSource`, then uses the normal validation, normalization, hashing, and generated-runtime path when explicitly consumed.

## Syntax

```text
game <ruleset_id> {
  schema <integer>

  arena {
    size <width> by <height>
    indestructible_walls { (<x>, <y>), ... }
    destructible_walls { (<x>, <y>), ... }
  }

  rules {
    tick_rate <integer> hz
    bomb {
      fuse <integer> ticks
      radius <integer> cells
      max_per_player <integer>
    }
    random_bombs {
      batch <integer>
      every <integer> ticks
      seed <integer-or-hex>
    }
  }

  player <actor> at (<x>, <y>)
}
```

## Design rules

- `game` names the ruleset and is compiled into the versioned `ruleset_id`.
- `schema`, rules, walls, and players are required; omission is a parse error.
- Coordinates are integer grid cells, not Bevy transforms.
- `hz`, `ticks`, and `cells` are explicit units for readability and validation.
- `//` and `#` comments are supported.
- The language contains no presentation fields, engine object references, executable code, or implicit wall-clock behavior.
- Ordering is not authoritative: the compiler sorts players and wall cells before hashing.
- Unknown keywords and malformed values fail before the simulation is built.

The language is intentionally small. New gameplay concepts should first acquire a typed protocol representation and authority rules in `bevy_pipe_core`; syntax is added only after that contract exists.

## Village JSON compiler

`village.json` is the source for the generic village compiler in `bevy_gamesim_ast`. It describes a 1000 m × 1000 m village traffic sandbox:

- six primary streets leave the village center;
- the seeded generator produces thirty connected local streets, for 36 streets total;
- pedestrians prefer local streets with an 80% route preference, falling back to outgoing streets;
- every street has at least one endpoint connection to another street;
- streets render as lines;
- street paths use four named types: `straight`, `gentle_arc`, `s_bend`, and `hairpin`;
- red traffic lights stop cars, bicycles, motorcycles, mopeds, and e-scooters;
- pedestrians have priority over every street vehicle at the crossing;
- twelve deterministic agents use car, bicycle, motorcycle, moped, e-scooter, and pedestrian emoji presentation markers.

The compiler parses this JSON into typed source data, validates the traffic contract and monotonic center-density rule, generates the seeded local streets, normalizes ordering, emits canonical JSON bytes, and computes a SHA-256 definition hash:

Run the executable compiler with:

```sh
cargo run -p bevy_gamesim_ast --example village_compiler
```

`village.game` remains a readable design sketch for the same scenario. It is not an input to the Bomberman compiler.
