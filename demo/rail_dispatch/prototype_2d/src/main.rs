use bevy::input::mouse::MouseWheel;
use bevy::prelude::*;
use serde::Deserialize;
use std::cmp::Ordering;
use std::collections::{BinaryHeap, HashMap};
use std::time::{SystemTime, UNIX_EPOCH};

const HBF: GeoPoint = GeoPoint {
    lon: 11.5586,
    lat: 48.1402,
};
const PASING: GeoPoint = GeoPoint {
    lon: 11.4616,
    lat: 48.1494,
};
const LAIM: GeoPoint = GeoPoint {
    lon: 11.4908,
    lat: 48.1426,
};
const HIRSCHGARTEN: GeoPoint = GeoPoint {
    lon: 11.5007,
    lat: 48.1435,
};
const DONNERSBERGER_BRUECKE: GeoPoint = GeoPoint {
    lon: 11.5327,
    lat: 48.1430,
};
const HACKERBRUECKE: GeoPoint = GeoPoint {
    lon: 11.5482,
    lat: 48.1431,
};
const SWITCH_ROUTE_JOIN_DISTANCE: f32 = 50.0;
const ROUTE_JUNCTION_OFFSET: f32 = 18.0;
const SWITCH_APPROACH_DISTANCE: f32 = 90.0;
const MIN_CAMERA_SCALE: f32 = 0.02;
const LOCOMOTIVE_COUNT: usize = 10;
const CAMERA_PAN_SPEED: f32 = 700.0;

#[derive(Clone, Copy, Debug)]
struct GeoPoint {
    lon: f64,
    lat: f64,
}

#[derive(Deserialize)]
struct OsmData {
    elements: Vec<OsmElement>,
}

#[derive(Deserialize)]
#[serde(tag = "type")]
enum OsmElement {
    #[serde(rename = "way")]
    Way(OsmWay),
    #[serde(other)]
    Other,
}

#[derive(Deserialize)]
struct OsmWay {
    #[serde(rename = "id")]
    _id: u64,
    nodes: Vec<u64>,
    geometry: Vec<OsmPoint>,
    tags: Option<HashMap<String, String>>,
}

#[derive(Deserialize, Clone, Copy)]
struct OsmPoint {
    lon: f64,
    lat: f64,
}

#[derive(Clone, Copy)]
struct RailSegment {
    start: Vec2,
    end: Vec2,
    service: RailService,
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum RailService {
    Main,
    Yard,
    Siding,
    Crossover,
    Spur,
}

#[derive(Clone)]
struct RailRoute {
    points: Vec<Vec2>,
    length: f32,
}

#[derive(Resource)]
struct RailNetwork {
    segments: Vec<RailSegment>,
    routes: Vec<RailRoute>,
    way_count: usize,
    switch_count: usize,
}

#[derive(Resource, Clone, Copy)]
struct MapTransform {
    center: GeoPoint,
    scale: f32,
}

#[derive(Component)]
struct Locomotive {
    route_index: usize,
    phase: f32,
    speed: f32,
    direction: f32,
}
#[derive(Component)]
struct RailSwitch {
    position: Vec2,
    routes: Vec<SwitchRoute>,
    active: bool,
    armed: bool,
}

#[derive(Clone, Copy)]
struct SwitchRoute {
    route_index: usize,
    distance: f32,
}

#[derive(Resource)]
struct SwitchRng(u64);

#[derive(Component, Clone, Copy)]
struct StationMarker {
    station: Vec2,
    label: Vec2,
}

#[derive(Component)]
struct StatusText;

#[derive(Clone, Copy, Debug)]
struct QueueState {
    cost: f64,
    node: u64,
}

impl PartialEq for QueueState {
    fn eq(&self, other: &Self) -> bool {
        self.cost == other.cost && self.node == other.node
    }
}

impl Eq for QueueState {}

impl Ord for QueueState {
    fn cmp(&self, other: &Self) -> Ordering {
        other
            .cost
            .partial_cmp(&self.cost)
            .unwrap_or(Ordering::Equal)
            .then_with(|| self.node.cmp(&other.node))
    }
}

impl PartialOrd for QueueState {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

fn main() {
    App::new()
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                title: "Munich railway=rail · Bevy GameSim".into(),
                resolution: (1400.0_f32, 900.0_f32).into(),
                resizable: true,
                ..default()
            }),
            ..default()
        }))
        .add_systems(Startup, setup)
        .add_systems(
            Update,
            (
                draw_railway,
                draw_station_markers,
                update_switches,
                animate_locomotives,
                draw_train_dots,
                update_status,
                zoom_camera,
            ),
        )
        .run();
}

