use std::f32::consts::TAU;

use bevy::prelude::*;
use bevy::sprite::{ColorMaterial, MeshMaterial2d};
const WINDOW_WIDTH: f32 = 1_280.0;
const WINDOW_HEIGHT: f32 = 800.0;
const GROUND_Y: f32 = -330.0;
const GRAVITY: f32 = 150.0;

#[derive(Clone, Copy, PartialEq, Eq)]
enum SpeedProfile {
    Slow,
    Classic,
    Fast,
}

impl SpeedProfile {
    fn label(self) -> &'static str {
        match self {
            Self::Slow => "Slow",
            Self::Classic => "Classic",
            Self::Fast => "Fast",
        }
    }

    fn launch_speed(self) -> f32 {
        match self {
            Self::Slow => 330.0,
            Self::Classic => 430.0,
            Self::Fast => 570.0,
        }
    }

    fn auto_interval(self) -> f32 {
        match self {
            Self::Slow => 1.55,
            Self::Classic => 1.15,
            Self::Fast => 0.85,
        }
    }
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum Palette {
    Aurora,
    Sunset,
    Electric,
    Candy,
}

impl Palette {
    fn label(self) -> &'static str {
        match self {
            Self::Aurora => "Aurora",
            Self::Sunset => "Sunset",
            Self::Electric => "Electric",
            Self::Candy => "Candy",
        }
    }
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum ExplosionType {
    Burst,
    Ring,
    Willow,
}

impl ExplosionType {
    fn label(self) -> &'static str {
        match self {
            Self::Burst => "Burst",
            Self::Ring => "Ring",
            Self::Willow => "Willow",
        }
    }
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum RocketType {
    Classic,
    Comet,
    Spinner,
    Heavy,
}

impl RocketType {
    fn label(self) -> &'static str {
        match self {
            Self::Classic => "Classic",
            Self::Comet => "Comet",
            Self::Spinner => "Spinner",
            Self::Heavy => "Heavy",
        }
    }

    fn speed_factor(self) -> f32 {
        match self {
            Self::Classic => 1.0,
            Self::Comet => 1.08,
            Self::Spinner => 0.94,
            Self::Heavy => 0.78,
        }
    }

    fn horizontal_factor(self) -> f32 {
        match self {
            Self::Classic => 1.0,
            Self::Comet => 1.8,
            Self::Spinner => 1.35,
            Self::Heavy => 0.45,
        }
    }

    fn trail_interval(self) -> f32 {
        match self {
            Self::Classic => 0.035,
            Self::Comet => 0.022,
            Self::Spinner => 0.028,
            Self::Heavy => 0.052,
        }
    }

    fn trail_size(self) -> (f32, f32) {
        match self {
            Self::Classic => (2.0, 4.0),
            Self::Comet => (3.0, 6.0),
            Self::Spinner => (2.0, 5.0),
            Self::Heavy => (4.0, 8.0),
        }
    }

    fn scale(self) -> f32 {
        match self {
            Self::Classic => 1.0,
            Self::Comet => 1.25,
            Self::Spinner => 1.1,
            Self::Heavy => 1.65,
        }
    }
}

#[derive(Resource)]
struct ShowState {
    speed: SpeedProfile,
    palette: Palette,
    explosion: ExplosionType,
    rocket_type: RocketType,
    auto_show: bool,
    auto_timer: f32,
    launches: u32,
    explosions: u32,
}

#[derive(Resource)]
struct RandomState {
    state: u64,
}

#[derive(Resource)]
struct RocketMesh(Handle<Mesh>);

impl RandomState {
    fn next_u32(&mut self) -> u32 {
        self.state = self
            .state
            .wrapping_mul(6_364_136_223_846_793_005)
            .wrapping_add(1_442_695_040_888_963_407);
        (self.state >> 32) as u32
    }

    fn unit(&mut self) -> f32 {
        self.next_u32() as f32 / u32::MAX as f32
    }

    fn range(&mut self, min: f32, max: f32) -> f32 {
        min + (max - min) * self.unit()
    }

    fn index(&mut self, count: usize) -> usize {
        (self.next_u32() as usize) % count
    }
}

#[derive(Component)]
struct Rocket {
    velocity: Vec2,
    base_horizontal: f32,
    target_height: f32,
    trail_timer: f32,
    age: f32,
    rocket_type: RocketType,
    color: Color,
}

