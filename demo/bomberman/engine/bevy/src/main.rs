use std::time::Duration;

use bevy::prelude::*;
use bevy_pipe_core::{
    ARENA_HEIGHT, ARENA_WIDTH, BOMBERMAN_DEFINITION, Cell, Direction, PlayerDescription,
    PlayerInput, PresentationEvent, Simulation, TickInputFrame,
};
use serde::Deserialize;

const CELL_SIZE: f32 = 2.0;
const CELL_FILL: f32 = 1.65;
const GRID_LINE: f32 = 0.08;
const CONFIG_JSON: &str = include_str!("../../../config/stage1.json");
const CONFIG_SCHEMA_VERSION: u16 = 1;

#[derive(Resource)]
struct ArenaRuntime {
    simulation: Simulation,
}

#[derive(Resource)]
struct PresentationConfig {
    movement_delay: Duration,
    bomb_fuse_ticks: u16,
    help_hint: String,
    win_format: String,
    players: Vec<ConfiguredPlayer>,
}

struct ConfiguredPlayer {
    actor: u8,
    color: Color,
    keys: ConfiguredKeys,
}

struct ConfiguredKeys {
    up: KeyBinding,
    right: KeyBinding,
    down: KeyBinding,
    left: KeyBinding,
    bomb: KeyBinding,
}

struct KeyBinding {
    code: KeyCode,
    label: String,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct PresentationConfigFile {
    schema_version: u16,
    movement_delay_ms: u64,
    help_hint: String,
    win_format: String,
    players: Vec<PresentationPlayerFile>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct PresentationPlayerFile {
    actor: u8,
    color: [u8; 3],
    keys: PresentationKeysFile,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct PresentationKeysFile {
    up: String,
    right: String,
    down: String,
    left: String,
    bomb: String,
}

impl PresentationConfig {
    fn load_json(
        raw: &str,
        bomb_fuse_ticks: u16,
        source_players: &[PlayerDescription],
    ) -> Result<Self, String> {
        let file: PresentationConfigFile =
            serde_json::from_str(raw).map_err(|error| format!("config JSON: {error}"))?;
        if file.schema_version != CONFIG_SCHEMA_VERSION {
            return Err(format!("unsupported config schema {}", file.schema_version));
        }
        if file.movement_delay_ms == 0 {
            return Err("movement_delay_ms must be positive".into());
        }
        if file.help_hint.trim().is_empty() {
            return Err("help_hint must not be empty".into());
        }
        if file.win_format.trim().is_empty() {
            return Err("win_format must not be empty".into());
        }
        for placeholder in ["{score1}", "{score2}"] {
            if !file.win_format.contains(placeholder) {
                return Err(format!("win_format must contain {placeholder}"));
            }
        }

        let mut players = Vec::with_capacity(file.players.len());
        for player in file.players {
            if players
                .iter()
                .any(|existing: &ConfiguredPlayer| existing.actor == player.actor)
            {
                return Err(format!("duplicate configured actor {}", player.actor));
            }
            if !source_players
                .iter()
                .any(|source| source.actor == player.actor)
            {
                return Err(format!(
                    "config actor {} is absent from generated definition",
                    player.actor
                ));
            }
            players.push(ConfiguredPlayer {
                actor: player.actor,
                color: Color::srgb_u8(player.color[0], player.color[1], player.color[2]),
                keys: ConfiguredKeys::parse(player.keys)?,
            });
        }
        if players.iter().all(|player| player.actor != 1)
            || players.iter().all(|player| player.actor != 2)
        {
            return Err("config must define actors 1 and 2".into());
        }

        Ok(Self {
            movement_delay: Duration::from_millis(file.movement_delay_ms),
            bomb_fuse_ticks,
            help_hint: file.help_hint,
            win_format: file.win_format,
            players,
        })
    }

    fn player(&self, actor: u8) -> &ConfiguredPlayer {
        self.players
            .iter()
            .find(|player| player.actor == actor)
            .expect("validated presentation config actor must exist")
    }

    fn win_text(&self, winner: u8) -> String {
        let score1 = if winner == 1 { "1" } else { "0" };
        let score2 = if winner == 2 { "1" } else { "0" };
        self.win_format
            .replace("{score1}", score1)
            .replace("{score2}", score2)
    }
}

impl ConfiguredKeys {
    fn parse(file: PresentationKeysFile) -> Result<Self, String> {
        Ok(Self {
            up: KeyBinding::parse(file.up)?,
            right: KeyBinding::parse(file.right)?,
            down: KeyBinding::parse(file.down)?,
            left: KeyBinding::parse(file.left)?,
            bomb: KeyBinding::parse(file.bomb)?,
        })
    }
}

impl KeyBinding {
    fn parse(label: String) -> Result<Self, String> {
        let code = match label.as_str() {
            "KeyW" => KeyCode::KeyW,
            "KeyA" => KeyCode::KeyA,
            "KeyS" => KeyCode::KeyS,
            "KeyD" => KeyCode::KeyD,
            "ArrowUp" => KeyCode::ArrowUp,
            "ArrowRight" => KeyCode::ArrowRight,
            "ArrowDown" => KeyCode::ArrowDown,
            "ArrowLeft" => KeyCode::ArrowLeft,
            "Space" => KeyCode::Space,
            "Enter" => KeyCode::Enter,
            _ => return Err(format!("unsupported keyboard key {label}")),
        };
        Ok(Self { code, label })
    }
}

#[derive(Resource)]
struct KeyboardInputState {
    move_elapsed: Duration,
    move_delay: Duration,
}

impl KeyboardInputState {
    fn new(move_delay: Duration) -> Self {
        Self {
            move_elapsed: move_delay,
            move_delay,
        }
    }
}

#[derive(Component)]
struct ActorView {
    actor: u8,
}

#[derive(Component)]
struct BombView {
    bomb: u32,
}

#[derive(Component)]
struct DestructibleWallView {
    cell: Cell,
}

#[derive(Component)]
struct BlastView {
    timer: Timer,
}
#[derive(Component)]
struct HelpPanel;

#[derive(Component)]
struct StatusText {
    winner: Option<u8>,
}

fn main() {
    let definition = &BOMBERMAN_DEFINITION;
    let config = PresentationConfig::load_json(
        CONFIG_JSON,
        definition.rules.bomb_fuse_ticks,
        definition.players,
    )
    .expect("presentation config must validate");
    let simulation =
        Simulation::from_definition(definition).expect("generated Bomberman definition must run");

    App::new()
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                title: "Bomberman — presentation client".into(),
                resolution: (900.0_f32, 760.0_f32).into(),
                ..default()
            }),
            ..default()
        }))
        .insert_resource(Time::<Fixed>::from_hz(f64::from(definition.rules.tick_hz)))
        .insert_resource(ArenaRuntime { simulation })
        .insert_resource(KeyboardInputState::new(config.movement_delay))
        .insert_resource(config)
        .add_systems(Startup, setup)
        .add_systems(FixedUpdate, step_simulation)
        .add_systems(
            Update,
            (
                toggle_help,
                sync_status_text,
                sync_players,
                sync_walls,
                sync_bombs,
                sync_blasts,
            ),
        )
        .run();
}

