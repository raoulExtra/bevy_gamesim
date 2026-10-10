# Bevy GameSim: Munich railway=rail trains

A small Bevy demo using the extracted OpenStreetMap `railway=rail` ways around München Hbf–München-Pasing.

## Run

```bash
cd /tmp/bevy_gamesim
cargo run
```

The app opens a 1400×900 window and displays:

- the extracted rail network, colored by `service` tag;
- fourteen connected graph routes with station endpoints at München Hbf, Pasing, Laim, Hirschgarten, Donnersberger Brücke, and Hackerbrücke;
- exactly ten locomotive emoji-style markers, distributed across routes and traveling in both directions; each has a round route-colored marker centered on its current track;
- `RailSwitch` markers at deduplicated `service=crossover` locations plus rendered route endpoints and junctions; red X means straight and green X means diverging;
- route-junction switches connect nearby rendered routes so every displayed route has a reachable transfer point;
- endpoint handoff: when a locomotive reaches a route end, it switches to the next route and reverses direction;
- station callouts use offset labels, leader lines, and circles centered on München Hbf, München-Pasing, Laim, Hirschgarten, Donnersberger Brücke, and Hackerbrücke;
- a status line with locomotive, OSM way, route, and RailSwitch counts.

Controls:

- `+` / `=`: zoom in, including deep zoom down to track-level detail;
- `-`: zoom out;
- `ArrowLeft` / `ArrowRight`: pan the screen horizontally, including while deeply zoomed.

The simulation is illustrative. Switch routing uses the nearest connected rendered route rather than a full signaling/interlocking model; it does not model timetables, platform assignment, dispatching, speed limits, or train conflicts.

## Data

`assets/railway_rail.osm.json` is the raw OSM/Overpass extraction. `assets/railway_rail.geojson` is included for comparison and external GIS review. Only ways tagged `railway=rail` were extracted; sidings, yards, spurs, and crossovers are retained when they carry that tag.