#[derive(Component)]
struct Particle {
    velocity: Vec2,
    remaining: f32,
    lifetime: f32,
    gravity: f32,
    drag: f32,
}

#[derive(Component)]
struct HudText;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                title: "Fireworks — color, speed, and explosion lab".into(),
                resolution: (WINDOW_WIDTH, WINDOW_HEIGHT).into(),
                ..default()
            }),
            ..default()
        }))
        .insert_resource(ShowState {
            speed: SpeedProfile::Classic,
            palette: Palette::Aurora,
            explosion: ExplosionType::Burst,
            rocket_type: RocketType::Classic,
            auto_show: true,
            auto_timer: 0.9,
            launches: 0,
            explosions: 0,
        })
        .insert_resource(RandomState {
            state: 0x5EED_F00D_CAFE_BEEF,
        })
        .add_systems(Startup, setup)
        .add_systems(
            Update,
            (
                handle_controls,
                update_rockets,
                update_particles,
                update_hud,
            )
                .chain(),
        )
        .run();
}

fn setup(
    mut commands: Commands,
    mut random: ResMut<RandomState>,
    mut meshes: ResMut<Assets<Mesh>>,
) {
    commands.insert_resource(RocketMesh(meshes.add(Circle::new(4.5))));
    commands.spawn(Camera2d);
    commands.spawn((
        Sprite::from_color(
            Color::srgb(0.008, 0.014, 0.045),
            Vec2::new(WINDOW_WIDTH + 80.0, WINDOW_HEIGHT + 80.0),
        ),
        Transform::from_translation(Vec3::new(0.0, 0.0, -20.0)),
    ));

    for _ in 0..105 {
        let x = random.range(-WINDOW_WIDTH * 0.5, WINDOW_WIDTH * 0.5);
        let y = random.range(-35.0, WINDOW_HEIGHT * 0.5 - 20.0);
        let size = random.range(1.0, 3.0);
        let brightness = random.range(0.35, 0.9);
        commands.spawn((
            Sprite::from_color(
                Color::srgb(brightness * 0.55, brightness * 0.7, brightness),
                Vec2::splat(size),
            ),
            Transform::from_translation(Vec3::new(x, y, -10.0)),
        ));
    }

    commands.spawn((
        Sprite::from_color(
            Color::srgb(0.025, 0.04, 0.09),
            Vec2::new(WINDOW_WIDTH + 80.0, 115.0),
        ),
        Transform::from_translation(Vec3::new(0.0, GROUND_Y - 18.0, -4.0)),
    ));

    for index in 0..24 {
        let width = random.range(38.0, 72.0);
        let height = random.range(48.0, 155.0);
        let x = -WINDOW_WIDTH * 0.5 + 25.0 + index as f32 * 57.0;
        let y = GROUND_Y + height * 0.5 - 5.0;
        commands.spawn((
            Sprite::from_color(Color::srgb(0.035, 0.06, 0.12), Vec2::new(width, height)),
            Transform::from_translation(Vec3::new(x, y, -2.0)),
        ));

        if height > 75.0 {
            for row in 0..2 {
                commands.spawn((
                    Sprite::from_color(Color::srgb(0.75, 0.48, 0.18), Vec2::new(3.0, 5.0)),
                    Transform::from_translation(Vec3::new(
                        x - width * 0.2 + row as f32 * width * 0.4,
                        y + height * 0.2,
                        -1.0,
                    )),
                ));
            }
        }
    }

    for x in [-430.0, -145.0, 145.0, 430.0] {
        commands.spawn((
            Sprite::from_color(Color::srgb(0.12, 0.15, 0.24), Vec2::new(100.0, 12.0)),
            Transform::from_translation(Vec3::new(x, GROUND_Y + 6.0, 2.0)),
        ));
        commands.spawn((
            Sprite::from_color(Color::srgb(0.23, 0.27, 0.40), Vec2::new(12.0, 34.0)),
            Transform::from_translation(Vec3::new(x, GROUND_Y + 23.0, 2.0)),
        ));
    }

    commands
        .spawn((
            Node {
                position_type: PositionType::Absolute,
                top: Val::Px(22.0),
                right: Val::Px(22.0),
                width: Val::Px(410.0),
                padding: UiRect::all(Val::Px(18.0)),
                ..default()
            },
            BackgroundColor(Color::srgba(0.025, 0.04, 0.10, 0.92)),
        ))
        .with_children(|parent| {
            parent.spawn((
                HudText,
                Text::new(""),
                TextFont {
                    font_size: 16.0,
                    ..default()
                },
                TextColor(Color::srgb(0.88, 0.93, 1.0)),
            ));
        });
}