fn setup(mut commands: Commands, asset_server: Res<AssetServer>) {
    let osm_text = std::fs::read_to_string("assets/railway_rail.osm.json")
        .expect("assets/railway_rail.osm.json must be present");
    let osm: OsmData = serde_json::from_str(&osm_text).expect("valid OSM JSON");
    let (segments, graph, coordinates, bounds, way_count) = build_network(&osm);

    let center = GeoPoint {
        lon: (bounds.0.lon + bounds.1.lon) / 2.0,
        lat: (bounds.0.lat + bounds.1.lat) / 2.0,
    };
    let x_span = meters_x(bounds.1.lon - bounds.0.lon, center.lat).abs();
    let y_span = meters_y(bounds.1.lat - bounds.0.lat).abs();
    let scale = (1200.0 / x_span.min(1.0e9) as f32)
        .min(720.0 / y_span.min(1.0e9) as f32)
        .max(0.0001)
        * 0.92;
    let map_transform = MapTransform { center, scale };

    let route_specs = vec![
        (HBF, PASING),
        (PASING, HBF),
        (
            HBF,
            GeoPoint {
                lon: bounds.0.lon,
                lat: center.lat,
            },
        ),
        (
            PASING,
            GeoPoint {
                lon: center.lon,
                lat: bounds.1.lat,
            },
        ),
        (
            HBF,
            GeoPoint {
                lon: center.lon,
                lat: bounds.0.lat,
            },
        ),
        (
            PASING,
            GeoPoint {
                lon: bounds.1.lon,
                lat: center.lat,
            },
        ),
        (HBF, LAIM),
        (LAIM, HBF),
        (HBF, HIRSCHGARTEN),
        (HIRSCHGARTEN, HBF),
        (HBF, DONNERSBERGER_BRUECKE),
        (DONNERSBERGER_BRUECKE, HBF),
        (HBF, HACKERBRUECKE),
        (HACKERBRUECKE, HBF),
    ];
    let routes: Vec<RailRoute> = route_specs
        .into_iter()
        .filter_map(|(source, target)| {
            let start = nearest_node(&coordinates, source);
            let goal = nearest_node(&coordinates, target);
            let nodes = shortest_path(&graph, start, goal)?;
            let points: Vec<Vec2> = nodes
                .iter()
                .filter_map(|node| coordinates.get(node).copied())
                .map(|point| project(point, map_transform))
                .collect();
            let length = polyline_length(&points);
            (points.len() >= 2 && length > 10.0).then_some(RailRoute { points, length })
        })
        .collect();
    assert!(
        routes.len() >= 2,
        "at least two connected train routes are required"
    );

    let train_routes = routes.clone();
    let mut switch_entities = Vec::new();
    let mut switch_positions: Vec<Vec2> = Vec::new();
    for segment in segments
        .iter()
        .filter(|segment| segment.service == RailService::Crossover)
    {
        let position = project(
            GeoPoint {
                lon: (segment.start.x + segment.end.x) as f64 * 0.5,
                lat: (segment.start.y + segment.end.y) as f64 * 0.5,
            },
            map_transform,
        );
        add_route_switch(
            position,
            &train_routes,
            &mut switch_positions,
            &mut switch_entities,
            false,
        );
    }

    // Add switches at route endpoints and where two rendered routes diverge
    // or rejoin, so every route has a reachable transfer point.
    for route in &train_routes {
        let endpoint_positions = [
            route.points.first().copied(),
            route.points.last().copied(),
            Some(sample_polyline(
                &route.points,
                ROUTE_JUNCTION_OFFSET.min(route.length),
            )),
            Some(sample_polyline(
                &route.points,
                (route.length - ROUTE_JUNCTION_OFFSET).max(0.0),
            )),
        ];
        for position in endpoint_positions.into_iter().flatten() {
            add_route_switch(
                position,
                &train_routes,
                &mut switch_positions,
                &mut switch_entities,
                true,
            );
        }
    }
    for left_index in 0..train_routes.len() {
        for right_index in (left_index + 1)..train_routes.len() {
            let right = &train_routes[right_index];
            let mut was_near = false;
            for &position in &train_routes[left_index].points {
                let near = nearest_route_position(&right.points, position)
                    .is_some_and(|(distance, _)| distance <= SWITCH_ROUTE_JOIN_DISTANCE);
                if near != was_near {
                    add_route_switch(
                        position,
                        &train_routes,
                        &mut switch_positions,
                        &mut switch_entities,
                        true,
                    );
                    was_near = near;
                }
            }
        }
    }
    let switch_count = switch_entities.len();
    let seed = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_nanos() as u64
        | 1;
    commands.insert_resource(SwitchRng(seed));
    for switch in switch_entities {
        commands.spawn(switch);
    }
    commands.insert_resource(map_transform);
    commands.insert_resource(RailNetwork {
        segments,
        routes,
        way_count,
        switch_count,
    });
    commands.spawn(Camera2dBundle::default());

    let font = asset_server.load("fonts/NotoSans-Regular.ttf");
    let train_texture = asset_server.load("train_emoji.png");
    for (name, station, label_offset, size) in [
        ("München Hbf", HBF, Vec2::new(42.0, -48.0), 28.0),
        ("München-Pasing", PASING, Vec2::new(-58.0, 52.0), 28.0),
        ("Laim", LAIM, Vec2::new(-36.0, -52.0), 22.0),
        ("Hirschgarten", HIRSCHGARTEN, Vec2::new(-8.0, 58.0), 22.0),
        (
            "Donnersberger Brücke",
            DONNERSBERGER_BRUECKE,
            Vec2::new(34.0, -58.0),
            22.0,
        ),
        ("Hackerbrücke", HACKERBRUECKE, Vec2::new(58.0, 52.0), 22.0),
    ] {
        spawn_station_label(
            &mut commands,
            font.clone(),
            name,
            project(station, map_transform),
            label_offset,
            size,
        );
    }

    for number in 0..LOCOMOTIVE_COUNT {
        let route_index = number % train_routes.len();
        let direction = if number % 3 == 0 { -1.0 } else { 1.0 };
        let phase = ((number * 17) % 100) as f32 / 100.0;
        let route = &train_routes[route_index];
        let position = sample_polyline(&route.points, route.length * phase);
        commands
            .spawn(SpriteBundle {
                sprite: Sprite {
                    image: train_texture.clone(),
                    color: Color::WHITE,
                    custom_size: Some(Vec2::splat(24.0)),
                    ..default()
                },
                transform: Transform::from_translation(position.extend(10.0)),
                ..default()
            })
            .insert(Locomotive {
                route_index,
                phase,
                speed: 0.018 + number as f32 * 0.0012,
                direction,
            })
            .with_children(|parent| {
                parent.spawn((
                    Text2d::new(format!("{:02}", number + 1)),
                    TextFont {
                        font: font.clone(),
                        font_size: 15.0,
                        ..default()
                    },
                    TextColor(Color::WHITE),
                    Transform::from_translation(Vec3::new(0.0, 14.0, 0.0)),
                ));
            });
    }

    commands.spawn((
        Text::new("10 locomotives · OSM railway=rail · Hbf ⇄ Pasing · endpoint handoff"),
        TextFont {
            font: font.clone(),
            font_size: 19.0,
            ..default()
        },
        TextColor(Color::WHITE),
        Node {
            position_type: PositionType::Absolute,
            top: Val::Px(14.0),
            left: Val::Px(18.0),
            ..default()
        },
        StatusText,
    ));
}

