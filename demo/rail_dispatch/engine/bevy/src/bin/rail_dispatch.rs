use std::f32::consts::{FRAC_PI_2, PI};
use std::fmt::Write as _;

use bevy::{gltf::GltfAssetLabel, prelude::*};
use serde::Deserialize;

const WINDOW_WIDTH: f32 = 1_280.0;
const WINDOW_HEIGHT: f32 = 820.0;
const TRACK_LEFT: f32 = -440.0;
const TRACK_RIGHT: f32 = 440.0;
const TRACK_TOP: f32 = 230.0;
const TRACK_BOTTOM: f32 = -230.0;
const CURVE_RADIUS: f32 = 120.0;
const SAMPLE_SPACING: f32 = 8.0;
const WORLD_SCALE: f32 = 0.12;
const CAR_SPACING_WORLD: f32 = 10.0;
const CAR_SPACING_DISTANCE: f32 = CAR_SPACING_WORLD / WORLD_SCALE;
const COLLISION_GAP_DISTANCE: f32 = 32.0;
const TRAIN_ACCELERATION: f32 = 18.0;
const TRAIN_DECELERATION: f32 = 24.0;
const CROSS_ROUTE_CLEARANCE_WORLD: f32 = 10.0;
const CONFIG_JSON: &str = include_str!("../../../../config/rail_dispatch.json");
const MODEL_ROOT: &str = "demo/rail_dispatch/assets/rail_stock/runtime";

#[derive(Resource, Deserialize, Clone, Copy)]
#[serde(deny_unknown_fields)]
struct RailDispatchConfig {
    schema_version: u16,
    signal_block_length: f32,
    signal_stop_distance: f32,
    green_delay_ms: u64,
}

impl RailDispatchConfig {
    fn load(raw: &str) -> Result<Self, String> {
        let config: Self =
            serde_json::from_str(raw).map_err(|error| format!("rail dispatch config: {error}"))?;
        if config.schema_version != 1 {
            return Err(format!(
                "unsupported rail dispatch config schema {}",
                config.schema_version
            ));
        }
        if config.signal_block_length <= config.signal_stop_distance {
            return Err("signal_block_length must exceed signal_stop_distance".into());
        }
        if config.green_delay_ms == 0 {
            return Err("green_delay_ms must be positive".into());
        }
        Ok(config)
    }

    fn green_delay_seconds(self) -> f32 {
        self.green_delay_ms as f32 / 1_000.0
    }
}

#[derive(Clone, Copy)]
struct PathPoint {
    position: Vec2,
    distance: f32,
}

#[derive(Resource)]
struct TrackPath {
    points: Vec<PathPoint>,
    total_length: f32,
}