fn handle_controls(
    time: Res<Time>,
    keys: Res<ButtonInput<KeyCode>>,
    mouse: Res<ButtonInput<MouseButton>>,
    rocket_mesh: Res<RocketMesh>,
    mut commands: Commands,
    mut materials: ResMut<Assets<ColorMaterial>>,
    mut show: ResMut<ShowState>,
    mut random: ResMut<RandomState>,
) {
    if keys.just_pressed(KeyCode::Digit1) {
        show.speed = SpeedProfile::Slow;
    } else if keys.just_pressed(KeyCode::Digit2) {
        show.speed = SpeedProfile::Classic;
    } else if keys.just_pressed(KeyCode::Digit3) {
        show.speed = SpeedProfile::Fast;
    }

    if keys.just_pressed(KeyCode::KeyZ) {
        show.palette = Palette::Aurora;
    } else if keys.just_pressed(KeyCode::KeyX) {
        show.palette = Palette::Sunset;
    } else if keys.just_pressed(KeyCode::KeyC) {
        show.palette = Palette::Electric;
    } else if keys.just_pressed(KeyCode::KeyV) {
        show.palette = Palette::Candy;
    }

    if keys.just_pressed(KeyCode::KeyB) {
        show.explosion = ExplosionType::Burst;
    } else if keys.just_pressed(KeyCode::KeyN) {
        show.explosion = ExplosionType::Ring;
    } else if keys.just_pressed(KeyCode::KeyM) {
        show.explosion = ExplosionType::Willow;
    }

    if keys.just_pressed(KeyCode::KeyQ) {
        show.rocket_type = RocketType::Classic;
    } else if keys.just_pressed(KeyCode::KeyW) {
        show.rocket_type = RocketType::Comet;
    } else if keys.just_pressed(KeyCode::KeyE) {
        show.rocket_type = RocketType::Spinner;
    } else if keys.just_pressed(KeyCode::KeyT) {
        show.rocket_type = RocketType::Heavy;
    }

    if keys.just_pressed(KeyCode::KeyA) {
        show.auto_show = !show.auto_show;
        show.auto_timer = 0.0;
    }

    if keys.just_pressed(KeyCode::KeyR) {
        show.speed = [
            SpeedProfile::Slow,
            SpeedProfile::Classic,
            SpeedProfile::Fast,
        ][random.index(3)];
        show.palette = [
            Palette::Aurora,
            Palette::Sunset,
            Palette::Electric,
            Palette::Candy,
        ][random.index(4)];
        show.explosion = [
            ExplosionType::Burst,
            ExplosionType::Ring,
            ExplosionType::Willow,
        ][random.index(3)];
        show.rocket_type = [
            RocketType::Classic,
            RocketType::Comet,
            RocketType::Spinner,
            RocketType::Heavy,
        ][random.index(4)];
    }

    show.auto_timer += time.delta_secs();
    let mut launches = 0;
    while show.auto_show && show.auto_timer >= show.speed.auto_interval() {
        show.auto_timer -= show.speed.auto_interval();
        launches += 1;
    }
    if keys.just_pressed(KeyCode::Space) || mouse.just_pressed(MouseButton::Left) {
        launches += 1;
    }

    for _ in 0..launches {
        launch_firework(
            &mut commands,
            &mut materials,
            &rocket_mesh.0,
            &mut show,
            &mut random,
        );
    }
}