fn spawn_station_label(
    commands: &mut Commands,
    font: Handle<Font>,
    value: &str,
    station: Vec2,
    label_offset: Vec2,
    size: f32,
) {
    let label = station + label_offset;
    commands.spawn((
        Text2d::new(value),
        TextFont {
            font,
            font_size: size,
            ..default()
        },
        TextColor(Color::srgb(1.0, 0.9, 0.15)),
        Transform::from_translation(label.extend(8.0)),
        StationMarker { station, label },
    ));
}

fn draw_station_markers(
    mut gizmos: Gizmos,
    stations: Query<&StationMarker>,
    cameras: Query<&OrthographicProjection, With<Camera>>,
) {
    let scale = cameras.single().scale;
    for marker in &stations {
        gizmos.line_2d(marker.station, marker.label, Color::srgb(1.0, 0.9, 0.15));
        gizmos.circle_2d(marker.station, 8.0 * scale, Color::srgb(1.0, 0.9, 0.15));
    }
}

fn build_network(
    osm: &OsmData,
) -> (
    Vec<RailSegment>,
    HashMap<u64, Vec<(u64, f64)>>,
    HashMap<u64, GeoPoint>,
    (GeoPoint, GeoPoint),
    usize,
) {
    let mut segments = Vec::new();
    let mut graph: HashMap<u64, Vec<(u64, f64)>> = HashMap::new();
    let mut coordinates = HashMap::new();
    let mut min = GeoPoint {
        lon: 180.0,
        lat: 90.0,
    };
    let mut max = GeoPoint {
        lon: -180.0,
        lat: -90.0,
    };
    let mut way_count = 0;

    for element in &osm.elements {
        let OsmElement::Way(way) = element else {
            continue;
        };
        way_count += 1;
        let service = classify_service(way.tags.as_ref());
        for point in &way.geometry {
            min.lon = min.lon.min(point.lon);
            min.lat = min.lat.min(point.lat);
            max.lon = max.lon.max(point.lon);
            max.lat = max.lat.max(point.lat);
        }
        for (node_id, point) in way.nodes.iter().zip(&way.geometry) {
            coordinates.insert(
                *node_id,
                GeoPoint {
                    lon: point.lon,
                    lat: point.lat,
                },
            );
        }
        for pair in way.nodes.windows(2).zip(way.geometry.windows(2)) {
            let (node_pair, point_pair) = pair;
            let a = GeoPoint {
                lon: point_pair[0].lon,
                lat: point_pair[0].lat,
            };
            let b = GeoPoint {
                lon: point_pair[1].lon,
                lat: point_pair[1].lat,
            };
            let start = Vec2::new(a.lon as f32, a.lat as f32);
            let end = Vec2::new(b.lon as f32, b.lat as f32);
            segments.push(RailSegment {
                start,
                end,
                service,
            });
            let weight = distance_meters(a, b);
            graph
                .entry(node_pair[0])
                .or_default()
                .push((node_pair[1], weight));
            graph
                .entry(node_pair[1])
                .or_default()
                .push((node_pair[0], weight));
        }
    }

    (segments, graph, coordinates, (min, max), way_count)
}

