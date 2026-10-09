use bevy::prelude::*;
use bevy::window::{PrimaryWindow, WindowResolution};
use std::f32::consts::TAU;

const SHIPS_PER_FACTION: usize = 10;
const METEORITE_COUNT: usize = 80;
const BATTLEFIELD: Vec3 = Vec3::new(1_400.0, 850.0, 1_100.0);
const MAX_SHIP_SPEED: f32 = 145.0;
const WEAPON_RANGE: f32 = 900.0;
const SHIP_RADIUS: f32 = 26.0;

fn main() {
    App::new()
        .insert_resource(ClearColor(Color::srgb(0.003, 0.006, 0.018)))
        .insert_resource(AmbientLight {
            color: Color::srgb(0.25, 0.32, 0.5),
            brightness: 180.0,
        })
        .insert_resource(BattleStats::default())
        .insert_resource(CameraRig::default())
        .insert_resource(SelectionState::default())
        .add_plugins(
            DefaultPlugins
                .set(AssetPlugin {
                    file_path: "../../assets".to_string(),
                    ..default()
                })
                .set(WindowPlugin {
                    primary_window: Some(Window {
                        title: "AIRTRAIN — Tactical Space Sandbox".to_string(),
                        resolution: WindowResolution::new(1600.0, 900.0),
                        ..default()
                    }),
                    ..default()
                }),
        )
        .add_systems(Startup, setup)
        .add_systems(
            Update,
            (
                camera_controls,
                select_closest_object,
                animate_battle_clock,
                drift_meteorites,
                ship_ai_and_fire,
                move_lasers_and_apply_hits,
                update_hud,
            ),
        )
        .run();
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Faction {
    Kingdom,
    Defender,
}

impl Faction {
    fn label(self) -> &'static str {
        match self {
            Self::Kingdom => "KINGDOM",
            Self::Defender => "DEFENDER",
        }
    }

    fn color(self) -> Color {
        match self {
            Self::Kingdom => Color::srgb(0.95, 0.18, 0.12),
            Self::Defender => Color::srgb(0.18, 0.65, 1.0),
        }
    }
}

#[derive(Component)]
struct Ship {
    faction: Faction,
    health: f32,
    velocity: Vec3,
    target: Option<Entity>,
    fire_cooldown: f32,
    phase: f32,
    best_shoot_score: f32,
}

#[derive(Component)]
struct Meteorite {
    radius: f32,
    velocity: Vec3,
}

#[derive(Component)]
struct LaserBolt {
    faction: Faction,
    target: Entity,
    velocity: Vec3,
    damage: f32,
    ttl: f32,
}

#[derive(Component)]
struct HudText;

#[derive(Resource, Default)]
struct BattleStats {
    elapsed: f32,
    shots_fired: u32,
    hits: u32,
}

#[derive(Resource)]
struct SelectionState {
    message: String,
}

impl Default for SelectionState {
    fn default() -> Self {
        Self {
            message: "Click a fighter or meteorite for object info.".to_string(),
        }
    }
}

#[derive(Resource)]
struct CameraRig {
    yaw: f32,
    pitch: f32,
    distance: f32,
    hud_visible: bool,
}

impl Default for CameraRig {
    fn default() -> Self {
        Self {
            yaw: 0.0,
            pitch: 0.275,
            distance: 2_850.0,
            hud_visible: true,
        }
    }
}

#[derive(Resource)]
struct CombatAssets {
    bolt_mesh: Handle<Mesh>,
    kingdom_material: Handle<StandardMaterial>,
    defender_material: Handle<StandardMaterial>,
}

#[derive(Clone, Copy)]
struct ShipSnapshot {
    entity: Entity,
    faction: Faction,
    position: Vec3,
    health: f32,
}

#[derive(Clone, Copy)]
struct MeteorSnapshot {
    position: Vec3,
    radius: f32,
}

struct Lcg(u64);

impl Lcg {
    fn new(seed: u64) -> Self {
        Self(seed)
    }

    fn next(&mut self) -> f32 {
        self.0 = self
            .0
            .wrapping_mul(6_364_136_223_846_793_005)
            .wrapping_add(1_442_695_040_888_963_407);
        ((self.0 >> 32) as u32 as f32) / u32::MAX as f32
    }

    fn range(&mut self, low: f32, high: f32) -> f32 {
        low + (high - low) * self.next()
    }
}