fn launch_firework(
    commands: &mut Commands,
    materials: &mut Assets<ColorMaterial>,
    rocket_mesh: &Handle<Mesh>,
    show: &mut ShowState,
    random: &mut RandomState,
) {
    let rocket_type = show.rocket_type;
    let x = random.range(-500.0, 500.0);
    let color = palette_color(show.palette, random.index(8));
    let base_horizontal = random.range(-38.0, 38.0) * rocket_type.horizontal_factor();
    let velocity = Vec2::new(
        base_horizontal,
        show.speed.launch_speed() * rocket_type.speed_factor(),
    );
    let target_height = random.range(110.0, 285.0);
    commands.spawn((
        Rocket {
            velocity,
            base_horizontal,
            target_height,
            trail_timer: 0.0,
            age: 0.0,
            rocket_type,
            color: color.clone(),
        },
        Mesh2d(rocket_mesh.clone()),
        MeshMaterial2d(materials.add(ColorMaterial::from_color(color))),
        Transform::from_translation(Vec3::new(x, GROUND_Y + 35.0, 9.0))
            .with_scale(Vec3::splat(rocket_type.scale())),
    ));
    show.launches += 1;
}

fn update_rockets(
    time: Res<Time>,
    mut commands: Commands,
    mut random: ResMut<RandomState>,
    mut show: ResMut<ShowState>,
    mut rockets: Query<(Entity, &mut Rocket, &mut Transform)>,
) {
    let delta = time.delta_secs();
    for (entity, mut rocket, mut transform) in &mut rockets {
        rocket.age += delta;
        rocket.velocity.x = match rocket.rocket_type {
            RocketType::Classic => rocket.base_horizontal,
            RocketType::Comet => rocket.base_horizontal + (rocket.age * 2.4).sin() * 42.0,
            RocketType::Spinner => rocket.base_horizontal + (rocket.age * 10.0).sin() * 88.0,
            RocketType::Heavy => rocket.base_horizontal * 0.35,
        };
        rocket.velocity.y -= GRAVITY * delta;
        transform.translation += rocket.velocity.extend(0.0) * delta;
        rocket.trail_timer -= delta;
        while rocket.trail_timer <= 0.0 {
            rocket.trail_timer += rocket.rocket_type.trail_interval();
            let (trail_min, trail_max) = rocket.rocket_type.trail_size();
            spawn_particle(
                &mut commands,
                transform.translation,
                rocket.velocity * -0.04 + Vec2::new(random.range(-8.0, 8.0), -28.0),
                rocket.color.clone(),
                0.26,
                24.0,
                0.8,
                random.range(trail_min, trail_max),
            );
        }

        if transform.translation.y >= rocket.target_height && rocket.velocity.y <= 0.0 {
            spawn_explosion(&mut commands, &mut show, &mut random, transform.translation);
            commands.entity(entity).despawn();
        }
    }
}

fn spawn_explosion(
    commands: &mut Commands,
    show: &mut ShowState,
    random: &mut RandomState,
    origin: Vec3,
) {
    let (count, speed_min, speed_max, gravity, drag, life_min, life_max, size_min, size_max) =
        match show.explosion {
            ExplosionType::Burst => (88, 165.0, 380.0, 110.0, 0.03, 1.25, 2.0, 4.0, 10.0),
            ExplosionType::Ring => (112, 220.0, 325.0, 28.0, 0.08, 1.8, 2.8, 4.0, 9.0),
            ExplosionType::Willow => (78, 125.0, 250.0, 205.0, 0.46, 2.5, 4.0, 5.0, 12.0),
        };

    for index in 0..count {
        let angle = match show.explosion {
            ExplosionType::Ring => TAU * index as f32 / count as f32 + random.range(-0.025, 0.025),
            ExplosionType::Burst => random.range(0.0, TAU),
            ExplosionType::Willow => TAU * index as f32 / count as f32 + random.range(-0.07, 0.07),
        };
        let direction = Vec2::new(angle.cos(), angle.sin());
        let velocity = direction * random.range(speed_min, speed_max);
        let color = palette_color(show.palette, random.index(8));
        spawn_particle(
            commands,
            origin,
            velocity,
            color,
            random.range(life_min, life_max),
            gravity,
            drag,
            random.range(size_min, size_max),
        );
    }

    spawn_particle(
        commands,
        origin,
        Vec2::ZERO,
        Color::srgb(1.0, 0.94, 0.68),
        0.20,
        0.0,
        0.0,
        30.0,
    );
    show.explosions += 1;
}