fn classify_service(tags: Option<&HashMap<String, String>>) -> RailService {
    match tags
        .and_then(|tags| tags.get("service"))
        .map(String::as_str)
    {
        Some("yard") => RailService::Yard,
        Some("siding") => RailService::Siding,
        Some("crossover") => RailService::Crossover,
        Some("spur") => RailService::Spur,
        _ => RailService::Main,
    }
}

fn shortest_path(graph: &HashMap<u64, Vec<(u64, f64)>>, start: u64, goal: u64) -> Option<Vec<u64>> {
    let mut distances: HashMap<u64, f64> = HashMap::new();
    let mut previous: HashMap<u64, u64> = HashMap::new();
    let mut queue = BinaryHeap::new();
    distances.insert(start, 0.0);
    queue.push(QueueState {
        cost: 0.0,
        node: start,
    });

    while let Some(QueueState { cost, node }) = queue.pop() {
        if node == goal {
            break;
        }
        if cost > *distances.get(&node).unwrap_or(&f64::INFINITY) {
            continue;
        }
        for &(next, edge_cost) in graph.get(&node).into_iter().flatten() {
            let next_cost = cost + edge_cost;
            if next_cost < *distances.get(&next).unwrap_or(&f64::INFINITY) {
                distances.insert(next, next_cost);
                previous.insert(next, node);
                queue.push(QueueState {
                    cost: next_cost,
                    node: next,
                });
            }
        }
    }

    if !distances.contains_key(&goal) {
        return None;
    }
    let mut path = vec![goal];
    let mut cursor = goal;
    while cursor != start {
        cursor = *previous.get(&cursor)?;
        path.push(cursor);
    }
    path.reverse();
    Some(path)
}