fn camera_controls(
    time: Res<Time>,
    keys: Res<ButtonInput<KeyCode>>,
    mut rig: ResMut<CameraRig>,
    mut camera: Query<&mut Transform, With<Camera3d>>,
    mut hud: Query<&mut Visibility, With<HudText>>,
) {
    if keys.just_pressed(KeyCode::F1) {
        rig.hud_visible = !rig.hud_visible;
        let visibility = if rig.hud_visible {
            Visibility::Visible
        } else {
            Visibility::Hidden
        };
        for mut hud_visibility in &mut hud {
            *hud_visibility = visibility;
        }
    }

    let delta = time.delta_secs();
    if keys.pressed(KeyCode::ArrowLeft) {
        rig.yaw += delta * 0.75;
    }
    if keys.pressed(KeyCode::ArrowRight) {
        rig.yaw -= delta * 0.75;
    }
    if keys.pressed(KeyCode::ArrowUp) {
        rig.pitch = (rig.pitch + delta * 0.55).clamp(-0.25, 1.05);
    }
    if keys.pressed(KeyCode::ArrowDown) {
        rig.pitch = (rig.pitch - delta * 0.55).clamp(-0.25, 1.05);
    }
    if keys.pressed(KeyCode::Equal) || keys.pressed(KeyCode::NumpadAdd) {
        rig.distance -= delta * 900.0;
    }
    if keys.pressed(KeyCode::Minus) || keys.pressed(KeyCode::NumpadSubtract) {
        rig.distance += delta * 900.0;
    }

    let horizontal = rig.distance * rig.pitch.cos();
    let position = Vec3::new(
        rig.yaw.sin() * horizontal,
        rig.pitch.sin() * rig.distance,
        rig.yaw.cos() * horizontal,
    );
    for mut transform in &mut camera {
        *transform = Transform::from_translation(position).looking_at(Vec3::ZERO, Vec3::Y);
    }
}

fn select_closest_object(
    mouse: Res<ButtonInput<MouseButton>>,
    windows: Query<&Window, With<PrimaryWindow>>,
    cameras: Query<(&Camera, &GlobalTransform), With<Camera3d>>,
    ships: Query<(&Ship, &GlobalTransform)>,
    meteorites: Query<(&Meteorite, &GlobalTransform)>,
    mut selection: ResMut<SelectionState>,
) {
    if !mouse.just_pressed(MouseButton::Left) {
        return;
    }

    let Ok(window) = windows.get_single() else {
        return;
    };
    let Some(cursor) = window.cursor_position() else {
        selection.message = "CLICK: cursor is outside the game window.".to_string();
        return;
    };
    let Ok((camera, camera_transform)) = cameras.get_single() else {
        return;
    };
    let Ok(ray) = camera.viewport_to_world(camera_transform, cursor) else {
        return;
    };

    let origin = ray.origin;
    let direction = *ray.direction;
    let mut closest: Option<(f32, String)> = None;

    for (ship, transform) in &ships {
        if let Some(distance) =
            ray_sphere_distance(origin, direction, transform.translation(), 48.0)
        {
            let info = format!(
                "SELECTED {}  |  health {:03.0}  |  distance {:.0}m  |  shooting score {:.2}",
                ship.faction.label(),
                ship.health.max(0.0),
                origin.distance(transform.translation()),
                ship.best_shoot_score,
            );
            consider_selection(&mut closest, distance, info);
        }
    }

    for (meteorite, transform) in &meteorites {
        if let Some(distance) = ray_sphere_distance(
            origin,
            direction,
            transform.translation(),
            meteorite.radius + 8.0,
        ) {
            let info = format!(
                "SELECTED METEORITE  |  radius {:.0}m  |  distance {:.0}m  |  drift {:.1}m/s",
                meteorite.radius,
                origin.distance(transform.translation()),
                meteorite.velocity.length(),
            );
            consider_selection(&mut closest, distance, info);
        }
    }

    selection.message = closest
        .map(|(_, info)| info)
        .unwrap_or_else(|| "CLICK: no fighter or meteorite under the cursor.".to_string());
}

fn consider_selection(closest: &mut Option<(f32, String)>, distance: f32, info: String) {
    if closest
        .as_ref()
        .is_none_or(|(closest_distance, _)| distance < *closest_distance)
    {
        *closest = Some((distance, info));
    }
}

fn ray_sphere_distance(origin: Vec3, direction: Vec3, center: Vec3, radius: f32) -> Option<f32> {
    let to_center = center - origin;
    let along_ray = to_center.dot(direction);
    if along_ray < 0.0 {
        return None;
    }
    let perpendicular_squared = to_center.length_squared() - along_ray * along_ray;
    let radius_squared = radius * radius;
    if perpendicular_squared > radius_squared {
        return None;
    }
    Some((along_ray - (radius_squared - perpendicular_squared).sqrt()).max(0.0))
}