fn spawn_particle(
    commands: &mut Commands,
    position: Vec3,
    velocity: Vec2,
    color: Color,
    lifetime: f32,
    gravity: f32,
    drag: f32,
    size: f32,
) {
    commands.spawn((
        Particle {
            velocity,
            remaining: lifetime,
            lifetime,
            gravity,
            drag,
        },
        Sprite::from_color(color, Vec2::splat(size)),
        Transform::from_translation(position + Vec3::new(0.0, 0.0, 5.0)),
    ));
}

fn update_particles(
    time: Res<Time>,
    mut commands: Commands,
    mut particles: Query<(Entity, &mut Particle, &mut Transform)>,
) {
    let delta = time.delta_secs();
    for (entity, mut particle, mut transform) in &mut particles {
        particle.remaining -= delta;
        if particle.remaining <= 0.0 {
            commands.entity(entity).despawn();
            continue;
        }

        particle.velocity.y -= particle.gravity * delta;
        let drag_factor = (1.0 - particle.drag * delta).max(0.0);
        particle.velocity *= drag_factor;
        transform.translation += particle.velocity.extend(0.0) * delta;
        let fade = (particle.remaining / particle.lifetime).clamp(0.0, 1.0);
        transform.scale = Vec3::splat(fade.sqrt());
    }
}

fn update_hud(show: Res<ShowState>, mut query: Query<&mut Text, With<HudText>>) {
    let auto_status = if show.auto_show {
        "AUTO SHOW ON"
    } else {
        "AUTO SHOW OFF"
    };
    for mut text in &mut query {
        text.0 = format!(
            "FIREWORKS LAB\n\n\
             {auto_status}\n\
             Launches {:03}   Explosions {:03}\n\n\
             SPEED   {}\n\
             COLOR   {}\n\
             SHAPE   {}\n\
             ROCKET  {}\n\n\
             1 2 3     speed: slow / classic / fast\n\
             Z X C V   color palette\n\
             B N M     burst / ring / willow\n\
             Q W E T   classic / comet / spinner / heavy\n\
             SPACE     launch firework\n\
             A         toggle auto show\n\
             R         randomize style and rocket\n\
             Click anywhere to launch",
            show.launches,
            show.explosions,
            show.speed.label(),
            show.palette.label(),
            show.explosion.label(),
            show.rocket_type.label(),
        );
    }
}

fn palette_color(palette: Palette, index: usize) -> Color {
    let colors = match palette {
        Palette::Aurora => [
            Color::srgb(0.10, 0.92, 1.0),
            Color::srgb(0.18, 1.0, 0.70),
            Color::srgb(0.48, 0.35, 1.0),
            Color::srgb(0.92, 0.98, 1.0),
            Color::srgb(0.08, 0.68, 0.76),
            Color::srgb(0.64, 1.0, 0.32),
            Color::srgb(0.76, 0.48, 1.0),
            Color::srgb(0.20, 0.62, 1.0),
        ],
        Palette::Sunset => [
            Color::srgb(1.0, 0.18, 0.04),
            Color::srgb(1.0, 0.48, 0.02),
            Color::srgb(1.0, 0.08, 0.40),
            Color::srgb(1.0, 0.86, 0.18),
            Color::srgb(1.0, 0.32, 0.14),
            Color::srgb(0.86, 0.06, 0.34),
            Color::srgb(1.0, 0.64, 0.26),
            Color::srgb(1.0, 0.96, 0.38),
        ],
        Palette::Electric => [
            Color::srgb(0.08, 0.38, 1.0),
            Color::srgb(0.40, 0.12, 1.0),
            Color::srgb(1.0, 0.08, 0.78),
            Color::srgb(0.20, 0.86, 1.0),
            Color::srgb(0.12, 0.12, 0.92),
            Color::srgb(0.72, 0.10, 1.0),
            Color::srgb(0.96, 0.34, 0.70),
            Color::srgb(0.48, 0.96, 1.0),
        ],
        Palette::Candy => [
            Color::srgb(1.0, 0.22, 0.62),
            Color::srgb(0.48, 0.30, 1.0),
            Color::srgb(0.18, 0.92, 0.96),
            Color::srgb(1.0, 0.78, 0.16),
            Color::srgb(1.0, 0.40, 0.36),
            Color::srgb(1.0, 0.48, 0.82),
            Color::srgb(0.22, 0.72, 0.68),
            Color::srgb(0.72, 0.52, 1.0),
        ],
    };
    colors[index % colors.len()].clone()
}
