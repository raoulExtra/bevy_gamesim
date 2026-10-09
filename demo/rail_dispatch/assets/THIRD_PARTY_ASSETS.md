# Third-party assets

This directory contains third-party presentation assets used by the Rail Dispatch demo.

## Quaternius Modular Train Pack

- **Creator:** Quaternius
- **Pack:** [Modular Train Pack](https://quaternius.com/packs/modulartrain.html)
- **Download source:** [Poly Pizza bundle](https://poly.pizza/bundle/Modular-Train-Pack-jYEybkFVr1)
- **Downloaded FBX archive:** `Modular Train Pack-zip.zip`
- **Downloaded GLB archive:** `Modular Train Pack-glb.zip` from `https://static.poly.pizza/list/jYEybkFVr1-glb--249349190.zip`
- **License:** [Creative Commons CC0 1.0 Universal](https://creativecommons.org/publicdomain/zero/1.0/)
- **Permitted use:** copying, modification, commercial use, and redistribution without required attribution.
- **Imported source format:** FBX source files.
- **Runtime format:** GLB scenes loaded by Bevy 0.15.
- **Source path:** `rail_stock/source/quaternius_modular_train/`
- **Runtime path:** `rail_stock/runtime/quaternius_modular_train/`

The downloaded pack contains these source models:

- `CargoTrain_CoalContainer.fbx`
- `CargoTrain_Container.fbx`
- `CargoTrain_Front.fbx`
- `CargoTrain_Wagon.fbx`
- `HighSpeed_Front.fbx`
- `HighSpeed_Wagon.fbx`
- `Locomotive_Front.fbx`
- `Locomotive_PassengerWagon.fbx`
- `Locomotive_Wagon.fbx`
- `RailwayTrack_Curve.fbx`
- `RailwayTrack_Straight.fbx`

The runtime GLB directory contains the corresponding normalized lowercase files:

- `cargo_train_coal_container.glb`
- `cargo_train_container.glb`
- `cargo_train_front.glb`
- `cargo_train_wagon.glb`
- `high_speed_front.glb`
- `high_speed_wagon.glb`
- `locomotive_front.glb`
- `locomotive_passenger_carriage.glb`
- `locomotive_wagon.glb`
- `railway_track_curve.glb`
- `railway_track_straight.glb`

The official pack page identifies 15 models, FBX/OBJ/Blend formats, and CC0 licensing. The Poly Pizza distribution used here provides the downloadable FBX and GLB archives and identifies the bundle as Public Domain (CC0).

## Attribution policy

CC0 does not require attribution. This file records provenance for maintainability and redistribution audits. Do not relicense these files as MIT; retain the original CC0 terms.

## Other approved sources

These sources were reviewed for future environment and material assets. No files from them are currently vendored here:

- [Poly Haven license](https://polyhaven.com/license) — all listed HDRIs, textures, and 3D models are CC0.
- [ambientCG license](https://docs.ambientcg.com/license/) — downloadable assets are CC0.

## Munich-inspired S-Bahn GLBs

The red commuter train scenes are authored locally as low-poly geometry and materials; no photograph pixels, logos, or third-party mesh are embedded in the runtime files.

- **Reference subject:** DBAG Class 423 of S-Bahn München.
- **Primary reference:** [DBAG Class 423 of S-Bahn München](https://commons.wikimedia.org/wiki/File:DBAG_Class_423_of_S-Bahn_M%C3%BCnchen.jpg)
- **Reference author:** Wilfredor.
- **Reference license:** CC0 1.0 Universal Public Domain Dedication.
- **Additional reference index:** [Wikimedia Commons Class 423 category](https://commons.wikimedia.org/wiki/Category:DBAG_Class_423_of_S-Bahn_M%C3%BCnchen).
- **Authoring source:** `rail_stock/source/munich_sbahn/build_munich_sbahn.py`.
- **Runtime scenes:** `rail_stock/runtime/munich_sbahn/munich_sbahn_front.glb` and `munich_sbahn_carriage.glb`.

The GLBs are generated project assets. Their red/ivory livery is an original approximation of the reference train, not a photographic or logo reproduction.