#[derive(Resource)]
struct SidingPath {
    path: TrackPath,
    exit_distance: f32,
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum RouteKind {
    Main,
    Siding,
}

#[derive(Clone, Copy)]
enum CurveKind {
    NorthEast,
    SouthEast,
    SouthWest,
    NorthWest,
}

#[derive(Resource)]
struct TrainAssets {
    locomotive_front: Handle<Scene>,
    locomotive_wagon: Handle<Scene>,
    passenger_wagon: Handle<Scene>,
    munich_sbahn_front: Handle<Scene>,
    munich_sbahn_carriage: Handle<Scene>,
    cargo_front: Handle<Scene>,
    cargo_wagon: Handle<Scene>,
    cargo_container: Handle<Scene>,
}

#[derive(Component)]
struct Vehicle {
    id: u8,
    name: &'static str,
    route: RouteKind,
    distance: f32,
    speed: f32,
    current_speed: f32,
    length: f32,
    stopped: bool,
}

#[derive(Component)]
struct TrainCar {
    vehicle: Entity,
    route: RouteKind,
    offset: f32,
}

struct VehicleSnapshot {
    entity: Entity,
    route: RouteKind,
    distance: f32,
    current_speed: f32,
    length: f32,
}

#[derive(Component)]
struct SignalLamp {
    id: u8,
    name: &'static str,
    route: RouteKind,
    distance: f32,
    red: bool,
    manual_red: Option<bool>,
    green_delay_remaining: f32,
}

#[derive(Component)]
struct Telemetry;

#[derive(Component)]
struct DispatcherStatus;

#[derive(Resource)]
struct DispatcherState {
    selected_train: u8,
    selected_signal: u8,
    message: String,
}

impl Default for DispatcherState {
    fn default() -> Self {
        Self {
            selected_train: 1,
            selected_signal: 1,
            message: "Ready for dispatcher commands.".into(),
        }
    }
}

#[derive(Component)]
struct OrbitCamera {
    yaw: f32,
    pitch: f32,
    distance: f32,
}

fn main() {
    let path = TrackPath::new();
    let siding_path = SidingPath::new();
    let config = RailDispatchConfig::load(CONFIG_JSON).expect("rail dispatch config must validate");

    App::new()
        .add_plugins(
            DefaultPlugins
                .set(AssetPlugin {
                    file_path: "../../../..".into(),
                    ..default()
                })
                .set(WindowPlugin {
                    primary_window: Some(Window {
                        title: "Rail Dispatch — 3D railway control yard".into(),
                        resolution: (WINDOW_WIDTH, WINDOW_HEIGHT).into(),
                        ..default()
                    }),
                    ..default()
                }),
        )
        .insert_resource(ClearColor(Color::srgb(0.025, 0.04, 0.065)))
        .insert_resource(AmbientLight {
            color: Color::srgb(0.72, 0.78, 0.9),
            brightness: 500.0,
        })
        .insert_resource(path)
        .insert_resource(siding_path)
        .insert_resource(config)
        .insert_resource(DispatcherState::default())
        .add_systems(Startup, (load_assets, setup).chain())
        .add_systems(
            Update,
            (
                dispatcher_controls,
                camera_controls,
                update_signal_lamps,
                move_vehicles,
                update_telemetry,
                update_dispatcher_ui,
            )
                .chain(),
        )
        .run();
}

fn load_assets(asset_server: Res<AssetServer>, mut commands: Commands) {
    let scene = |file: &str| {
        asset_server.load(GltfAssetLabel::Scene(0).from_asset(format!("{MODEL_ROOT}/{file}")))
    };

    commands.insert_resource(TrainAssets {
        locomotive_front: scene("quaternius_modular_train/locomotive_front.glb"),
        locomotive_wagon: scene("quaternius_modular_train/locomotive_wagon.glb"),
        passenger_wagon: scene("quaternius_modular_train/locomotive_passenger_carriage.glb"),
        munich_sbahn_front: scene("munich_sbahn/munich_sbahn_front.glb"),
        munich_sbahn_carriage: scene("munich_sbahn/munich_sbahn_carriage.glb"),
        cargo_front: scene("quaternius_modular_train/cargo_train_front.glb"),
        cargo_wagon: scene("quaternius_modular_train/cargo_train_wagon.glb"),
        cargo_container: scene("quaternius_modular_train/cargo_train_container.glb"),
    });
}

fn setup(
    mut commands: Commands,
    path: Res<TrackPath>,
    siding_path: Res<SidingPath>,
    assets: Res<TrainAssets>,
    config: Res<RailDispatchConfig>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    commands.spawn((
        Camera3d::default(),
        Transform::from_xyz(0.0, 92.0, 118.0).looking_at(Vec3::new(0.0, 0.0, -2.0), Vec3::Y),
        OrbitCamera {
            yaw: 0.0,
            pitch: 0.66,
            distance: 150.0,
        },
    ));

    commands.spawn((
        DirectionalLight {
            illuminance: 12_000.0,
            shadows_enabled: true,
            ..default()
        },
        Transform::from_xyz(-40.0, 90.0, 50.0).looking_at(Vec3::ZERO, Vec3::Y),
    ));

    let ground_material = materials.add(StandardMaterial {
        base_color: Color::srgb(0.055, 0.11, 0.12),
        perceptual_roughness: 0.95,
        ..default()
    });
    commands.spawn((
        Mesh3d(meshes.add(Plane3d::default().mesh().size(150.0, 100.0))),
        MeshMaterial3d(ground_material),
    ));

    let track_bed = materials.add(StandardMaterial {
        base_color: Color::srgb(0.10, 0.12, 0.16),
        perceptual_roughness: 0.9,
        ..default()
    });
    let rail_material = materials.add(StandardMaterial {
        base_color: Color::srgb(0.55, 0.62, 0.68),
        metallic: 0.65,
        perceptual_roughness: 0.35,
        ..default()
    });
    let tie_material = materials.add(StandardMaterial {
        base_color: Color::srgb(0.30, 0.18, 0.11),
        perceptual_roughness: 0.95,
        ..default()
    });
    let cube = meshes.add(Cuboid::default());

    draw_track(
        &mut commands,
        &path,
        &cube,
        &track_bed,
        &rail_material,
        &tie_material,
    );
    draw_track(
        &mut commands,
        &siding_path.path,
        &cube,
        &track_bed,
        &rail_material,
        &tie_material,
    );

    spawn_station(&mut commands, &mut meshes, &mut materials);
    spawn_signals(&mut commands, &path, &mut meshes, &mut materials);
    spawn_siding_signal(&mut commands, &siding_path, &mut meshes, &mut materials);
    spawn_vehicles(&mut commands, &path, &siding_path, &assets);
    spawn_ui(&mut commands, *config);
}

fn draw_track(
    commands: &mut Commands,
    path: &TrackPath,
    cube: &Handle<Mesh>,
    bed_material: &Handle<StandardMaterial>,
    rail_material: &Handle<StandardMaterial>,
    tie_material: &Handle<StandardMaterial>,
) {
    for (index, point) in path.points.iter().enumerate() {
        let next = path.points[(index + 1) % path.points.len()];
        let delta = next.position - point.position;
        let direction = delta.normalize_or_zero();
        let normal = Vec2::new(-direction.y, direction.x);

        spawn_track_segment(
            commands,
            cube,
            bed_material,
            point.position,
            next.position,
            5.0,
            -1.45,
            1.0,
        );
        spawn_track_segment(
            commands,
            cube,
            rail_material,
            point.position + normal * 10.0,
            next.position + normal * 10.0,
            0.42,
            0.25,
            0.26,
        );
        spawn_track_segment(
            commands,
            cube,
            rail_material,
            point.position - normal * 10.0,
            next.position - normal * 10.0,
            0.42,
            0.25,
            0.26,
        );

        let midpoint = point.position.lerp(next.position, 0.5);
        spawn_tie(commands, cube, tie_material, midpoint, direction);
    }
}

fn spawn_track_segment(
    commands: &mut Commands,
    cube: &Handle<Mesh>,
    material: &Handle<StandardMaterial>,
    start: Vec2,
    end: Vec2,
    width: f32,
    height: f32,
    y: f32,
) {
    let delta = end - start;
    let length = delta.length() * WORLD_SCALE;
    if length <= f32::EPSILON {
        return;
    }
    let center = world_position(start.lerp(end, 0.5), y);
    let yaw = -(delta.y).atan2(delta.x);
    commands.spawn((
        Mesh3d(cube.clone()),
        MeshMaterial3d(material.clone()),
        Transform::from_translation(center)
            .with_rotation(Quat::from_rotation_y(yaw))
            .with_scale(Vec3::new(length, width * WORLD_SCALE, height)),
    ));
}

fn spawn_tie(
    commands: &mut Commands,
    cube: &Handle<Mesh>,
    material: &Handle<StandardMaterial>,
    position: Vec2,
    direction: Vec2,
) {
    let world_direction = Vec3::new(direction.x, 0.0, -direction.y);
    let yaw = -world_direction.z.atan2(world_direction.x);
    commands.spawn((
        Mesh3d(cube.clone()),
        MeshMaterial3d(material.clone()),
        Transform::from_translation(world_position(position, -0.55))
            .with_rotation(Quat::from_rotation_y(yaw))
            .with_scale(Vec3::new(1.0, 0.32, 5.0 * WORLD_SCALE)),
    ));
}

fn spawn_station(
    commands: &mut Commands,
    meshes: &mut Assets<Mesh>,
    materials: &mut Assets<StandardMaterial>,
) {
    let platform = materials.add(StandardMaterial {
        base_color: Color::srgb(0.38, 0.41, 0.44),
        perceptual_roughness: 0.8,
        ..default()
    });
    let station_wall = materials.add(StandardMaterial {
        base_color: Color::srgb(0.70, 0.31, 0.12),
        perceptual_roughness: 0.7,
        ..default()
    });
    let station_roof = materials.add(StandardMaterial {
        base_color: Color::srgb(0.08, 0.10, 0.14),
        perceptual_roughness: 0.5,
        ..default()
    });
    let glass = materials.add(StandardMaterial {
        base_color: Color::srgb(0.18, 0.55, 0.68),
        metallic: 0.2,
        perceptual_roughness: 0.2,
        ..default()
    });

    let platform_mesh = meshes.add(Cuboid::new(24.0, 0.9, 8.0));
    let wall_mesh = meshes.add(Cuboid::new(14.0, 6.0, 5.0));
    let roof_mesh = meshes.add(Cuboid::new(16.0, 0.8, 6.0));
    let window_mesh = meshes.add(Cuboid::new(3.0, 2.3, 0.25));
    let station_position = world_position(Vec2::new(0.0, 330.0), 0.0);

    commands.spawn((
        Mesh3d(platform_mesh),
        MeshMaterial3d(platform),
        Transform::from_translation(station_position),
    ));
    commands.spawn((
        Mesh3d(wall_mesh),
        MeshMaterial3d(station_wall),
        Transform::from_translation(station_position + Vec3::Y * 3.0),
    ));
    commands.spawn((
        Mesh3d(roof_mesh),
        MeshMaterial3d(station_roof),
        Transform::from_translation(station_position + Vec3::Y * 6.4),
    ));
    commands.spawn((
        Mesh3d(window_mesh.clone()),
        MeshMaterial3d(glass.clone()),
        Transform::from_translation(station_position + Vec3::new(-4.0, 3.2, -2.62)),
    ));
    commands.spawn((
        Mesh3d(window_mesh),
        MeshMaterial3d(glass),
        Transform::from_translation(station_position + Vec3::new(4.0, 3.2, -2.62)),
    ));
}

fn spawn_signals(
    commands: &mut Commands,
    path: &TrackPath,
    meshes: &mut Assets<Mesh>,
    materials: &mut Assets<StandardMaterial>,
) {
    for (id, label, distance) in [
        (1, "S1", 180.0),
        (2, "S2", 760.0),
        (3, "S3", 1_340.0),
        (4, "S4", 1_920.0),
    ] {
        spawn_signal(
            commands,
            path,
            meshes,
            materials,
            RouteKind::Main,
            distance,
            id,
            label,
        );
    }
}

fn spawn_siding_signal(
    commands: &mut Commands,
    siding_path: &SidingPath,
    meshes: &mut Assets<Mesh>,
    materials: &mut Assets<StandardMaterial>,
) {
    spawn_signal(
        commands,
        &siding_path.path,
        meshes,
        materials,
        RouteKind::Siding,
        siding_path.exit_distance,
        5,
        "S5",
    );
}

fn spawn_signal(
    commands: &mut Commands,
    path: &TrackPath,
    meshes: &mut Assets<Mesh>,
    materials: &mut Assets<StandardMaterial>,
    route: RouteKind,
    distance: f32,
    id: u8,
    name: &'static str,
) {
    let sample = path.sample_at(distance);
    let normal = Vec2::new(-sample.heading.sin(), sample.heading.cos());
    let signal_position = sample.position + normal * 42.0;
    let pole_material = materials.add(StandardMaterial {
        base_color: Color::srgb(0.12, 0.14, 0.17),
        metallic: 0.4,
        ..default()
    });
    let lamp_material = materials.add(StandardMaterial {
        base_color: Color::srgb(0.15, 0.95, 0.35),
        emissive: LinearRgba::new(0.05, 0.35, 0.08, 1.0),
        ..default()
    });

    commands.spawn((
        Mesh3d(meshes.add(Cylinder::new(0.22, 5.0))),
        MeshMaterial3d(pole_material),
        Transform::from_translation(world_position(signal_position, 2.0) + Vec3::Y * 2.5),
    ));
    commands.spawn((
        SignalLamp {
            id,
            name,
            route,
            distance,
            red: false,
            manual_red: None,
            green_delay_remaining: 0.0,
        },
        Mesh3d(meshes.add(Sphere::new(0.68).mesh().uv(16, 8))),
        MeshMaterial3d(lamp_material),
        Transform::from_translation(world_position(signal_position, 5.0) + Vec3::Y * 5.0),
    ));
}

fn spawn_vehicles(
    commands: &mut Commands,
    path: &TrackPath,
    siding_path: &SidingPath,
    assets: &TrainAssets,
) {
    spawn_vehicle(
        commands,
        path,
        1,
        "Express",
        RouteKind::Main,
        40.0,
        86.0,
        &[
            assets.locomotive_front.clone(),
            assets.locomotive_wagon.clone(),
            assets.passenger_wagon.clone(),
        ],
    );
    spawn_vehicle(
        commands,
        path,
        2,
        "S-Bahn",
        RouteKind::Main,
        path.total_length * 0.47,
        68.0,
        &[
            assets.munich_sbahn_front.clone(),
            assets.munich_sbahn_carriage.clone(),
        ],
    );
    spawn_vehicle(
        commands,
        &siding_path.path,
        3,
        "Cargo",
        RouteKind::Siding,
        siding_path.path.total_length * 0.04,
        78.0,
        &[
            assets.cargo_front.clone(),
            assets.cargo_wagon.clone(),
            assets.cargo_container.clone(),
        ],
    );
}

fn spawn_vehicle(
    commands: &mut Commands,
    path: &TrackPath,
    id: u8,
    name: &'static str,
    route: RouteKind,
    distance: f32,
    speed: f32,
    cars: &[Handle<Scene>],
) {
    let length = cars.len() as f32 * CAR_SPACING_DISTANCE;
    let vehicle = commands
        .spawn((
            Vehicle {
                id,
                name,
                route,
                distance,
                speed,
                current_speed: speed,
                length,
                stopped: false,
            },
            Transform::default(),
            Visibility::default(),
        ))
        .id();

    commands.entity(vehicle).with_children(|train| {
        for (index, car) in cars.iter().enumerate() {
            let offset = index as f32 * CAR_SPACING_DISTANCE;
            let sample = path.sample_at(distance - offset);
            train.spawn((
                TrainCar {
                    vehicle,
                    route,
                    offset,
                },
                SceneRoot(car.clone()),
                Transform::from_translation(world_position(sample.position, 1.8))
                    .with_rotation(vehicle_rotation(sample))
                    .with_scale(Vec3::splat(1.05)),
            ));
        }
    });
}

fn spawn_ui(commands: &mut Commands, config: RailDispatchConfig) {
    commands
        .spawn((
            Node {
                position_type: PositionType::Absolute,
                left: Val::Px(20.0),
                top: Val::Px(18.0),
                width: Val::Px(420.0),
                padding: UiRect::all(Val::Px(10.0)),
                flex_direction: FlexDirection::Column,
                row_gap: Val::Px(5.0),
                ..default()
            },
            BackgroundColor(Color::srgba(0.025, 0.04, 0.065, 0.90)),
        ))
        .with_children(|panel| {
            panel.spawn((
                Text::new("RAIL DISPATCH — PLAYER CONTROL\n"),
                TextFont {
                    font_size: 20.0,
                    ..default()
                },
                TextColor(Color::srgb(0.98, 0.82, 0.35)),
            ));
            panel.spawn((
                Text::new(format!(
                    "Select train: 1/2/3 or Tab\nSpeed: Q slower  |  E faster\nSelect signal: F1–F5\nSignal: S stop  |  G clear  |  A automatic\n\nSignals auto-protect occupied blocks.\nAutomatic release delay: {} ms.\nCamera: ARROWS orbit  |  +/- zoom  |  R reset",
                    config.green_delay_ms
                )),
                TextFont {
                    font_size: 13.0,
                    ..default()
                },
                TextColor(Color::srgb(0.78, 0.86, 0.94)),
            ));
            panel.spawn((
                DispatcherStatus,
                Text::new(""),
                TextFont {
                    font_size: 13.0,
                    ..default()
                },
                TextColor(Color::srgb(0.98, 0.82, 0.35)),
            ));
        });

    commands.spawn((
        Telemetry,
        Node {
            position_type: PositionType::Absolute,
            right: Val::Px(28.0),
            bottom: Val::Px(24.0),
            padding: UiRect::axes(Val::Px(14.0), Val::Px(8.0)),
            ..default()
        },
        BackgroundColor(Color::srgba(0.025, 0.04, 0.065, 0.90)),
        Text::new(""),
        TextFont {
            font_size: 16.0,
            ..default()
        },
        TextColor(Color::srgb(0.72, 0.92, 0.78)),
    ));
}

fn dispatcher_controls(
    keyboard: Res<ButtonInput<KeyCode>>,
    mut state: ResMut<DispatcherState>,
    mut vehicles: Query<&mut Vehicle>,
    mut lamps: Query<&mut SignalLamp>,
) {
    if keyboard.just_pressed(KeyCode::Digit1) {
        state.selected_train = 1;
        state.message = "Selected train 1.".into();
    } else if keyboard.just_pressed(KeyCode::Digit2) {
        state.selected_train = 2;
        state.message = "Selected train 2.".into();
    } else if keyboard.just_pressed(KeyCode::Digit3) {
        state.selected_train = 3;
        state.message = "Selected train 3.".into();
    } else if keyboard.just_pressed(KeyCode::Tab) {
        state.selected_train = state.selected_train % 3 + 1;
        state.message = format!("Selected train {}.", state.selected_train);
    }

    if keyboard.just_pressed(KeyCode::KeyQ) || keyboard.just_pressed(KeyCode::KeyE) {
        if let Some(mut vehicle) = vehicles
            .iter_mut()
            .find(|vehicle| vehicle.id == state.selected_train)
        {
            let delta = if keyboard.just_pressed(KeyCode::KeyE) {
                10.0
            } else {
                -10.0
            };
            vehicle.speed = (vehicle.speed + delta).clamp(0.0, 160.0);
            state.message = format!(
                "Train {} ({}) target speed set to {:.0}.",
                vehicle.id, vehicle.name, vehicle.speed
            );
        }
    }

    for (key, signal_id) in [
        (KeyCode::F1, 1),
        (KeyCode::F2, 2),
        (KeyCode::F3, 3),
        (KeyCode::F4, 4),
        (KeyCode::F5, 5),
    ] {
        if keyboard.just_pressed(key) {
            state.selected_signal = signal_id;
            state.message = format!("Selected signal S{signal_id}.");
        }
    }

    if keyboard.just_pressed(KeyCode::KeyS)
        || keyboard.just_pressed(KeyCode::KeyG)
        || keyboard.just_pressed(KeyCode::KeyA)
    {
        if let Some(mut lamp) = lamps
            .iter_mut()
            .find(|lamp| lamp.id == state.selected_signal)
        {
            let (manual_red, message) = if keyboard.just_pressed(KeyCode::KeyS) {
                (Some(true), format!("Signal {} set to STOP.", lamp.name))
            } else if keyboard.just_pressed(KeyCode::KeyG) {
                (Some(false), format!("Signal {} set to CLEAR.", lamp.name))
            } else {
                (None, format!("Signal {} returned to AUTOMATIC.", lamp.name))
            };
            lamp.manual_red = manual_red;
            lamp.green_delay_remaining = 0.0;
            lamp.red = manual_red.unwrap_or(lamp.red);
            state.message = message;
        }
    }
}

fn camera_controls(
    time: Res<Time>,
    keyboard: Res<ButtonInput<KeyCode>>,
    mut cameras: Query<(&mut OrbitCamera, &mut Transform)>,
) {
    let (mut orbit, mut transform) = cameras.single_mut();
    let dt = time.delta_secs();
    let rotation_speed = 0.9 * dt;
    if keyboard.pressed(KeyCode::ArrowLeft) {
        orbit.yaw += rotation_speed;
    }
    if keyboard.pressed(KeyCode::ArrowRight) {
        orbit.yaw -= rotation_speed;
    }
    if keyboard.pressed(KeyCode::ArrowUp) {
        orbit.pitch = (orbit.pitch + rotation_speed).clamp(0.35, 1.35);
    }
    if keyboard.pressed(KeyCode::ArrowDown) {
        orbit.pitch = (orbit.pitch - rotation_speed).clamp(0.35, 1.35);
    }
    if keyboard.pressed(KeyCode::Equal) || keyboard.pressed(KeyCode::NumpadAdd) {
        orbit.distance = (orbit.distance - 50.0 * dt).clamp(85.0, 230.0);
    }
    if keyboard.pressed(KeyCode::Minus) || keyboard.pressed(KeyCode::NumpadSubtract) {
        orbit.distance = (orbit.distance + 50.0 * dt).clamp(85.0, 230.0);
    }
    if keyboard.just_pressed(KeyCode::KeyR) {
        orbit.yaw = 0.0;
        orbit.pitch = 0.66;
        orbit.distance = 150.0;
    }

    let horizontal = orbit.distance * orbit.pitch.cos();
    transform.translation = Vec3::new(
        orbit.yaw.sin() * horizontal,
        orbit.pitch.sin() * orbit.distance,
        orbit.yaw.cos() * horizontal,
    );
    transform.look_at(Vec3::new(0.0, 0.0, -2.0), Vec3::Y);
}

fn move_vehicles(
    time: Res<Time>,
    path: Res<TrackPath>,
    siding_path: Res<SidingPath>,
    config: Res<RailDispatchConfig>,
    lamps: Query<&SignalLamp>,
    mut vehicles: Query<(Entity, &mut Vehicle)>,
    mut cars: Query<(&TrainCar, &mut Transform)>,
) {
    let advance_limit = time.delta_secs();
    let snapshots: Vec<VehicleSnapshot> = vehicles
        .iter()
        .map(|(entity, vehicle)| VehicleSnapshot {
            entity,
            route: vehicle.route,
            distance: vehicle.distance,
            current_speed: vehicle.current_speed,
            length: vehicle.length,
        })
        .collect();

    for (entity, mut vehicle) in &mut vehicles {
        let route_path = match vehicle.route {
            RouteKind::Main => &*path,
            RouteKind::Siding => &siding_path.path,
        };
        let current_distance = vehicle.distance;
        let mut speed_limit = vehicle.speed;

        for lamp in &lamps {
            if lamp.route != vehicle.route || !lamp.red {
                continue;
            }
            let distance_to_signal =
                (lamp.distance - current_distance).rem_euclid(route_path.total_length);
            let distance_to_stop = (distance_to_signal - config.signal_stop_distance).max(0.0);
            let braking_speed = (2.0 * TRAIN_DECELERATION * distance_to_stop).sqrt();
            speed_limit = speed_limit.min(braking_speed);
        }

        for other in &snapshots {
            if other.entity == entity || other.route != vehicle.route {
                continue;
            }
            let distance_ahead =
                (other.distance - current_distance).rem_euclid(route_path.total_length);
            if distance_ahead >= route_path.total_length / 2.0 {
                continue;
            }
            let safe_gap = other.length + COLLISION_GAP_DISTANCE;
            let available_distance = distance_ahead - safe_gap;
            let braking_speed = (2.0 * TRAIN_DECELERATION * available_distance.max(0.0)).sqrt();
            speed_limit = speed_limit.min(braking_speed);
        }

        let speed_change = (speed_limit - vehicle.current_speed).clamp(
            -TRAIN_DECELERATION * advance_limit,
            TRAIN_ACCELERATION * advance_limit,
        );
        vehicle.current_speed = (vehicle.current_speed + speed_change).max(0.0);
        let advance = vehicle.current_speed * advance_limit;
        let mut allowed_advance = advance;

        for lamp in &lamps {
            if lamp.route != vehicle.route || !lamp.red {
                continue;
            }
            let distance_to_signal =
                (lamp.distance - current_distance).rem_euclid(route_path.total_length);
            if distance_to_signal <= advance + config.signal_stop_distance {
                let distance_to_stop = (distance_to_signal - config.signal_stop_distance).max(0.0);
                allowed_advance = allowed_advance.min(distance_to_stop);
            }
        }

        for other in &snapshots {
            if other.entity == entity || other.route != vehicle.route {
                continue;
            }
            let distance_ahead =
                (other.distance - current_distance).rem_euclid(route_path.total_length);
            if distance_ahead >= route_path.total_length / 2.0 {
                continue;
            }
            let safe_gap = other.length + COLLISION_GAP_DISTANCE;
            let available_advance = (distance_ahead - safe_gap).max(0.0);
            allowed_advance = allowed_advance.min(available_advance);
        }

        let candidate = route_path.sample_at(current_distance + allowed_advance);
        let candidate_position = world_position(candidate.position, 0.0);
        let current_position = world_position(route_path.sample_at(current_distance).position, 0.0);
        for other in &snapshots {
            if other.entity == entity || other.route == vehicle.route {
                continue;
            }
            let other_path = match other.route {
                RouteKind::Main => &*path,
                RouteKind::Siding => &siding_path.path,
            };
            let other_current_position =
                world_position(other_path.sample_at(other.distance).position, 0.0);
            let other_candidate_position = world_position(
                other_path
                    .sample_at(other.distance + other.current_speed * advance_limit)
                    .position,
                0.0,
            );
            let clearance =
                CROSS_ROUTE_CLEARANCE_WORLD + (vehicle.length + other.length) * WORLD_SCALE * 0.5;
            let candidate_hits_other_current =
                candidate_position.distance(other_current_position) < clearance;
            let candidate_hits_other_candidate =
                candidate_position.distance(other_candidate_position) < clearance;
            let current_hits_other_candidate =
                current_position.distance(other_candidate_position) < clearance;
            let conflict = if vehicle.route == RouteKind::Siding {
                candidate_hits_other_current
                    || candidate_hits_other_candidate
                    || current_hits_other_candidate
            } else {
                candidate_hits_other_current
            };
            if conflict {
                allowed_advance = 0.0;
                break;
            }
        }

        if allowed_advance + f32::EPSILON < advance {
            vehicle.current_speed =
                (vehicle.current_speed - TRAIN_DECELERATION * advance_limit).max(0.0);
        }
        vehicle.stopped =
            allowed_advance + f32::EPSILON < advance || vehicle.current_speed <= f32::EPSILON;
        vehicle.distance = (current_distance + allowed_advance).rem_euclid(route_path.total_length);
    }

    for (car, mut transform) in &mut cars {
        let (_, vehicle) = vehicles
            .get(car.vehicle)
            .expect("every train car must have a vehicle");
        let route_path = match car.route {
            RouteKind::Main => &*path,
            RouteKind::Siding => &siding_path.path,
        };
        let sample = route_path.sample_at(vehicle.distance - car.offset);
        transform.translation = world_position(sample.position, 1.8);
        transform.rotation = vehicle_rotation(sample);
    }
}

fn update_signal_lamps(
    time: Res<Time>,
    path: Res<TrackPath>,
    siding_path: Res<SidingPath>,
    config: Res<RailDispatchConfig>,
    vehicles: Query<&Vehicle>,
    mut lamps: Query<(&mut SignalLamp, &MeshMaterial3d<StandardMaterial>)>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    for (mut lamp, material_handle) in &mut lamps {
        let route_path = match lamp.route {
            RouteKind::Main => &*path,
            RouteKind::Siding => &siding_path.path,
        };
        let occupied = vehicles.iter().any(|vehicle| {
            if vehicle.route != lamp.route {
                return false;
            }
            let distance_ahead =
                (vehicle.distance - lamp.distance).rem_euclid(route_path.total_length);
            distance_ahead > 0.0 && distance_ahead < config.signal_block_length + vehicle.length
        });
        if let Some(manual_red) = lamp.manual_red {
            lamp.green_delay_remaining = 0.0;
            lamp.red = manual_red;
        } else {
            if occupied {
                lamp.green_delay_remaining = config.green_delay_seconds();
            } else {
                lamp.green_delay_remaining =
                    (lamp.green_delay_remaining - time.delta_secs()).max(0.0);
            }
            lamp.red = occupied || lamp.green_delay_remaining > 0.0;
        }
        if let Some(material) = materials.get_mut(&material_handle.0) {
            material.base_color = if lamp.red {
                Color::srgb(0.98, 0.18, 0.14)
            } else {
                Color::srgb(0.15, 0.95, 0.35)
            };
            material.emissive = if lamp.red {
                LinearRgba::new(0.65, 0.04, 0.02, 1.0)
            } else {
                LinearRgba::new(0.05, 0.35, 0.08, 1.0)
            };
        }
    }
}

fn update_dispatcher_ui(
    state: Res<DispatcherState>,
    vehicles: Query<&Vehicle>,
    lamps: Query<&SignalLamp>,
    mut text: Query<&mut Text, With<DispatcherStatus>>,
) {
    let mut value = format!("{}\n\nTRAINS\n", state.message);
    for vehicle in &vehicles {
        let selected = if vehicle.id == state.selected_train {
            ">"
        } else {
            " "
        };
        let _ = writeln!(
            value,
            "{selected} {} {}  {:>3.0} speed  {}",
            vehicle.id,
            vehicle.name,
            vehicle.current_speed,
            if vehicle.stopped {
                "STOPPED"
            } else {
                "RUNNING"
            }
        );
    }

    value.push_str("\nSIGNALS\n");
    for lamp in &lamps {
        let selected = if lamp.id == state.selected_signal {
            ">"
        } else {
            " "
        };
        let control = match lamp.manual_red {
            Some(true) => "MANUAL STOP",
            Some(false) => "MANUAL CLEAR",
            None => "AUTO",
        };
        let aspect = if lamp.red { "RED" } else { "GREEN" };
        let _ = writeln!(value, "{selected} {}  {aspect:<5}  {control}", lamp.name);
    }

    *text.single_mut() = Text::new(value);
}

fn update_telemetry(
    path: Res<TrackPath>,
    siding_path: Res<SidingPath>,
    vehicles: Query<&Vehicle>,
    lamps: Query<&SignalLamp>,
    mut text: Query<&mut Text, With<Telemetry>>,
) {
    let vehicle_count = vehicles.iter().count();
    let red_signals = lamps.iter().filter(|lamp| lamp.red).count();
    let stopped = vehicles.iter().filter(|vehicle| vehicle.stopped).count();
    let average_speed = vehicles
        .iter()
        .map(|vehicle| vehicle.current_speed)
        .sum::<f32>()
        / vehicle_count.max(1) as f32;
    let laps = vehicles
        .iter()
        .map(|vehicle| {
            let route_length = match vehicle.route {
                RouteKind::Main => path.total_length,
                RouteKind::Siding => siding_path.path.total_length,
            };
            vehicle.distance / route_length
        })
        .sum::<f32>();
    let mut value = text.single_mut();
    *value = Text::new(format!(
        "{vehicle_count} TRAINS  |  {red_signals} RED SIGNALS  |  {stopped} STOPPED  |  AVG {average_speed:.0} px/s  |  LOOP {laps:.1}"
    ));
}

fn world_position(position: Vec2, y: f32) -> Vec3 {
    Vec3::new(position.x * WORLD_SCALE, y, -position.y * WORLD_SCALE)
}

fn vehicle_rotation(sample: Sample) -> Quat {
    let direction = Vec3::new(sample.heading.cos(), 0.0, -sample.heading.sin());
    Quat::from_rotation_y(-direction.z.atan2(direction.x))
}

impl SidingPath {
    fn new() -> Self {
        let left_join = Vec2::new(-180.0, TRACK_TOP);
        let right_join = Vec2::new(180.0, TRACK_TOP);
        let mut positions = Vec::new();

        push_point(&mut positions, left_join);
        push_line(&mut positions, left_join, Vec2::new(-120.0, 145.0));
        push_line(
            &mut positions,
            Vec2::new(-120.0, 145.0),
            Vec2::new(120.0, 145.0),
        );
        push_line(&mut positions, Vec2::new(120.0, 145.0), right_join);
        push_line(
            &mut positions,
            right_join,
            Vec2::new(TRACK_RIGHT - CURVE_RADIUS, TRACK_TOP),
        );
        push_curve(
            &mut positions,
            CurveKind::NorthEast,
            Vec2::new(TRACK_RIGHT - CURVE_RADIUS, TRACK_TOP - CURVE_RADIUS),
            CURVE_RADIUS,
        );
        push_line(
            &mut positions,
            Vec2::new(TRACK_RIGHT, TRACK_TOP - CURVE_RADIUS),
            Vec2::new(TRACK_RIGHT, TRACK_BOTTOM + CURVE_RADIUS),
        );
        push_curve(
            &mut positions,
            CurveKind::SouthEast,
            Vec2::new(TRACK_RIGHT - CURVE_RADIUS, TRACK_BOTTOM + CURVE_RADIUS),
            CURVE_RADIUS,
        );
        push_line(
            &mut positions,
            Vec2::new(TRACK_RIGHT - CURVE_RADIUS, TRACK_BOTTOM),
            Vec2::new(TRACK_LEFT + CURVE_RADIUS, TRACK_BOTTOM),
        );
        push_curve(
            &mut positions,
            CurveKind::SouthWest,
            Vec2::new(TRACK_LEFT + CURVE_RADIUS, TRACK_BOTTOM + CURVE_RADIUS),
            CURVE_RADIUS,
        );
        push_line(
            &mut positions,
            Vec2::new(TRACK_LEFT, TRACK_BOTTOM + CURVE_RADIUS),
            Vec2::new(TRACK_LEFT, TRACK_TOP - CURVE_RADIUS),
        );
        push_curve(
            &mut positions,
            CurveKind::NorthWest,
            Vec2::new(TRACK_LEFT + CURVE_RADIUS, TRACK_TOP - CURVE_RADIUS),
            CURVE_RADIUS,
        );
        push_line(
            &mut positions,
            Vec2::new(TRACK_LEFT + CURVE_RADIUS, TRACK_TOP),
            left_join,
        );

        let path = TrackPath::from_positions(positions);
        let exit_distance = path
            .points
            .iter()
            .find(|point| point.position.distance(right_join) <= f32::EPSILON)
            .expect("siding route must contain its exit")
            .distance;
        Self {
            path,
            exit_distance,
        }
    }
}

impl TrackPath {
    fn new() -> Self {
        let mut positions = Vec::new();
        let left = TRACK_LEFT;
        let right = TRACK_RIGHT;
        let top = TRACK_TOP;
        let bottom = TRACK_BOTTOM;
        let radius = CURVE_RADIUS;

        push_point(&mut positions, Vec2::new(left + radius, top));
        push_line(
            &mut positions,
            Vec2::new(left + radius, top),
            Vec2::new(right - radius, top),
        );
        push_curve(
            &mut positions,
            CurveKind::NorthEast,
            Vec2::new(right - radius, top - radius),
            radius,
        );
        push_line(
            &mut positions,
            Vec2::new(right, top - radius),
            Vec2::new(right, bottom + radius),
        );
        push_curve(
            &mut positions,
            CurveKind::SouthEast,
            Vec2::new(right - radius, bottom + radius),
            radius,
        );
        push_line(
            &mut positions,
            Vec2::new(right - radius, bottom),
            Vec2::new(left + radius, bottom),
        );
        push_curve(
            &mut positions,
            CurveKind::SouthWest,
            Vec2::new(left + radius, bottom + radius),
            radius,
        );
        push_line(
            &mut positions,
            Vec2::new(left, bottom + radius),
            Vec2::new(left, top - radius),
        );
        push_curve(
            &mut positions,
            CurveKind::NorthWest,
            Vec2::new(left + radius, top - radius),
            radius,
        );

        Self::from_positions(positions)
    }