fn setup(mut commands: Commands, runtime: Res<ArenaRuntime>, config: Res<PresentationConfig>) {
    commands.spawn(Camera2d);

    commands.spawn((
        Sprite::from_color(
            Color::srgb(0.08, 0.10, 0.13),
            Vec2::new(
                f32::from(ARENA_WIDTH) * CELL_SIZE,
                f32::from(ARENA_HEIGHT) * CELL_SIZE,
            ),
        ),
        Transform::from_translation(Vec3::ZERO),
    ));
    for x in 0..ARENA_WIDTH {
        commands.spawn((
            Sprite::from_color(
                Color::srgb(0.18, 0.21, 0.26),
                Vec2::new(GRID_LINE, f32::from(ARENA_HEIGHT) * CELL_SIZE),
            ),
            Transform::from_translation(cell_translation(Cell::new(x, ARENA_HEIGHT / 2), 0.5)),
        ));
    }
    for y in 0..ARENA_HEIGHT {
        commands.spawn((
            Sprite::from_color(
                Color::srgb(0.18, 0.21, 0.26),
                Vec2::new(f32::from(ARENA_WIDTH) * CELL_SIZE, GRID_LINE),
            ),
            Transform::from_translation(cell_translation(Cell::new(ARENA_WIDTH / 2, y), 0.5)),
        ));
    }

    for &cell in &runtime.simulation.state().destructible_walls {
        commands.spawn((
            DestructibleWallView { cell },
            Sprite::from_color(Color::srgb(0.64, 0.35, 0.16), Vec2::splat(CELL_FILL)),
            Transform::from_translation(cell_translation(cell, 1.0)),
        ));
    }

    for player in &runtime.simulation.state().players {
        commands.spawn((
            ActorView {
                actor: player.actor,
            },
            Sprite::from_color(config.player(player.actor).color, Vec2::splat(CELL_FILL)),
            Transform::from_translation(cell_translation(player.cell, 2.0)),
            Visibility::Visible,
        ));
    }

    setup_help_ui(&mut commands, &config);
}