fn setup(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    commands.spawn((
        Camera3d::default(),
        Msaa::Sample4,
        Transform::from_xyz(0.0, 780.0, 2_750.0).looking_at(Vec3::ZERO, Vec3::Y),
    ));

    commands.spawn((
        DirectionalLight {
            illuminance: 12_000.0,
            shadows_enabled: true,
            ..default()
        },
        Transform::from_xyz(-600.0, 1_200.0, 700.0).looking_at(Vec3::ZERO, Vec3::Y),
    ));
    commands.spawn((
        PointLight {
            intensity: 2_000_000.0,
            range: 5_000.0,
            color: Color::srgb(0.35, 0.48, 1.0),
            ..default()
        },
        Transform::from_xyz(0.0, 700.0, 300.0),
    ));

    let meteor_mesh = meshes.add(Sphere::new(1.0).mesh().ico(4).unwrap());
    let mut rng = Lcg::new(0x5EED_2026);
    for _ in 0..METEORITE_COUNT {
        let radius = rng.range(18.0, 78.0);
        let position = Vec3::new(
            rng.range(-BATTLEFIELD.x, BATTLEFIELD.x),
            rng.range(-BATTLEFIELD.y, BATTLEFIELD.y),
            rng.range(-BATTLEFIELD.z, BATTLEFIELD.z),
        );
        let material = materials.add(StandardMaterial {
            base_color: Color::srgb(
                rng.range(0.12, 0.28),
                rng.range(0.14, 0.31),
                rng.range(0.18, 0.36),
            ),
            perceptual_roughness: 0.92,
            metallic: 0.18,
            ..default()
        });
        commands.spawn((
            Meteorite {
                radius,
                velocity: Vec3::new(
                    rng.range(-7.0, 7.0),
                    rng.range(-4.0, 4.0),
                    rng.range(-7.0, 7.0),
                ),
            },
            Mesh3d(meteor_mesh.clone()),
            MeshMaterial3d(material),
            Transform::from_translation(position).with_scale(Vec3::splat(radius)),
        ));
    }

    let tie_handle: Handle<Scene> = asset_server.load("side_fighter.glb#Scene0");
    let xwing_handle: Handle<Scene> = asset_server.load("winger.glb#Scene0");

    for index in 0..SHIPS_PER_FACTION {
        spawn_ship(
            &mut commands,
            Faction::Kingdom,
            index,
            formation_position(Faction::Kingdom, index),
            tie_handle.clone(),
            0.92,
        );
        spawn_ship(
            &mut commands,
            Faction::Defender,
            index,
            formation_position(Faction::Defender, index),
            xwing_handle.clone(),
            1.7,
        );
    }

    let bolt_mesh = meshes.add(Sphere::new(1.0).mesh().uv(12, 8));
    let kingdom_material = materials.add(StandardMaterial {
        base_color: Faction::Kingdom.color(),
        emissive: LinearRgba::new(1.0, 0.04, 0.01, 1.0),
        ..default()
    });
    let defender_material = materials.add(StandardMaterial {
        base_color: Faction::Defender.color(),
        emissive: LinearRgba::new(0.02, 0.35, 1.0, 1.0),
        ..default()
    });
    commands.insert_resource(CombatAssets {
        bolt_mesh,
        kingdom_material,
        defender_material,
    });

    commands.spawn((
        Text::new("Loading tactical space..."),
        Node {
            position_type: PositionType::Absolute,
            left: Val::Px(24.0),
            top: Val::Px(20.0),
            ..default()
        },
        TextFont {
            font_size: 19.0,
            ..default()
        },
        TextColor(Color::srgb(0.82, 0.9, 1.0)),
        HudText,
    ));
}

fn formation_position(faction: Faction, index: usize) -> Vec3 {
    let lane = (index % 5) as f32 - 2.0;
    let echelon = (index / 5) as f32 - 0.5;
    let side = match faction {
        Faction::Kingdom => -1.0,
        Faction::Defender => 1.0,
    };

    Vec3::new(
        side * (930.0 + echelon * 300.0),
        lane * 210.0 + echelon * 45.0,
        echelon * 560.0,
    )
}

