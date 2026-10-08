//! Deterministic, engine-neutral Stage 1 simulation core for Bevy GameSim.
//!
//! The crate deliberately has no engine dependency. It defines the versioned
//! protocol and authoritative local loop that a Bevy plugin and external engine
//! adapters can consume without introducing a second gameplay authority.

use serde::{Deserialize, Serialize};
use std::fmt;

pub const PROTOCOL_VERSION: u16 = 1;
pub const MAP_SCHEMA_VERSION: u16 = 1;
pub const RULESET_ID: &str = "pipe_arena_stage1";
pub const ARENA_WIDTH: u8 = 13;
pub const ARENA_HEIGHT: u8 = 11;
pub const MAX_PLAYERS: usize = 4;
pub const MAX_INTENTS: usize = 16;
pub const MAX_COMMANDS: usize = 64;
pub const MAX_EVENTS: usize = 64;
pub const MAX_REJECTIONS: usize = 32;
pub const BOMB_FUSE_TICKS: u16 = 3;
pub const BOMB_RADIUS: u8 = 2;

pub type StateHash = u64;

#[derive(Clone, Copy, Debug, Deserialize, Eq, Hash, Ord, PartialEq, PartialOrd, Serialize)]
pub struct Cell {
    pub x: u8,
    pub y: u8,
}

impl Cell {
    pub const fn new(x: u8, y: u8) -> Self {
        Self { x, y }
    }

    fn encode(self, bytes: &mut Vec<u8>) {
        bytes.push(self.x);
        bytes.push(self.y);
    }
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, Hash, PartialEq, Serialize)]
#[repr(u8)]
pub enum Direction {
    None = 0,
    Up = 1,
    Right = 2,
    Down = 3,
    Left = 4,
}

impl Direction {
    fn decode(value: u8) -> Result<Self, ProtocolError> {
        match value {
            0 => Ok(Self::None),
            1 => Ok(Self::Up),
            2 => Ok(Self::Right),
            3 => Ok(Self::Down),
            4 => Ok(Self::Left),
            _ => Err(ProtocolError::InvalidDirection(value)),
        }
    }

