use bevy::prelude::*;
use bevy_pipe_core::{
    ARENA_HEIGHT, ARENA_WIDTH, ArenaMapDocument, Cell, Direction, PlayerInput, Simulation,
    TickInputFrame,
};

const CELL_SIZE: f32 = 56.0;
const MAP_JSON: &str = include_str!("../../../assets/maps/stage1.json");

#[derive(Resource)]
struct ArenaRuntime {
    simulation: Simulation,
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

fn main() {
    let map = ArenaMapDocument::load_json(MAP_JSON).expect("stage1 map JSON must parse");
    let simulation = Simulation::from_map(&map).expect("stage1 map must validate");

    App::new()
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                title: "Pipe Arena — presentation client".into(),
                resolution: (900.0_f32, 760.0_f32).into(),
                ..default()
            }),
            ..default()
        }))
        .insert_resource(ArenaRuntime { simulation })
        .add_systems(Startup, setup)
        .add_systems(FixedUpdate, step_simulation)
        .add_systems(Update, (sync_players, sync_walls, sync_bombs))
        .run();
}

fn setup(mut commands: Commands, runtime: Res<ArenaRuntime>) {
    commands.spawn(Camera2d);

    for y in 0..ARENA_HEIGHT {
        for x in 0..ARENA_WIDTH {
            commands.spawn((
                Sprite::from_color(Color::srgb(0.08, 0.10, 0.13), Vec2::splat(CELL_SIZE - 2.0)),
                Transform::from_translation(cell_translation(Cell::new(x, y), 0.0)),
            ));
        }
    }

    for y in 0..ARENA_HEIGHT {
        for x in 0..ARENA_WIDTH {
            let cell = Cell::new(x, y);
            if is_border(cell)
                || runtime
                    .simulation
                    .state
                    .indestructible_walls
                    .contains(&cell)
            {
                commands.spawn((
                    Sprite::from_color(Color::srgb(0.22, 0.25, 0.30), Vec2::splat(CELL_SIZE - 4.0)),
                    Transform::from_translation(cell_translation(cell, 1.0)),
                ));
            }
        }
    }

    for &cell in &runtime.simulation.state.destructible_walls {
        commands.spawn((
            DestructibleWallView { cell },
            Sprite::from_color(Color::srgb(0.64, 0.35, 0.16), Vec2::splat(CELL_SIZE - 4.0)),
            Transform::from_translation(cell_translation(cell, 1.0)),
        ));
    }

    for player in &runtime.simulation.state.players {
        commands.spawn((
            ActorView {
                actor: player.actor,
            },
            Sprite::from_color(actor_color(player.actor), Vec2::splat(CELL_SIZE - 12.0)),
            Transform::from_translation(cell_translation(player.cell, 2.0)),
        ));
    }
}

fn step_simulation(keys: Res<ButtonInput<KeyCode>>, mut runtime: ResMut<ArenaRuntime>) {
    let tick = runtime.simulation.state.tick;
    let frame = TickInputFrame::new(
        tick,
        vec![
            PlayerInput {
                player: 1,
                direction: direction_from_keys(
                    &keys,
                    KeyCode::KeyW,
                    KeyCode::KeyD,
                    KeyCode::KeyS,
                    KeyCode::KeyA,
                ),
                place_bomb: keys.pressed(KeyCode::Space),
            },
            PlayerInput {
                player: 2,
                direction: direction_from_keys(
                    &keys,
                    KeyCode::ArrowUp,
                    KeyCode::ArrowRight,
                    KeyCode::ArrowDown,
                    KeyCode::ArrowLeft,
                ),
                place_bomb: keys.pressed(KeyCode::Enter),
            },
        ],
    )
    .expect("client input frame must be valid");

    match runtime.simulation.step(&frame) {
        Ok(result) => {
            for event in result.events {
                info!(tick = result.tick, ?event, "presentation event");
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
            .state
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
            .state
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
            .state
            .bombs
            .iter()
            .any(|bomb| bomb.id == view.bomb)
        {
            commands.entity(entity).despawn();
        }
    }

    for bomb in &runtime.simulation.state.bombs {
        if !query.iter().any(|(_, view)| view.bomb == bomb.id) {
            commands.spawn((
                BombView { bomb: bomb.id },
                Sprite::from_color(Color::srgb(0.90, 0.12, 0.10), Vec2::splat(CELL_SIZE - 16.0)),
                Transform::from_translation(cell_translation(bomb.cell, 3.0)),
            ));
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

fn actor_color(actor: u8) -> Color {
    match actor {
        1 => Color::srgb(0.15, 0.65, 0.95),
        2 => Color::srgb(0.95, 0.80, 0.15),
        _ => Color::srgb(0.75, 0.35, 0.90),
    }
}

fn is_border(cell: Cell) -> bool {
    cell.x == 0 || cell.y == 0 || cell.x + 1 == ARENA_WIDTH || cell.y + 1 == ARENA_HEIGHT
}
