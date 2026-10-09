use bevy::prelude::*;
use bevy_gamesim_ast::{
    VILLAGE_RULESET_ID, VillageCompileOptions, VillageIr, compile_village, parse_village,
};

const VILLAGE_SOURCE: &str = include_str!("../../../../bomberman/engine/bevy/lang/village.json");
const MAP_SCALE: f32 = 0.72;
const WINDOW_WIDTH: f32 = 1_200.0;
const WINDOW_HEIGHT: f32 = 800.0;

#[derive(Resource)]
struct VillageDefinition(VillageIr);

#[derive(Resource)]
struct SignalState {
    timer: Timer,
    green: bool,
}

#[derive(Component)]
struct MovingAgent {
    start: Vec2,
    end: Vec2,
    speed: f32,
    phase: f32,
}

#[derive(Component)]
struct SignalLight;

fn main() {
    let source = parse_village(VILLAGE_SOURCE).expect("village JSON must parse");
    let compiled = compile_village(
        source,
        VillageCompileOptions {
            ruleset_id: VILLAGE_RULESET_ID,
            expected_width_m: 1_000,
            expected_height_m: 1_000,
            expected_outgoing_streets: 6,
        },
    )
    .expect("village definition must compile");

    App::new()
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                title: "Village traffic sandbox".into(),
                resolution: (WINDOW_WIDTH, WINDOW_HEIGHT).into(),
                ..default()
            }),
            ..default()
        }))
        .insert_resource(VillageDefinition(compiled.ir))
        .insert_resource(SignalState {
            timer: Timer::from_seconds(3.0, TimerMode::Repeating),
            green: true,
        })
        .add_systems(Startup, setup)
        .add_systems(Update, (animate_agents, cycle_signal))
        .run();
}