    fn delta(self) -> (i8, i8) {
        match self {
            Self::None => (0, 0),
            Self::Up => (0, -1),
            Self::Right => (1, 0),
            Self::Down => (0, 1),
            Self::Left => (-1, 0),
        }
    }
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct PlayerInput {
    pub player: u8,
    pub direction: Direction,
    pub place_bomb: bool,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct TickInputFrame {
    pub tick: u64,
    pub players: Vec<PlayerInput>,
}

impl TickInputFrame {
    pub fn new(tick: u64, mut players: Vec<PlayerInput>) -> Result<Self, ProtocolError> {
        if players.len() > MAX_PLAYERS {
            return Err(ProtocolError::TooManyPlayers(players.len()));
        }
        players.sort_by_key(|input| input.player);
        for pair in players.windows(2) {
            if pair[0].player == pair[1].player {
                return Err(ProtocolError::DuplicatePlayer(pair[0].player));
            }
        }
        Ok(Self { tick, players })
    }

    pub fn encode(&self) -> Vec<u8> {
        let mut bytes = Vec::with_capacity(13 + self.players.len() * 3);
        bytes.extend_from_slice(&PROTOCOL_VERSION.to_le_bytes());
        bytes.extend_from_slice(&self.tick.to_le_bytes());
        bytes.push(self.players.len() as u8);
        for input in &self.players {
            bytes.push(input.player);
            bytes.push(input.direction as u8);
            bytes.push(u8::from(input.place_bomb));
        }
        bytes
    }

    pub fn decode(bytes: &[u8]) -> Result<Self, ProtocolError> {
        let mut reader = Reader::new(bytes);
        let version = reader.u16()?;
        if version != PROTOCOL_VERSION {
            return Err(ProtocolError::UnsupportedVersion(version));
        }
        let tick = reader.u64()?;
        let count = reader.u8()? as usize;
        if count > MAX_PLAYERS {
            return Err(ProtocolError::TooManyPlayers(count));
        }
        let mut players = Vec::with_capacity(count);
        for _ in 0..count {
            players.push(PlayerInput {
                player: reader.u8()?,
                direction: Direction::decode(reader.u8()?)?,
                place_bomb: match reader.u8()? {
                    0 => false,
                    1 => true,
                    value => return Err(ProtocolError::InvalidBoolean(value)),
                },
            });
        }
        if reader.remaining() != 0 {
            return Err(ProtocolError::TrailingBytes(reader.remaining()));
        }
        Self::new(tick, players)
    }
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct MapCell {
    pub x: u8,
    pub y: u8,
}

impl MapCell {
    fn as_cell(&self) -> Cell {
        Cell::new(self.x, self.y)
    }
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct MapSpawn {
    pub actor: u8,
    pub x: u8,
    pub y: u8,
}

impl MapSpawn {
    fn cell(&self) -> Cell {
        Cell::new(self.x, self.y)
    }
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct ArenaMapDocument {
    pub schema_version: u16,
    pub ruleset_id: String,
    pub width: u8,
    pub height: u8,
    #[serde(default)]
    pub indestructible_walls: Vec<MapCell>,
    #[serde(default)]
    pub destructible_walls: Vec<MapCell>,
    pub spawns: Vec<MapSpawn>,
}

impl ArenaMapDocument {
    pub fn load_json(json: &str) -> Result<Self, MapError> {
        serde_json::from_str(json).map_err(|error| MapError::Json(error.to_string()))
    }

    pub fn validate(&self) -> Result<ArenaMap, MapError> {
        if self.schema_version != MAP_SCHEMA_VERSION {
            return Err(MapError::UnsupportedSchema(self.schema_version));
        }
        if self.ruleset_id != RULESET_ID {
            return Err(MapError::RulesetMismatch(self.ruleset_id.clone()));
        }
        if self.width != ARENA_WIDTH || self.height != ARENA_HEIGHT {
            return Err(MapError::InvalidDimensions {
                width: self.width,
                height: self.height,
            });
        }
        if self.spawns.is_empty() {
            return Err(MapError::NoPlayers);
        }
        if self.spawns.len() > MAX_PLAYERS {
            return Err(MapError::TooManyPlayers(self.spawns.len()));
        }

        let mut indestructible_walls = self
            .indestructible_walls
            .iter()
            .map(MapCell::as_cell)
            .collect::<Vec<_>>();
        let mut destructible_walls = self
            .destructible_walls
            .iter()
            .map(MapCell::as_cell)
            .collect::<Vec<_>>();
        indestructible_walls.sort_unstable();
        destructible_walls.sort_unstable();

        validate_cells("indestructible wall", &indestructible_walls)?;
        validate_cells("destructible wall", &destructible_walls)?;
        for cell in &indestructible_walls {
            if destructible_walls.contains(cell) {
                return Err(MapError::OverlappingWalls(*cell));
            }
        }

        let mut players = Vec::with_capacity(self.spawns.len());
        for spawn in &self.spawns {
            if spawn.actor == 0 {
                return Err(MapError::InvalidActor(spawn.actor));
            }
            let cell = spawn.cell();
            if players
                .iter()
                .any(|player: &PlayerState| player.actor == spawn.actor)
            {
                return Err(MapError::DuplicateActor(spawn.actor));
            }
            if players
                .iter()
                .any(|player: &PlayerState| player.cell == cell)
            {
                return Err(MapError::DuplicateSpawnCell(cell));
            }
            if is_border(cell)
                || indestructible_walls.contains(&cell)
                || destructible_walls.contains(&cell)
            {
                return Err(MapError::SpawnOnBlockedCell(cell));
            }
            validate_cell("spawn", cell)?;
            players.push(PlayerState {
                actor: spawn.actor,
                cell,
                alive: true,
            });
        }
        players.sort_by_key(|player| player.actor);

        Ok(ArenaMap {
            players,
            indestructible_walls,
            destructible_walls,
        })
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ArenaMap {
    pub players: Vec<PlayerState>,
    pub indestructible_walls: Vec<Cell>,
    pub destructible_walls: Vec<Cell>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum MapError {
    Json(String),
    UnsupportedSchema(u16),
    RulesetMismatch(String),
    InvalidDimensions { width: u8, height: u8 },
    TooManyPlayers(usize),
    NoPlayers,
    InvalidActor(u8),
    OutOfBounds { kind: &'static str, cell: Cell },
    BorderCell { kind: &'static str, cell: Cell },
    DuplicateCell { kind: &'static str, cell: Cell },
    OverlappingWalls(Cell),
    DuplicateActor(u8),
    DuplicateSpawnCell(Cell),
    SpawnOnBlockedCell(Cell),
}

impl fmt::Display for MapError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "map validation failed: {self:?}")
    }
}

impl std::error::Error for MapError {}

fn validate_cells(kind: &'static str, cells: &[Cell]) -> Result<(), MapError> {
    for &cell in cells {
        validate_cell(kind, cell)?;
        if cells.iter().filter(|candidate| **candidate == cell).count() > 1 {
            return Err(MapError::DuplicateCell { kind, cell });
        }
    }
    Ok(())
}

fn validate_cell(kind: &'static str, cell: Cell) -> Result<(), MapError> {
    if cell.x >= ARENA_WIDTH || cell.y >= ARENA_HEIGHT {
        return Err(MapError::OutOfBounds { kind, cell });
    }
    if is_border(cell) && kind.contains("wall") {
        return Err(MapError::BorderCell { kind, cell });
    }
    Ok(())
}

fn is_border(cell: Cell) -> bool {
    cell.x == 0 || cell.y == 0 || cell.x + 1 == ARENA_WIDTH || cell.y + 1 == ARENA_HEIGHT
}

pub fn default_map_document() -> ArenaMapDocument {
    ArenaMapDocument {
        schema_version: MAP_SCHEMA_VERSION,
        ruleset_id: RULESET_ID.to_owned(),
        width: ARENA_WIDTH,
        height: ARENA_HEIGHT,
        indestructible_walls: (2..ARENA_WIDTH - 1)
            .step_by(2)
            .flat_map(|x| {
                (2..ARENA_HEIGHT - 1)
                    .step_by(2)
                    .map(move |y| MapCell { x, y })
            })
            .collect(),
        destructible_walls: vec![
            MapCell { x: 3, y: 3 },
            MapCell { x: 5, y: 3 },
            MapCell { x: 7, y: 3 },
            MapCell { x: 9, y: 3 },
            MapCell { x: 3, y: 5 },
            MapCell { x: 5, y: 5 },
            MapCell { x: 7, y: 5 },
            MapCell { x: 9, y: 5 },
            MapCell { x: 3, y: 7 },
            MapCell { x: 5, y: 7 },
            MapCell { x: 7, y: 7 },
            MapCell { x: 9, y: 7 },
        ],
        spawns: vec![
            MapSpawn {
                actor: 1,
                x: 1,
                y: 1,
            },
            MapSpawn {
                actor: 2,
                x: 11,
                y: 9,
            },
        ],
    }
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct InputFixtureDocument {
    pub protocol_version: u16,
    pub ruleset_id: String,
    pub frames: Vec<InputFixtureFrame>,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct InputFixtureFrame {
    pub tick: u64,
    pub players: Vec<InputFixturePlayer>,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct InputFixturePlayer {
    pub player: u8,
    pub direction: String,
    pub place_bomb: bool,
}

impl InputFixtureDocument {
    pub fn load_json(json: &str) -> Result<Self, FixtureError> {
        serde_json::from_str(json).map_err(|error| FixtureError::Json(error.to_string()))
    }

    pub fn to_frames(&self) -> Result<Vec<TickInputFrame>, FixtureError> {
        if self.protocol_version != PROTOCOL_VERSION {
            return Err(FixtureError::UnsupportedProtocol(self.protocol_version));
        }
        if self.ruleset_id != RULESET_ID {
            return Err(FixtureError::RulesetMismatch(self.ruleset_id.clone()));
        }

        self.frames
            .iter()
            .map(|frame| {
                let players = frame
                    .players
                    .iter()
                    .map(|player| {
                        Ok(PlayerInput {
                            player: player.player,
                            direction: parse_direction(&player.direction).ok_or_else(|| {
                                FixtureError::InvalidDirection {
                                    tick: frame.tick,
                                    player: player.player,
                                    value: player.direction.clone(),
                                }
                            })?,
                            place_bomb: player.place_bomb,
                        })
                    })
                    .collect::<Result<Vec<_>, FixtureError>>()?;
                TickInputFrame::new(frame.tick, players).map_err(FixtureError::Protocol)
            })
            .collect()
    }
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct ExpectedRunDocument {
    pub protocol_version: u16,
    pub ruleset_id: String,
    pub initial_state_hash: StateHash,
    pub ticks: Vec<ExpectedTickDocument>,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct ExpectedTickDocument {
    pub tick: u64,
    pub commands: Vec<AuthoritativeCommand>,
    pub events: Vec<PresentationEvent>,
    pub rejections: Vec<Rejection>,
    pub state_hash: StateHash,
}

impl ExpectedRunDocument {
    pub fn load_json(json: &str) -> Result<Self, FixtureError> {
        serde_json::from_str(json).map_err(|error| FixtureError::Json(error.to_string()))
    }

    pub fn from_results(initial_state_hash: StateHash, results: &[TickResult]) -> Self {
        Self {
            protocol_version: PROTOCOL_VERSION,
            ruleset_id: RULESET_ID.to_owned(),
            initial_state_hash,
            ticks: results
                .iter()
                .map(|result| ExpectedTickDocument {
                    tick: result.tick,
                    commands: result.commands.clone(),
                    events: result.events.clone(),
                    rejections: result.rejections.clone(),
                    state_hash: result.state_hash,
                })
                .collect(),
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum FixtureError {
    Json(String),
    UnsupportedProtocol(u16),
    RulesetMismatch(String),
    InvalidDirection {
        tick: u64,
        player: u8,
        value: String,
    },
    Protocol(ProtocolError),
}

impl fmt::Display for FixtureError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "fixture error: {self:?}")
    }
}

impl std::error::Error for FixtureError {}

fn parse_direction(value: &str) -> Option<Direction> {
    if value.eq_ignore_ascii_case("none") {
        Some(Direction::None)
    } else if value.eq_ignore_ascii_case("up") {
        Some(Direction::Up)
    } else if value.eq_ignore_ascii_case("right") {
        Some(Direction::Right)
    } else if value.eq_ignore_ascii_case("down") {
        Some(Direction::Down)
    } else if value.eq_ignore_ascii_case("left") {
        Some(Direction::Left)
    } else {
        None
    }
}
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub enum Intent {
    Move { actor: u8, direction: Direction },
    PlaceBomb { actor: u8 },
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub enum AuthoritativeCommand {
    Move {
        actor: u8,
        from: Cell,
        to: Cell,
    },
    PlaceBomb {
        actor: u8,
        bomb: u32,
        cell: Cell,
        fuse_ticks: u16,
    },
    DetonateBomb {
        bomb: u32,
        cell: Cell,
    },
    DestroyWall {
        cell: Cell,
    },
    EliminatePlayer {
        actor: u8,
    },
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub enum PresentationEvent {
    BombPlaced { bomb: u32, cell: Cell },
    BombExploded { bomb: u32, cell: Cell },
    WallDestroyed { cell: Cell },
    PlayerDefeated { actor: u8, cell: Cell },
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub enum RejectionReason {
    UnknownPlayer,
    PlayerDefeated,
    BlockedCell,
    BombAlreadyPresent,
    BombLimit,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct Rejection {
    pub actor: u8,
    pub intent: Intent,
    pub reason: RejectionReason,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ProtocolError {
    InvalidDirection(u8),
    InvalidBoolean(u8),
    UnsupportedVersion(u16),
    TooManyPlayers(usize),
    DuplicatePlayer(u8),
    UnexpectedEnd,
    TrailingBytes(usize),
}

impl fmt::Display for ProtocolError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidDirection(value) => write!(formatter, "invalid direction {value}"),
            Self::InvalidBoolean(value) => write!(formatter, "invalid boolean {value}"),
            Self::UnsupportedVersion(version) => {
                write!(formatter, "unsupported protocol version {version}")
            }
            Self::TooManyPlayers(count) => write!(formatter, "too many players: {count}"),
            Self::DuplicatePlayer(player) => write!(formatter, "duplicate player {player}"),
            Self::UnexpectedEnd => formatter.write_str("unexpected end of protocol data"),
            Self::TrailingBytes(count) => write!(formatter, "trailing protocol bytes: {count}"),
        }
    }
}

impl std::error::Error for ProtocolError {}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum BufferError {
    Full { capacity: usize },
}

impl fmt::Display for BufferError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Full { capacity } => {
                write!(formatter, "bounded buffer is full at {capacity} items")
            }
        }
    }
}

impl std::error::Error for BufferError {}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct BoundedBuffer<T> {
    capacity: usize,
    items: Vec<T>,
}

impl<T> BoundedBuffer<T> {
    pub fn new(capacity: usize) -> Self {
        Self {
            capacity,
            items: Vec::with_capacity(capacity),
        }
    }

    pub fn push(&mut self, item: T) -> Result<(), BufferError> {
        if self.items.len() == self.capacity {
            return Err(BufferError::Full {
                capacity: self.capacity,
            });
        }
        self.items.push(item);
        Ok(())
    }

    pub fn as_slice(&self) -> &[T] {
        &self.items
    }

    pub fn into_inner(self) -> Vec<T> {
        self.items
    }
}

pub trait Pipe {
    type Input;
    type Output;
    type Fault;

    fn run(
        &mut self,
        input: &Self::Input,
        output: &mut BoundedBuffer<Self::Output>,
    ) -> Result<(), Self::Fault>;
}

pub struct InterpreterInput<'a> {
    pub view: WorldView<'a>,
    pub frame: &'a TickInputFrame,
}

pub trait InterpreterPipe {
    type Fault;

    fn run(
        &mut self,
        input: &InterpreterInput<'_>,
        output: &mut BoundedBuffer<Intent>,
    ) -> Result<(), Self::Fault>;
}

#[derive(Default)]
pub struct RustGameplayInterpreter;

impl InterpreterPipe for RustGameplayInterpreter {
    type Fault = BufferError;

    fn run(
        &mut self,
        input: &InterpreterInput<'_>,
        output: &mut BoundedBuffer<Intent>,
    ) -> Result<(), Self::Fault> {
        for player in &input.frame.players {
            if input
                .view
                .player(player.player)
                .is_some_and(|state| !state.alive)
            {
                continue;
            }
            if player.direction != Direction::None {
                output.push(Intent::Move {
                    actor: player.player,
                    direction: player.direction,
                })?;
            }
            if player.place_bomb {
                output.push(Intent::PlaceBomb {
                    actor: player.player,
                })?;
            }
        }
        Ok(())
    }
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct PlayerState {
    pub actor: u8,
    pub cell: Cell,
    pub alive: bool,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct BombState {
    pub id: u32,
    pub cell: Cell,
    pub owner: u8,
    pub fuse_ticks: u16,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct GameState {
    pub tick: u64,
    pub players: Vec<PlayerState>,
    pub bombs: Vec<BombState>,
    pub indestructible_walls: Vec<Cell>,
    pub destructible_walls: Vec<Cell>,
    pub next_bomb_id: u32,
}

impl GameState {
    pub fn initial() -> Self {
        Self::from_map(&default_map_document()).expect("default map must validate")
    }

    pub fn from_map(document: &ArenaMapDocument) -> Result<Self, MapError> {
        let map = document.validate()?;
        Ok(Self {
            tick: 0,
            players: map.players,
            bombs: Vec::new(),
            indestructible_walls: map.indestructible_walls,
            destructible_walls: map.destructible_walls,
            next_bomb_id: 1,
        })
    }

    pub fn hash(&self) -> StateHash {
        fnv1a64(&self.canonical_bytes())
    }

    pub fn canonical_bytes(&self) -> Vec<u8> {
        let mut bytes = Vec::new();
        bytes.extend_from_slice(&PROTOCOL_VERSION.to_le_bytes());
        bytes.extend_from_slice(&(RULESET_ID.len() as u16).to_le_bytes());
        bytes.extend_from_slice(RULESET_ID.as_bytes());
        bytes.extend_from_slice(&self.tick.to_le_bytes());
        bytes.push(ARENA_WIDTH);
        bytes.push(ARENA_HEIGHT);
        bytes.push(self.players.len() as u8);
        for player in &self.players {
            bytes.push(player.actor);
            player.cell.encode(&mut bytes);
            bytes.push(u8::from(player.alive));
        }
        bytes.push(self.bombs.len() as u8);
        for bomb in &self.bombs {
            bytes.extend_from_slice(&bomb.id.to_le_bytes());
            bomb.cell.encode(&mut bytes);
            bytes.push(bomb.owner);
            bytes.extend_from_slice(&bomb.fuse_ticks.to_le_bytes());
        }
        bytes.push(self.indestructible_walls.len() as u8);
        for wall in &self.indestructible_walls {
            wall.encode(&mut bytes);
        }
        bytes.push(self.destructible_walls.len() as u8);
        for wall in &self.destructible_walls {
            wall.encode(&mut bytes);
        }
        bytes.extend_from_slice(&self.next_bomb_id.to_le_bytes());
        bytes
    }

    fn player(&self, actor: u8) -> Option<&PlayerState> {
        self.players.iter().find(|player| player.actor == actor)
    }

    fn player_mut(&mut self, actor: u8) -> Option<&mut PlayerState> {
        self.players.iter_mut().find(|player| player.actor == actor)
    }

    fn is_border(cell: Cell) -> bool {
        cell.x == 0 || cell.y == 0 || cell.x + 1 == ARENA_WIDTH || cell.y + 1 == ARENA_HEIGHT
    }

    fn has_indestructible_wall(&self, cell: Cell) -> bool {
        self.indestructible_walls.contains(&cell)
    }

    fn has_wall(&self, cell: Cell) -> bool {
        self.destructible_walls.contains(&cell)
    }

    fn has_bomb(&self, cell: Cell) -> bool {
        self.bombs.iter().any(|bomb| bomb.cell == cell)
    }

    fn blocked(&self, cell: Cell) -> bool {
        Self::is_border(cell)
            || self.has_indestructible_wall(cell)
            || self.has_wall(cell)
            || self.has_bomb(cell)
    }

    fn target(cell: Cell, direction: Direction) -> Option<Cell> {
        let (dx, dy) = direction.delta();
        let x = i16::from(cell.x) + i16::from(dx);
        let y = i16::from(cell.y) + i16::from(dy);
        if x < 0 || y < 0 || x >= i16::from(ARENA_WIDTH) || y >= i16::from(ARENA_HEIGHT) {
            None
        } else {
            Some(Cell::new(x as u8, y as u8))
        }
    }

    fn apply_command(
        &mut self,
        command: &AuthoritativeCommand,
        events: &mut BoundedBuffer<PresentationEvent>,
    ) -> Result<Vec<AuthoritativeCommand>, CommandError> {
        match command {
            AuthoritativeCommand::Move { actor, from, to } => {
                let (alive, current_cell) = self
                    .player(*actor)
                    .map(|player| (player.alive, player.cell))
                    .ok_or(CommandError::UnknownPlayer(*actor))?;
                if !alive || current_cell != *from || self.blocked(*to) {
                    return Err(CommandError::InvalidMove(*actor));
                }
                self.player_mut(*actor)
                    .expect("player was checked above")
                    .cell = *to;
                Ok(Vec::new())
            }
            AuthoritativeCommand::PlaceBomb {
                actor,
                bomb,
                cell,
                fuse_ticks,
            } => {
                let player = self
                    .player(*actor)
                    .ok_or(CommandError::UnknownPlayer(*actor))?;
                if !player.alive || player.cell != *cell || self.has_bomb(*cell) {
                    return Err(CommandError::InvalidBombPlacement(*actor));
                }
                if self.bombs.iter().any(|existing| existing.owner == *actor) {
                    return Err(CommandError::BombLimit(*actor));
                }
                if *bomb != self.next_bomb_id {
                    return Err(CommandError::UnexpectedBombId(*bomb));
                }
                self.bombs.push(BombState {
                    id: *bomb,
                    cell: *cell,
                    owner: *actor,
                    fuse_ticks: *fuse_ticks,
                });
                self.bombs.sort_by_key(|existing| existing.id);
                self.next_bomb_id += 1;
                events
                    .push(PresentationEvent::BombPlaced {
                        bomb: *bomb,
                        cell: *cell,
                    })
                    .map_err(|_| CommandError::EventLimit)?;
                Ok(Vec::new())
            }
            AuthoritativeCommand::DetonateBomb { bomb, cell } => {
                let index = self
                    .bombs
                    .iter()
                    .position(|existing| existing.id == *bomb && existing.cell == *cell)
                    .ok_or(CommandError::UnknownBomb(*bomb))?;
                self.bombs.remove(index);
                events
                    .push(PresentationEvent::BombExploded {
                        bomb: *bomb,
                        cell: *cell,
                    })
                    .map_err(|_| CommandError::EventLimit)?;

                let mut generated = Vec::new();
                for blast_cell in self.blast_cells(*cell) {
                    if self.has_wall(blast_cell) {
                        generated.push(AuthoritativeCommand::DestroyWall { cell: blast_cell });
                        break;
                    }
                    for player in &self.players {
                        if player.alive && player.cell == blast_cell {
                            generated.push(AuthoritativeCommand::EliminatePlayer {
                                actor: player.actor,
                            });
                        }
                    }
                }
                Ok(generated)
            }
            AuthoritativeCommand::DestroyWall { cell } => {
                let Some(index) = self.destructible_walls.iter().position(|wall| wall == cell)
                else {
                    return Err(CommandError::UnknownWall(*cell));
                };
                self.destructible_walls.remove(index);
                events
                    .push(PresentationEvent::WallDestroyed { cell: *cell })
                    .map_err(|_| CommandError::EventLimit)?;
                Ok(Vec::new())
            }
            AuthoritativeCommand::EliminatePlayer { actor } => {
                let player = self
                    .player_mut(*actor)
                    .ok_or(CommandError::UnknownPlayer(*actor))?;
                if player.alive {
                    player.alive = false;
                    events
                        .push(PresentationEvent::PlayerDefeated {
                            actor: *actor,
                            cell: player.cell,
                        })
                        .map_err(|_| CommandError::EventLimit)?;
                }
                Ok(Vec::new())
            }
        }
    }

    fn blast_cells(&self, origin: Cell) -> Vec<Cell> {
        let mut cells = vec![origin];
        for direction in [
            Direction::Up,
            Direction::Right,
            Direction::Down,
            Direction::Left,
        ] {
            let mut current = origin;
            for _ in 0..BOMB_RADIUS {
                let Some(next) = Self::target(current, direction) else {
                    break;
                };
                if Self::is_border(next) {
                    break;
                }
                cells.push(next);
                if self.has_indestructible_wall(next) || self.has_wall(next) {
                    break;
                }
                current = next;
            }
        }
        cells
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum CommandError {
    UnknownPlayer(u8),
    UnknownBomb(u32),
    UnknownWall(Cell),
    InvalidMove(u8),
    InvalidBombPlacement(u8),
    BombLimit(u8),
    UnexpectedBombId(u32),
    EventLimit,
}

impl fmt::Display for CommandError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "authoritative command failed: {self:?}")
    }
}

impl std::error::Error for CommandError {}

#[derive(Debug)]
pub enum SimulationError {
    Protocol(ProtocolError),
    TickMismatch { expected: u64, received: u64 },
    Pipe(BufferError),
    Command(CommandError),
    CommandLimit,
    RejectionLimit,
}

impl fmt::Display for SimulationError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Protocol(error) => error.fmt(formatter),
            Self::TickMismatch { expected, received } => {
                write!(
                    formatter,
                    "expected tick {expected}, received tick {received}"
                )
            }
            Self::Pipe(error) => error.fmt(formatter),
            Self::Command(error) => error.fmt(formatter),
            Self::CommandLimit => formatter.write_str("authoritative command limit exceeded"),
            Self::RejectionLimit => formatter.write_str("rejection limit exceeded"),
        }
    }
}

impl std::error::Error for SimulationError {}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct TickResult {
    pub tick: u64,
    pub commands: Vec<AuthoritativeCommand>,
    pub events: Vec<PresentationEvent>,
    pub rejections: Vec<Rejection>,
    pub state_hash: StateHash,
}

pub struct WorldView<'a> {
    state: &'a GameState,
}

impl<'a> WorldView<'a> {
    pub fn new(state: &'a GameState) -> Self {
        Self { state }
    }

    pub fn tick(&self) -> u64 {
        self.state.tick
    }

    pub fn player(&self, actor: u8) -> Option<&PlayerState> {
        self.state.player(actor)
    }
}

pub struct Simulation {
    pub state: GameState,
    interpreter: RustGameplayInterpreter,
}

impl Default for Simulation {
    fn default() -> Self {
        Self::new()
    }
}

impl Simulation {
    pub fn new() -> Self {
        Self {
            state: GameState::initial(),
            interpreter: RustGameplayInterpreter,
        }
    }

    pub fn from_map(document: &ArenaMapDocument) -> Result<Self, MapError> {
        Ok(Self {
            state: GameState::from_map(document)?,
            interpreter: RustGameplayInterpreter,
        })
    }

    pub fn step(&mut self, input: &TickInputFrame) -> Result<TickResult, SimulationError> {
        if input.tick != self.state.tick {
            return Err(SimulationError::TickMismatch {
                expected: self.state.tick,
                received: input.tick,
            });
        }

        let mut intents = BoundedBuffer::new(MAX_INTENTS);
        let interpreter_input = InterpreterInput {
            view: WorldView::new(&self.state),
            frame: input,
        };
        self.interpreter
            .run(&interpreter_input, &mut intents)
            .map_err(SimulationError::Pipe)?;
        let mut commands = BoundedBuffer::new(MAX_COMMANDS);
        let mut events = BoundedBuffer::new(MAX_EVENTS);
        let mut rejections = BoundedBuffer::new(MAX_REJECTIONS);

        for intent in intents.into_inner() {
            let actor = match &intent {
                Intent::Move { actor, .. } | Intent::PlaceBomb { actor } => *actor,
            };
            let command = match self.validate_intent(&intent) {
                Ok(command) => command,
                Err(reason) => {
                    rejections
                        .push(Rejection {
                            actor,
                            intent,
                            reason,
                        })
                        .map_err(|_| SimulationError::RejectionLimit)?;
                    continue;
                }
            };
            apply_and_record(&mut self.state, command, &mut commands, &mut events)?;
        }

        let expired_bombs = {
            for bomb in &mut self.state.bombs {
                bomb.fuse_ticks = bomb.fuse_ticks.saturating_sub(1);
            }
            self.state
                .bombs
                .iter()
                .filter(|bomb| bomb.fuse_ticks == 0)
                .map(|bomb| (bomb.id, bomb.cell))
                .collect::<Vec<_>>()
        };
        for (bomb, cell) in expired_bombs {
            apply_and_record(
                &mut self.state,
                AuthoritativeCommand::DetonateBomb { bomb, cell },
                &mut commands,
                &mut events,
            )?;
        }

        let tick = self.state.tick;
        self.state.tick += 1;
        Ok(TickResult {
            tick,
            commands: commands.into_inner(),
            events: events.into_inner(),
            rejections: rejections.into_inner(),
            state_hash: self.state.hash(),
        })
    }

    fn validate_intent(&self, intent: &Intent) -> Result<AuthoritativeCommand, RejectionReason> {
        match intent {
            Intent::Move { actor, direction } => {
                let player = self.player(*actor)?;
                if !player.alive {
                    return Err(RejectionReason::PlayerDefeated);
                }
                let Some(to) = GameState::target(player.cell, *direction) else {
                    return Err(RejectionReason::BlockedCell);
                };
                if self.state.blocked(to) {
                    return Err(RejectionReason::BlockedCell);
                }
                Ok(AuthoritativeCommand::Move {
                    actor: *actor,
                    from: player.cell,
                    to,
                })
            }
            Intent::PlaceBomb { actor } => {
                let player = self.player(*actor)?;
                if !player.alive {
                    return Err(RejectionReason::PlayerDefeated);
                }
                if self.state.has_bomb(player.cell) {
                    return Err(RejectionReason::BombAlreadyPresent);
                }
                if self.state.bombs.iter().any(|bomb| bomb.owner == *actor) {
                    return Err(RejectionReason::BombLimit);
                }
                Ok(AuthoritativeCommand::PlaceBomb {
                    actor: *actor,
                    bomb: self.state.next_bomb_id,
                    cell: player.cell,
                    fuse_ticks: BOMB_FUSE_TICKS,
                })
            }
        }
    }

    fn player(&self, actor: u8) -> Result<&PlayerState, RejectionReason> {
        self.state
            .player(actor)
            .ok_or(RejectionReason::UnknownPlayer)
    }

    pub fn run_hashes(
        &mut self,
        frames: &[TickInputFrame],
    ) -> Result<Vec<StateHash>, SimulationError> {
        let mut hashes = Vec::with_capacity(frames.len());
        for frame in frames {
            hashes.push(self.step(frame)?.state_hash);
        }
        Ok(hashes)
    }
}

fn apply_and_record(
    state: &mut GameState,
    command: AuthoritativeCommand,
    commands: &mut BoundedBuffer<AuthoritativeCommand>,
    events: &mut BoundedBuffer<PresentationEvent>,
) -> Result<(), SimulationError> {
    let generated = state
        .apply_command(&command, events)
        .map_err(SimulationError::Command)?;
    commands
        .push(command)
        .map_err(|_| SimulationError::CommandLimit)?;
    for generated_command in generated {
        apply_and_record(state, generated_command, commands, events)?;
    }
    Ok(())
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Replay {
    pub protocol_version: u16,
    pub ruleset_id: &'static str,
    pub initial_state_hash: StateHash,
    pub frames: Vec<TickInputFrame>,
    pub state_hashes: Vec<StateHash>,
}

impl Replay {
    pub fn record(frames: &[TickInputFrame]) -> Result<Self, SimulationError> {
        let mut simulation = Simulation::new();
        let initial_state_hash = simulation.state.hash();
        let state_hashes = simulation.run_hashes(frames)?;
        Ok(Self {
            protocol_version: PROTOCOL_VERSION,
            ruleset_id: RULESET_ID,
            initial_state_hash,
            frames: frames.to_vec(),
            state_hashes,
        })
    }

    pub fn verify(&self) -> Result<(), ReplayError> {
        if self.protocol_version != PROTOCOL_VERSION || self.ruleset_id != RULESET_ID {
            return Err(ReplayError::Incompatible);
        }
        let mut simulation = Simulation::new();
        if simulation.state.hash() != self.initial_state_hash {
            return Err(ReplayError::InitialStateMismatch);
        }
        for (index, frame) in self.frames.iter().enumerate() {
            let result = simulation.step(frame).map_err(ReplayError::Simulation)?;
            if self.state_hashes.get(index) != Some(&result.state_hash) {
                return Err(ReplayError::Diverged {
                    frame: index,
                    expected: self.state_hashes.get(index).copied(),
                    actual: result.state_hash,
                });
            }
        }
        if self.state_hashes.len() != self.frames.len() {
            return Err(ReplayError::LengthMismatch);
        }
        Ok(())
    }
}

#[derive(Debug)]
pub enum ReplayError {
    Incompatible,
    InitialStateMismatch,
    Simulation(SimulationError),
    Diverged {
        frame: usize,
        expected: Option<StateHash>,
        actual: StateHash,
    },
    LengthMismatch,
}

impl fmt::Display for ReplayError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "replay verification failed: {self:?}")
    }
}

impl std::error::Error for ReplayError {}

pub fn scripted_frames() -> Vec<TickInputFrame> {
    (0..8)
        .map(|tick| {
            TickInputFrame::new(
                tick,
                vec![
                    PlayerInput {
                        player: 1,
                        direction: if tick < 2 {
                            Direction::Right
                        } else {
                            Direction::None
                        },
                        place_bomb: tick == 2,
                    },
                    PlayerInput {
                        player: 2,
                        direction: if tick < 2 {
                            Direction::Left
                        } else {
                            Direction::None
                        },
                        place_bomb: false,
                    },
                ],
            )
            .expect("scripted frame is valid")
        })
        .collect()
}

fn fnv1a64(bytes: &[u8]) -> StateHash {
    let mut hash = 0xcbf29ce484222325;
    for byte in bytes {
        hash ^= u64::from(*byte);
        hash = hash.wrapping_mul(0x100000001b3);
    }
    hash
}

struct Reader<'a> {
    bytes: &'a [u8],
    offset: usize,
}

impl<'a> Reader<'a> {
    fn new(bytes: &'a [u8]) -> Self {
        Self { bytes, offset: 0 }
    }

    fn take<const N: usize>(&mut self) -> Result<[u8; N], ProtocolError> {
        let end = self.offset + N;
        let Some(bytes) = self.bytes.get(self.offset..end) else {
            return Err(ProtocolError::UnexpectedEnd);
        };
        self.offset = end;
        bytes.try_into().map_err(|_| ProtocolError::UnexpectedEnd)
    }

    fn u8(&mut self) -> Result<u8, ProtocolError> {
        Ok(self.take::<1>()?[0])
    }

    fn u16(&mut self) -> Result<u16, ProtocolError> {
        Ok(u16::from_le_bytes(self.take::<2>()?))
    }

    fn u64(&mut self) -> Result<u64, ProtocolError> {
        Ok(u64::from_le_bytes(self.take::<8>()?))
    }

    fn remaining(&self) -> usize {
        self.bytes.len() - self.offset
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn stage1_map_fixture_loads_into_authoritative_state() {
        let document = ArenaMapDocument::load_json(include_str!(
            "../../../demo/pipe_arena/assets/maps/stage1.json"
        ))
        .expect("map JSON parses");
        let map = document.validate().expect("map validates");
        assert_eq!(map.players.len(), 2);
        assert_eq!(map.indestructible_walls.len(), 20);
        assert_eq!(map.destructible_walls.len(), 12);
        let simulation = Simulation::from_map(&document).expect("simulation loads map");
        assert_eq!(simulation.state.players, map.players);
    }

    #[test]
    fn map_loader_rejects_unknown_fields() {
        let error = ArenaMapDocument::load_json(
            r#"{
                "schema_version": 1,
                "ruleset_id": "pipe_arena_stage1",
                "width": 13,
                "height": 11,
                "spawns": [{"actor": 1, "x": 1, "y": 1}],
                "unexpected": true
            }"#,
        )
        .expect_err("unknown map field must be rejected");
        assert!(matches!(error, MapError::Json(_)));
    }

    #[test]
    fn input_frame_encoding_is_canonical() {
        let frame = TickInputFrame::new(
            4,
            vec![
                PlayerInput {
                    player: 2,
                    direction: Direction::Left,
                    place_bomb: false,
                },
                PlayerInput {
                    player: 1,
                    direction: Direction::Right,
                    place_bomb: true,
                },
            ],
        )
        .expect("valid input");
        let decoded = TickInputFrame::decode(&frame.encode()).expect("decode succeeds");
        assert_eq!(decoded.players[0].player, 1);
        assert_eq!(decoded, frame);
    }

    #[test]
    fn repeated_runs_produce_identical_hashes() {
        let frames = scripted_frames();
        let first = Simulation::new().run_hashes(&frames).expect("first run");
        let second = Simulation::new().run_hashes(&frames).expect("second run");
        assert_eq!(first, second);
    }

    #[test]
    fn blocked_move_is_rejected_without_position_mutation() {
        let frame = TickInputFrame::new(
            0,
            vec![PlayerInput {
                player: 1,
                direction: Direction::Up,
                place_bomb: false,
            }],
        )
        .expect("valid input");
        let mut simulation = Simulation::new();
        let result = simulation.step(&frame).expect("step succeeds");
        assert!(result.commands.is_empty());
        assert_eq!(result.rejections.len(), 1);
        assert_eq!(
            simulation.state.player(1).expect("player exists").cell,
            Cell::new(1, 1)
        );
    }

    #[test]
    fn bomb_expires_and_emits_events() {
        let mut simulation = Simulation::new();
        let frames = (0..BOMB_FUSE_TICKS)
            .map(|tick| {
                TickInputFrame::new(
                    u64::from(tick),
                    vec![PlayerInput {
                        player: 1,
                        direction: Direction::None,
                        place_bomb: tick == 0,
                    }],
                )
                .expect("valid input")
            })
            .collect::<Vec<_>>();
        let results = frames
            .iter()
            .map(|frame| simulation.step(frame).expect("step succeeds"))
            .collect::<Vec<_>>();
        assert!(
            results[0]
                .events
                .iter()
                .any(|event| matches!(event, PresentationEvent::BombPlaced { .. }))
        );
        assert!(
            results
                .iter()
                .flat_map(|result| &result.events)
                .any(|event| matches!(event, PresentationEvent::BombExploded { .. }))
        );
        assert!(simulation.state.bombs.is_empty());
    }

    #[test]
    fn replay_reproduces_hashes() {
        let replay = Replay::record(&scripted_frames()).expect("record succeeds");
        replay.verify().expect("replay verifies");
    }
}
