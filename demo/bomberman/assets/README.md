# Assets

Place source and runtime-ready presentation assets here. The Rust simulation uses stable identifiers, not engine object references.

```text
assets/
├── characters/   player sprites, models, portraits
├── tiles/        floor, wall, border, grid textures
├── props/        bombs, power-ups, breakable objects
├── effects/      explosions, smoke, hit and spawn effects
├── audio/        sound effects and UI sounds
├── music/        music stems and tracks
├── ui/           HUD, menus, icons, buttons
├── fonts/        font files and font licenses
└── maps/         authored arena layouts and thumbnails
```

## Naming

Use stable, lowercase, engine-neutral names:

```text
characters/player_red/source/player_red.svg
props/bomb/source/bomb.svg
props/bomb/runtime/bomb.png
```

Use `source/` for editable originals and `runtime/` for processed files. Keep licensing and attribution in `../docs/licenses/` or next to the relevant asset.


## Required initial assets

- player color variants;
- floor and indestructible wall tiles;
- destructible wall tile;
- bomb;
- explosion frames or effect;
- power-up icons;
- round/winner UI; and
- basic placement, explosion, damage, and defeat sounds.