fn spawn_ship(
    commands: &mut Commands,
    faction: Faction,
    index: usize,
    position: Vec3,
    scene: Handle<Scene>,
    scale: f32,
) {
    commands.spawn((
        Ship {
            faction,
            health: 100.0,
            velocity: Vec3::ZERO,
            target: None,
            fire_cooldown: 0.5 + index as f32 * 0.06,
            phase: index as f32 * TAU / SHIPS_PER_FACTION as f32,
            best_shoot_score: 0.0,
        },
        SceneRoot(scene),
        Transform::from_translation(position).with_scale(Vec3::splat(scale)),
        Name::new(format!("{} ship {:02}", faction.label(), index + 1)),
    ));
}

fn animate_battle_clock(time: Res<Time>, mut stats: ResMut<BattleStats>) {
    stats.elapsed += time.delta_secs();
}

fn drift_meteorites(time: Res<Time>, mut meteorites: Query<(&Meteorite, &mut Transform)>) {
    let delta = time.delta_secs();
    for (meteorite, mut transform) in &mut meteorites {
        transform.translation += meteorite.velocity * delta;
        for axis in 0..3 {
            if transform.translation[axis] > BATTLEFIELD[axis] {
                transform.translation[axis] = -BATTLEFIELD[axis];
            } else if transform.translation[axis] < -BATTLEFIELD[axis] {
                transform.translation[axis] = BATTLEFIELD[axis];
            }
        }
        transform.rotate(Quat::from_rotation_y(delta * 0.08));
    }
}

fn ship_ai_and_fire(
    time: Res<Time>,
    mut commands: Commands,
    assets: Res<CombatAssets>,
    mut stats: ResMut<BattleStats>,
    mut ships: Query<(Entity, &mut Ship, &mut Transform), Without<Meteorite>>,
    meteorites: Query<(&Meteorite, &Transform)>,
) {
    let delta = time.delta_secs();
    let snapshots: Vec<ShipSnapshot> = ships
        .iter()
        .map(|(entity, ship, transform)| ShipSnapshot {
            entity,
            faction: ship.faction,
            position: transform.translation,
            health: ship.health,
        })
        .collect();
    let meteors: Vec<MeteorSnapshot> = meteorites
        .iter()
        .map(|(meteorite, transform)| MeteorSnapshot {
            position: transform.translation,
            radius: meteorite.radius,
        })
        .collect();

    for (entity, mut ship, mut transform) in &mut ships {
        ship.fire_cooldown -= delta;
        let Some(target) = snapshots
            .iter()
            .filter(|candidate| candidate.faction != ship.faction && candidate.entity != entity)
            .min_by(|a, b| {
                transform
                    .translation
                    .distance_squared(a.position)
                    .partial_cmp(&transform.translation.distance_squared(b.position))
                    .unwrap_or(std::cmp::Ordering::Equal)
            })
            .copied()
        else {
            continue;
        };
        ship.target = Some(target.entity);

        let to_target = (target.position - transform.translation).normalize_or_zero();
        let side = Vec3::new(-to_target.z, 0.0, to_target.x).normalize_or_zero();
        let phase = ship.phase + stats.elapsed * 0.4;
        let mut desired = target.position - to_target * 360.0
            + side * phase.sin() * 180.0
            + Vec3::Y * phase.cos() * 130.0;

        let blocked = line_blocked(transform.translation, target.position, &meteors);
        if blocked {
            desired += side * 260.0;
        }
        for meteor in &meteors {
            let away = desired - meteor.position;
            let minimum = meteor.radius + 90.0;
            if away.length_squared() < minimum * minimum {
                desired += away.normalize_or_zero() * (minimum - away.length()).max(0.0);
            }
        }
        desired.x = desired.x.clamp(-BATTLEFIELD.x, BATTLEFIELD.x);
        desired.y = desired.y.clamp(-BATTLEFIELD.y, BATTLEFIELD.y);
        desired.z = desired.z.clamp(-BATTLEFIELD.z, BATTLEFIELD.z);

        let distance = transform.translation.distance(target.position);
        let range_score = 1.0 - ((distance - 460.0).abs() / 460.0).min(1.0);
        let forward_score = to_target.dot(*transform.forward()).abs();
        let cover_score = if blocked { 0.8 } else { 0.2 };
        ship.best_shoot_score = 0.30 * range_score
            + 0.25 * forward_score
            + 0.20 * (1.0 - if blocked { 1.0 } else { 0.0 })
            + 0.15 * cover_score
            + 0.10 * (target.health / 100.0);

        let desired_velocity =
            (desired - transform.translation).normalize_or_zero() * MAX_SHIP_SPEED;
        ship.velocity = ship.velocity.lerp(desired_velocity, (delta * 0.8).min(1.0));
        transform.translation += ship.velocity * delta;
        if ship.velocity.length_squared() > 1.0 {
            let look_target = transform.translation + ship.velocity;
            transform.look_at(look_target, Vec3::Y);
        }

        if ship.fire_cooldown <= 0.0
            && distance <= WEAPON_RANGE
            && !blocked
            && ship.best_shoot_score > 0.32
        {
            let bolt_velocity = to_target * 620.0;
            let material = match ship.faction {
                Faction::Kingdom => assets.kingdom_material.clone(),
                Faction::Defender => assets.defender_material.clone(),
            };
            commands.spawn((
                LaserBolt {
                    faction: ship.faction,
                    target: target.entity,
                    velocity: bolt_velocity,
                    damage: 12.0,
                    ttl: 2.0,
                },
                Mesh3d(assets.bolt_mesh.clone()),
                MeshMaterial3d(material),
                Transform::from_translation(transform.translation).with_scale(Vec3::splat(7.0)),
            ));
            ship.fire_cooldown = 0.48;
            stats.shots_fired += 1;
        }
    }
}