fn nearest_node(coordinates: &HashMap<u64, GeoPoint>, target: GeoPoint) -> u64 {
    coordinates
        .iter()
        .min_by(|(_, a), (_, b)| {
            distance_meters(**a, target)
                .partial_cmp(&distance_meters(**b, target))
                .unwrap_or(Ordering::Equal)
        })
        .map(|(node, _)| *node)
        .expect("OSM network must contain nodes")
}

fn project(point: GeoPoint, map: MapTransform) -> Vec2 {
    Vec2::new(
        meters_x(point.lon - map.center.lon, map.center.lat) as f32 * map.scale,
        meters_y(point.lat - map.center.lat) as f32 * map.scale,
    )
}

fn meters_x(delta_lon: f64, latitude: f64) -> f64 {
    delta_lon * 111_320.0 * latitude.to_radians().cos()
}

fn meters_y(delta_lat: f64) -> f64 {
    delta_lat * 110_540.0
}

fn distance_meters(a: GeoPoint, b: GeoPoint) -> f64 {
    let x = meters_x(a.lon - b.lon, (a.lat + b.lat) * 0.5);
    let y = meters_y(a.lat - b.lat);
    (x * x + y * y).sqrt()
}

fn polyline_length(points: &[Vec2]) -> f32 {
    points
        .windows(2)
        .map(|pair| pair[0].distance(pair[1]))
        .sum()
}

fn nearest_route_position(points: &[Vec2], target: Vec2) -> Option<(f32, f32)> {
    let mut best: Option<(f32, f32)> = None;
    let mut distance_along = 0.0;
    for pair in points.windows(2) {
        let segment = pair[0].distance(pair[1]);
        if segment <= f32::EPSILON {
            continue;
        }
        let delta = pair[1] - pair[0];
        let fraction = ((target - pair[0]).dot(delta) / delta.length_squared()).clamp(0.0, 1.0);
        let closest = pair[0] + delta * fraction;
        let distance = target.distance(closest);
        let candidate = (distance, distance_along + segment * fraction);
        if best.is_none_or(|current| candidate.0 < current.0) {
            best = Some(candidate);
        }
        distance_along += segment;
    }
    best
}

fn sample_polyline(points: &[Vec2], distance: f32) -> Vec2 {
    if points.len() < 2 {
        return points.first().copied().unwrap_or(Vec2::ZERO);
    }
    let mut remaining = distance.clamp(0.0, polyline_length(points));
    for pair in points.windows(2) {
        let segment = pair[0].distance(pair[1]);
        if remaining <= segment {
            return pair[0].lerp(pair[1], remaining / segment.max(0.0001));
        }
        remaining -= segment;
    }
    *points.last().unwrap()
}