fn setup(
    mut commands: Commands,
    definition: Res<VillageDefinition>,
    mut fonts: ResMut<Assets<Font>>,
) {
    let emoji_font = fonts.add(
        Font::try_from_bytes(
            std::fs::read("/usr/share/fonts/noto/NotoColorEmoji.ttf")
                .expect("Noto Color Emoji must be installed"),
        )
        .expect("Noto Color Emoji must be a valid font"),
    );

    commands.spawn(Camera2d);

    commands.spawn((
        Sprite::from_color(Color::srgb(0.08, 0.15, 0.12), Vec2::new(1_000.0, 760.0)),
        Transform::from_translation(Vec3::new(0.0, 0.0, -2.0)),
    ));

    for street in &definition.0.world.streets {
        let start = point_to_screen(street.path.start);
        let end = point_to_screen(street.path.end);
        let delta = end - start;
        let width = f32::from(street.width_m).max(2.0) * MAP_SCALE;
        let color = if street.role == "outgoing" {
            Color::srgb(0.34, 0.36, 0.38)
        } else {
            Color::srgb(0.25, 0.28, 0.27)
        };
        commands.spawn((
            Sprite::from_color(color, Vec2::new(delta.length(), width)),
            Transform::from_translation(Vec3::new(
                (start.x + end.x) * 0.5,
                (start.y + end.y) * 0.5,
                0.0,
            ))
            .with_rotation(Quat::from_rotation_z(delta.y.atan2(delta.x))),
        ));
    }

    commands.spawn((
        Sprite::from_color(Color::srgb(0.78, 0.66, 0.37), Vec2::splat(42.0)),
        Transform::from_translation(Vec3::new(0.0, 0.0, 1.0)),
    ));
    commands.spawn((
        Sprite::from_color(Color::srgb(0.12, 0.12, 0.12), Vec2::new(11.0, 62.0)),
        Transform::from_translation(Vec3::new(28.0, 28.0, 2.0)),
    ));
    commands.spawn((
        SignalLight,
        Sprite::from_color(Color::srgb(0.15, 0.9, 0.28), Vec2::splat(13.0)),
        Transform::from_translation(Vec3::new(28.0, 49.0, 3.0)),
    ));

    let outgoing: Vec<_> = definition
        .0
        .world
        .streets
        .iter()
        .filter(|street| street.role == "outgoing")
        .collect();
    let local: Vec<_> = definition
        .0
        .world
        .streets
        .iter()
        .filter(|street| street.role == "local")
        .collect();
    for (index, agent) in definition.0.agents.iter().enumerate() {
        let (start, end) = if agent.kind == "pedestrian" {
            let pedestrian_index = definition.0.agents[..index]
                .iter()
                .filter(|candidate| candidate.kind == "pedestrian")
                .count();
            let prefers_local = pedestrian_index % 5 != 4;
            let route_pool = if prefers_local && !local.is_empty() {
                &local
            } else {
                &outgoing
            };
            let route_index = (pedestrian_index * 3 + 1) % route_pool.len();
            let street = route_pool[route_index];
            (
                point_to_screen(street.path.start),
                point_to_screen(street.path.end),
            )
        } else {
            let route_index = (index * 5 + 1) % outgoing.len();
            let street = outgoing[route_index];
            (
                point_to_screen(street.path.start),
                point_to_screen(street.path.end),
            )
        };
        let phase = ((index * 17) % 100) as f32 / 100.0;
        commands.spawn((
            MovingAgent {
                start,
                end,
                speed: 0.12 + index as f32 * 0.012,
                phase,
            },
            Text2d::new(agent.emoji.clone()),
            TextFont {
                font: emoji_font.clone(),
                font_size: 30.0,
                ..default()
            },
            TextColor(Color::WHITE),
            Transform::from_translation(Vec3::new(start.x, start.y, 5.0)),
        ));
    }

    commands.spawn((
        Text::new(format!(
            "VILLAGE TRAFFIC SANDBOX\n{} streets  •  {} agents  •  seeded local roads",
            definition.0.world.streets.len(),
            definition.0.agents.len()
        )),
        Node {
            position_type: PositionType::Absolute,
            left: Val::Px(24.0),
            top: Val::Px(20.0),
            ..default()
        },
        TextFont {
            font_size: 20.0,
            ..default()
        },
        TextColor(Color::srgb(0.92, 0.95, 0.88)),
    ));
    commands.spawn((
        Text::new(format!(
            "Connected streets • {} local roads • center signal cycles every 3 seconds",
            definition
                .0
                .world
                .streets
                .iter()
                .filter(|street| street.role == "local")
                .count()
        )),
        Node {
            position_type: PositionType::Absolute,
            left: Val::Px(24.0),
            bottom: Val::Px(20.0),
            ..default()
        },
        TextFont {
            font_size: 16.0,
            ..default()
        },
        TextColor(Color::srgb(0.65, 0.72, 0.68)),
    ));
}

fn point_to_screen(point: bevy_gamesim_ast::VillagePointSource) -> Vec2 {
    Vec2::new(
        (point.x_m as f32 - 500.0) * MAP_SCALE,
        (500.0 - point.y_m as f32) * MAP_SCALE,
    )
}

fn animate_agents(time: Res<Time>, mut agents: Query<(&MovingAgent, &mut Transform)>) {
    for (agent, mut transform) in &mut agents {
        let progress = ((time.elapsed_secs() * agent.speed + agent.phase).sin() + 1.0) * 0.5;
        let position = agent.start.lerp(agent.end, progress);
        transform.translation.x = position.x;
        transform.translation.y = position.y;
    }
}

fn cycle_signal(
    time: Res<Time>,
    mut state: ResMut<SignalState>,
    mut lights: Query<&mut Sprite, With<SignalLight>>,
) {
    state.timer.tick(time.delta());
    if state.timer.just_finished() {
        state.green = !state.green;
    }
    for mut sprite in &mut lights {
        sprite.color = if state.green {
            Color::srgb(0.15, 0.9, 0.28)
        } else {
            Color::srgb(0.95, 0.18, 0.12)
        };
    }
}