fn move_lasers_and_apply_hits(
    time: Res<Time>,
    mut commands: Commands,
    mut stats: ResMut<BattleStats>,
    mut lasers: Query<(Entity, &mut LaserBolt, &mut Transform)>,
    mut ships: Query<(Entity, &mut Ship, &Transform), Without<LaserBolt>>,
) {
    let delta = time.delta_secs();
    let ship_positions: Vec<ShipSnapshot> = ships
        .iter()
        .map(|(entity, ship, transform)| ShipSnapshot {
            entity,
            faction: ship.faction,
            position: transform.translation,
            health: ship.health,
        })
        .collect();
    let mut hits = Vec::new();

    for (entity, mut laser, mut transform) in &mut lasers {
        laser.ttl -= delta;
        transform.translation += laser.velocity * delta;
        if laser.ttl <= 0.0 {
            commands.entity(entity).despawn();
            continue;
        }
        if let Some(target) = ship_positions.iter().find(|candidate| {
            candidate.entity == laser.target && candidate.faction != laser.faction
        }) {
            if transform.translation.distance(target.position) < 38.0 {
                hits.push((entity, target.entity, laser.damage));
            }
        }
    }

    for (laser_entity, target_entity, damage) in hits {
        commands.entity(laser_entity).despawn();
        if let Ok((_, mut ship, _)) = ships.get_mut(target_entity) {
            ship.health -= damage;
            stats.hits += 1;
            if ship.health <= 0.0 {
                commands.entity(target_entity).despawn();
            }
        }
    }
}

fn update_hud(
    stats: Res<BattleStats>,
    ships: Query<&Ship>,
    meteorites: Query<&Meteorite>,
    selection: Res<SelectionState>,
    mut hud: Query<&mut Text, With<HudText>>,
) {
    let mut kingdom = 0;
    let mut defender = 0;
    let mut best_score: f32 = 0.0;
    for ship in &ships {
        match ship.faction {
            Faction::Kingdom => kingdom += 1,
            Faction::Defender => defender += 1,
        }
        best_score = best_score.max(ship.best_shoot_score);
    }
    for mut text in &mut hud {
        *text = Text::new(format!(
            "AIRTRAIN // TACTICAL SPACE SANDBOX\n\
             {:02}:{:02}  |  KINGDOM {:02}  |  DEFENDER {:02}\n\
             meteorites {:02}  |  shots {:04}  |  hits {:04}  |  best shooting-position score {:.2}\n\
             {}\n\
             AI: target aspect + range + forward arc + line-of-fire + meteorite cover",
            (stats.elapsed as u32) / 60,
            (stats.elapsed as u32) % 60,
            kingdom,
            defender,
            meteorites.iter().count(),
            stats.shots_fired,
            stats.hits,
            best_score,
            selection.message,
        ));
    }
}

fn line_blocked(start: Vec3, end: Vec3, meteorites: &[MeteorSnapshot]) -> bool {
    let segment = end - start;
    let length_squared = segment.length_squared();
    if length_squared <= f32::EPSILON {
        return false;
    }
    meteorites.iter().any(|meteor| {
        let t = ((meteor.position - start).dot(segment) / length_squared).clamp(0.0, 1.0);
        let closest = start + segment * t;
        closest.distance(meteor.position) < meteor.radius + SHIP_RADIUS
    })
}