    fn from_positions(positions: Vec<Vec2>) -> Self {
        let mut distance = 0.0;
        let mut points: Vec<PathPoint> = Vec::with_capacity(positions.len());
        for position in positions {
            if let Some(previous) = points.last() {
                distance += previous.position.distance(position);
            }
            points.push(PathPoint { position, distance });
        }
        let total_length = distance + points.last().unwrap().position.distance(points[0].position);
        Self {
            points,
            total_length,
        }
    }

    fn sample_at(&self, distance: f32) -> Sample {
        let distance = distance.rem_euclid(self.total_length);
        let index = self
            .points
            .iter()
            .rposition(|point| point.distance <= distance)
            .unwrap_or(0);
        let current = self.points[index];
        let next = self.points[(index + 1) % self.points.len()];
        let segment_length = if index + 1 == self.points.len() {
            self.total_length - current.distance
        } else {
            next.distance - current.distance
        };
        let ratio = if segment_length > 0.0 {
            (distance - current.distance) / segment_length
        } else {
            0.0
        };
        let position = current.position.lerp(next.position, ratio);
        Sample {
            position,
            heading: (next.position - current.position).to_angle(),
        }
    }
}

#[derive(Clone, Copy)]
struct Sample {
    position: Vec2,
    heading: f32,
}

fn push_point(points: &mut Vec<Vec2>, point: Vec2) {
    if points
        .last()
        .is_none_or(|last| last.distance(point) > f32::EPSILON)
    {
        points.push(point);
    }
}

fn push_line(points: &mut Vec<Vec2>, start: Vec2, end: Vec2) {
    let steps = (start.distance(end) / SAMPLE_SPACING).ceil() as usize;
    for step in 1..=steps.max(1) {
        push_point(points, start.lerp(end, step as f32 / steps.max(1) as f32));
    }
}

fn push_curve(points: &mut Vec<Vec2>, kind: CurveKind, center: Vec2, radius: f32) {
    let (start_angle, end_angle) = match kind {
        CurveKind::NorthEast => (FRAC_PI_2, 0.0),
        CurveKind::SouthEast => (0.0, -FRAC_PI_2),
        CurveKind::SouthWest => (-FRAC_PI_2, -PI),
        CurveKind::NorthWest => (PI, FRAC_PI_2),
    };
    let steps = ((radius * FRAC_PI_2) / SAMPLE_SPACING).ceil() as usize;
    for step in 1..=steps.max(1) {
        let ratio = step as f32 / steps.max(1) as f32;
        let angle = start_angle.lerp(end_angle, ratio);
        push_point(
            points,
            center + Vec2::new(angle.cos(), angle.sin()) * radius,
        );
    }
}