fn setup_help_ui(commands: &mut Commands, config: &PresentationConfig) {
    commands.spawn((
        StatusText { winner: None },
        Node {
            position_type: PositionType::Absolute,
            left: Val::Px(14.0),
            bottom: Val::Px(12.0),
            ..default()
        },
        Text::new(config.help_hint.clone()),
        TextFont {
            font_size: 18.0,
            ..default()
        },
        TextColor(Color::WHITE),
    ));

    let player_one = config.player(1);
    let player_two = config.player(2);
    let help_text = format!(
        "BOMBERMAN HELP\n\n\
         {}       Toggle this help\n\
         Player 1: {} / {} / {} / {}  Move\n\
         Player 1: {}                Place bomb\n\
         Player 2: {} / {} / {} / {}  Move\n\
         Player 2: {}                Place bomb\n\n\
         Movement repeats every {} ms.\n\
         Bombs explode after {} simulation ticks.\n\
         Bomb blasts destroy brown walls.",
        config.help_hint,
        player_one.keys.up.label,
        player_one.keys.right.label,
        player_one.keys.down.label,
        player_one.keys.left.label,
        player_one.keys.bomb.label,
        player_two.keys.up.label,
        player_two.keys.right.label,
        player_two.keys.down.label,
        player_two.keys.left.label,
        player_two.keys.bomb.label,
        config.movement_delay.as_millis(),
        config.bomb_fuse_ticks,
    );

    commands
        .spawn((
            HelpPanel,
            Node {
                position_type: PositionType::Absolute,
                left: Val::Px(24.0),
                top: Val::Px(24.0),
                width: Val::Px(440.0),
                padding: UiRect::all(Val::Px(20.0)),
                display: Display::None,
                ..default()
            },
            BackgroundColor(Color::srgba(0.03, 0.04, 0.07, 0.94)),
        ))
        .with_children(|parent| {
            parent.spawn((
                Text::new(help_text),
                TextFont {
                    font_size: 20.0,
                    ..default()
                },
                TextColor(Color::WHITE),
            ));
        });
}

fn toggle_help(keys: Res<ButtonInput<KeyCode>>, mut query: Query<&mut Node, With<HelpPanel>>) {
    if !keys.just_pressed(KeyCode::F1) {
        return;
    }

    for mut node in &mut query {
        node.display = if matches!(node.display, Display::None) {
            Display::Flex
        } else {
            Display::None
        };
    }
}

fn sync_status_text(
    runtime: Res<ArenaRuntime>,
    config: Res<PresentationConfig>,
    mut query: Query<(&mut Text, &mut StatusText)>,
) {
    let mut alive_count = 0;
    let mut last_alive_actor = None;
    for player in &runtime.simulation.state().players {
        if player.alive {
            alive_count += 1;
            last_alive_actor = Some(player.actor);
        }
    }
    let winner = if alive_count == 1 {
        last_alive_actor
    } else {
        None
    };

    for (mut text, mut status) in &mut query {
        if status.winner == winner {
            continue;
        }
        status.winner = winner;
        text.0 = winner.map_or_else(
            || config.help_hint.clone(),
            |winner| config.win_text(winner),
        );
    }
}

