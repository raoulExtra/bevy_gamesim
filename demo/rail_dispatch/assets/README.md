# Rail Dispatch assets

The railway, signals, station, lighting, and UI are generated procedurally. Runtime train scenes are loaded from `rail_stock/runtime/`; editable sources stay under `rail_stock/source/`.

The runtime GLB paths are stable lowercase names and load through Bevy's `GltfAssetLabel::Scene(0)`. Licensing and provenance are recorded in `THIRD_PARTY_ASSETS.md`.