fn add_route_switch(
    position: Vec2,
    train_routes: &[RailRoute],
    switch_positions: &mut Vec<Vec2>,
    switch_entities: &mut Vec<RailSwitch>,
    require_connection: bool,
) {
    if switch_positions
        .iter()
        .any(|known| known.distance(position) < 12.0)
    {
        return;
    }
    let routes = train_routes
        .iter()
        .enumerate()
        .filter_map(|(route_index, route)| {
            let (distance, route_distance) = nearest_route_position(&route.points, position)?;
            (distance <= SWITCH_ROUTE_JOIN_DISTANCE).then_some(SwitchRoute {
                route_index,
                distance: route_distance,
            })
        })
        .collect::<Vec<_>>();
    if require_connection && routes.len() < 2 {
        return;
    }
    switch_positions.push(position);
    switch_entities.push(RailSwitch {
        position,
        routes,
        active: false,
        armed: false,
    });
}

fn route_color(route_index: usize) -> Color {
    match route_index % 6 {
        0 => Color::srgb(1.0, 0.75, 0.05),
        1 => Color::srgb(0.95, 0.55, 0.12),
        2 => Color::srgb(0.15, 0.80, 0.95),
        3 => Color::srgb(0.95, 0.30, 0.75),
        4 => Color::srgb(0.55, 0.40, 1.0),
        _ => Color::srgb(0.20, 0.90, 0.45),
    }
}

fn draw_railway(
    mut gizmos: Gizmos,
    network: Res<RailNetwork>,
    map: Res<MapTransform>,
    switches: Query<&RailSwitch>,
) {
    for segment in &network.segments {
        let color = match segment.service {
            RailService::Main => Color::srgb(0.20, 0.28, 0.52),
            RailService::Yard => Color::srgb(0.75, 0.20, 0.16),
            RailService::Siding => Color::srgb(0.82, 0.48, 0.08),
            RailService::Crossover | RailService::Spur => Color::srgb(0.08, 0.55, 0.38),
        };
        gizmos.line_2d(
            project(
                GeoPoint {
                    lon: segment.start.x as f64,
                    lat: segment.start.y as f64,
                },
                *map,
            ),
            project(
                GeoPoint {
                    lon: segment.end.x as f64,
                    lat: segment.end.y as f64,
                },
                *map,
            ),
            color,
        );
    }
    for (index, route) in network.routes.iter().enumerate() {
        let color = route_color(index);
        for pair in route.points.windows(2) {
            gizmos.line_2d(pair[0], pair[1], color);
        }
    }
    for switch in &switches {
        let color = if switch.active {
            Color::srgb(0.20, 0.95, 0.35)
        } else {
            Color::srgb(0.95, 0.25, 0.20)
        };
        let arm = Vec2::splat(5.0);
        gizmos.line_2d(switch.position - arm, switch.position + arm, color);
        gizmos.line_2d(
            switch.position + Vec2::new(-5.0, 5.0),
            switch.position + Vec2::new(5.0, -5.0),
            color,
        );
    }
}