fn step_simulation(
    mut commands: Commands,
    time: Res<Time<Fixed>>,
    keys: Res<ButtonInput<KeyCode>>,
    config: Res<PresentationConfig>,
    mut input_state: ResMut<KeyboardInputState>,
    mut runtime: ResMut<ArenaRuntime>,
) {
    input_state.move_elapsed = input_state.move_elapsed.saturating_add(time.delta());
    let move_ready = input_state.move_elapsed >= input_state.move_delay;
    let player_one = config.player(1);
    let player_two = config.player(2);
    let player_one_direction = if move_ready {
        direction_from_keys(
            &keys,
            player_one.keys.up.code,
            player_one.keys.right.code,
            player_one.keys.down.code,
            player_one.keys.left.code,
        )
    } else {
        Direction::None
    };
    let player_two_direction = if move_ready {
        direction_from_keys(
            &keys,
            player_two.keys.up.code,
            player_two.keys.right.code,
            player_two.keys.down.code,
            player_two.keys.left.code,
        )
    } else {
        Direction::None
    };
    let movement_requested = !matches!(player_one_direction, Direction::None)
        || !matches!(player_two_direction, Direction::None);
    let tick = runtime.simulation.state().tick;
    let frame = TickInputFrame::new(
        tick,
        vec![
            PlayerInput {
                player: 1,
                direction: player_one_direction,
                place_bomb: keys.pressed(player_one.keys.bomb.code),
            },
            PlayerInput {
                player: 2,
                direction: player_two_direction,
                place_bomb: keys.pressed(player_two.keys.bomb.code),
            },
        ],
    )
    .expect("client input frame must be valid");

    match runtime.simulation.step(&frame) {
        Ok(result) => {
            if movement_requested {
                input_state.move_elapsed = Duration::ZERO;
            }
            for event in result.events {
                info!(tick = result.tick, ?event, "presentation event");
                if let PresentationEvent::BombExploded { cell, .. } = event {
                    for blast_cell in runtime.simulation.state().blast_cells(cell) {
                        commands.spawn((
                            BlastView {
                                timer: Timer::from_seconds(0.2, TimerMode::Once),
                            },
                            Sprite::from_color(
                                Color::srgba(1.0, 0.85, 0.05, 0.88),
                                Vec2::splat(CELL_FILL),
                            ),
                            Transform::from_translation(cell_translation(blast_cell, 1.5)),
                        ));
                    }
                }
            }
        }
        Err(error) => error!(?error, "simulation step failed"),
    }
}

fn sync_players(
    runtime: Res<ArenaRuntime>,
    mut query: Query<(&ActorView, &mut Transform, &mut Visibility)>,
) {
    for (view, mut transform, mut visibility) in &mut query {
        let Some(player) = runtime
            .simulation
            .state()
            .players
            .iter()
            .find(|player| player.actor == view.actor)
        else {
            *visibility = Visibility::Hidden;
            continue;
        };
        transform.translation = cell_translation(player.cell, 2.0);
        *visibility = if player.alive {
            Visibility::Visible
        } else {
            Visibility::Hidden
        };
    }
}

fn sync_walls(
    runtime: Res<ArenaRuntime>,
    mut query: Query<(&DestructibleWallView, &mut Visibility)>,
) {
    for (view, mut visibility) in &mut query {
        *visibility = if runtime
            .simulation
            .state()
            .destructible_walls
            .contains(&view.cell)
        {
            Visibility::Visible
        } else {
            Visibility::Hidden
        };
    }
}

fn sync_bombs(
    mut commands: Commands,
    runtime: Res<ArenaRuntime>,
    query: Query<(Entity, &BombView)>,
) {
    for (entity, view) in &query {
        if !runtime
            .simulation
            .state()
            .bombs
            .iter()
            .any(|bomb| bomb.id == view.bomb)
        {
            commands.entity(entity).despawn();
        }
    }

    for bomb in &runtime.simulation.state().bombs {
        if !query.iter().any(|(_, view)| view.bomb == bomb.id) {
            commands.spawn((
                BombView { bomb: bomb.id },
                Sprite::from_color(Color::srgb(0.90, 0.12, 0.10), Vec2::splat(1.35)),
                Transform::from_translation(cell_translation(bomb.cell, 3.0)),
            ));
        }
    }
}

fn sync_blasts(
    mut commands: Commands,
    time: Res<Time>,
    mut query: Query<(Entity, &mut BlastView)>,
) {
    for (entity, mut blast) in &mut query {
        blast.timer.tick(time.delta());
        if blast.timer.finished() {
            commands.entity(entity).despawn();
        }
    }
}

fn direction_from_keys(
    keys: &ButtonInput<KeyCode>,
    up: KeyCode,
    right: KeyCode,
    down: KeyCode,
    left: KeyCode,
) -> Direction {
    if keys.pressed(up) {
        Direction::Up
    } else if keys.pressed(right) {
        Direction::Right
    } else if keys.pressed(down) {
        Direction::Down
    } else if keys.pressed(left) {
        Direction::Left
    } else {
        Direction::None
    }
}

fn cell_translation(cell: Cell, z: f32) -> Vec3 {
    let center_x = (f32::from(ARENA_WIDTH) - 1.0) / 2.0;
    let center_y = (f32::from(ARENA_HEIGHT) - 1.0) / 2.0;
    Vec3::new(
        (f32::from(cell.x) - center_x) * CELL_SIZE,
        (center_y - f32::from(cell.y)) * CELL_SIZE,
        z,
    )
}