fn update_switches(
    mut rng: ResMut<SwitchRng>,
    network: Res<RailNetwork>,
    mut switches: Query<&mut RailSwitch>,
    mut trains: Query<&mut Locomotive>,
) {
    for mut switch in &mut switches {
        let mut approaching = false;
        for mut train in &mut trains {
            let Some(current_switch_route) = switch
                .routes
                .iter()
                .find(|route| route.route_index == train.route_index)
                .copied()
            else {
                continue;
            };
            let current_distance = network.routes[train.route_index].length * train.phase;
            let distance_ahead =
                (current_switch_route.distance - current_distance) * train.direction;
            if (0.0..=SWITCH_APPROACH_DISTANCE).contains(&distance_ahead) {
                approaching = true;
                if !switch.armed {
                    switch.active = next_switch_state(&mut rng);
                    switch.armed = true;
                    if switch.active {
                        let alternate = switch
                            .routes
                            .iter()
                            .copied()
                            .filter(|route| route.route_index != train.route_index)
                            .min_by(|left, right| {
                                let left_delta =
                                    (left.distance - current_switch_route.distance).abs();
                                let right_delta =
                                    (right.distance - current_switch_route.distance).abs();
                                left_delta
                                    .partial_cmp(&right_delta)
                                    .unwrap_or(Ordering::Equal)
                            });
                        if let Some(alternate) = alternate {
                            train.route_index = alternate.route_index;
                            train.phase = (alternate.distance
                                / network.routes[alternate.route_index].length)
                                .clamp(0.0, 1.0);
                        }
                    }
                }
                break;
            }
        }
        if !approaching {
            switch.armed = false;
        }
    }
}

fn next_switch_state(rng: &mut SwitchRng) -> bool {
    let mut value = rng.0;
    value ^= value << 13;
    value ^= value >> 7;
    value ^= value << 17;
    rng.0 = value;
    value & 1 == 1
}

fn animate_locomotives(
    time: Res<Time>,
    network: Res<RailNetwork>,
    mut trains: Query<(&mut Transform, &mut Locomotive)>,
) {
    for (mut transform, mut train) in &mut trains {
        train.phase += train.speed * time.delta_secs() * train.direction;
        if !(0.0..=1.0).contains(&train.phase) {
            train.route_index = (train.route_index + 1) % network.routes.len();
            train.direction *= -1.0;
            train.phase = if train.direction > 0.0 { 0.0 } else { 1.0 };
        }
        let route = &network.routes[train.route_index];
        let position = sample_polyline(&route.points, route.length * train.phase);
        transform.translation.x = position.x;
        transform.translation.y = position.y;
    }
}

fn draw_train_dots(
    mut gizmos: Gizmos,
    trains: Query<(&Transform, &Locomotive)>,
    cameras: Query<&OrthographicProjection, With<Camera>>,
) {
    let radius = 5.0 * cameras.single().scale;
    for (transform, train) in &trains {
        gizmos.circle_2d(
            transform.translation.truncate(),
            radius,
            route_color(train.route_index),
        );
    }
}

fn update_status(network: Res<RailNetwork>, mut query: Query<&mut Text, With<StatusText>>) {
    for mut text in &mut query {
        text.0 = format!(
            "{} locomotives · colored route dots · {} railway=rail ways · {} routes · {} RailSwitches · green=diverge, red=straight · +/- zoom · left/right pan",
            LOCOMOTIVE_COUNT,
            network.way_count,
            network.routes.len(),
            network.switch_count,
        );
    }
}

fn zoom_camera(
    time: Res<Time>,
    keys: Res<ButtonInput<KeyCode>>,
    mut scroll: EventReader<MouseWheel>,
    mut cameras: Query<(&mut OrthographicProjection, &mut Transform), With<Camera>>,
) {
    let mut factor: f32 = 1.0;
    if keys.just_pressed(KeyCode::Equal) {
        factor *= 0.8;
    }
    if keys.just_pressed(KeyCode::Minus) {
        factor *= 1.25;
    }
    for event in scroll.read() {
        factor *= (1.0_f32 - event.y * 0.08_f32).clamp(0.75_f32, 1.25_f32);
    }
    let horizontal = match (
        keys.pressed(KeyCode::ArrowLeft),
        keys.pressed(KeyCode::ArrowRight),
    ) {
        (true, false) => -1.0,
        (false, true) => 1.0,
        _ => 0.0,
    };
    for (mut camera, mut transform) in &mut cameras {
        if factor != 1.0 {
            camera.scale = (camera.scale * factor).clamp(MIN_CAMERA_SCALE, 8.0);
        }
        transform.translation.x += horizontal * CAMERA_PAN_SPEED * camera.scale * time.delta_secs();
    }
}
