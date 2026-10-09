use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::{collections::BTreeSet, fmt, fmt::Write as _};

pub const BOMBERMAN_SCHEMA_VERSION: u16 = 1;
pub const BOMBERMAN_RULESET_ID: &str = "bomberman_stage1";

#[derive(Clone, Copy, Debug)]
pub struct BombermanCompileOptions<'a> {
    pub expected_width: u16,
    pub expected_height: u16,
    pub max_players: usize,
    pub ruleset_id: &'a str,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct BombermanSource {
    pub schema_version: u16,
    pub ruleset_id: String,
    pub arena: ArenaSource,
    pub rules: RulesSource,
    pub players: Vec<PlayerSource>,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct ArenaSource {
    pub width: u16,
    pub height: u16,
    #[serde(default)]
    pub indestructible_walls: Vec<CellSource>,
    #[serde(default)]
    pub destructible_walls: Vec<CellSource>,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct RulesSource {
    pub tick_hz: u16,
    pub bomb_fuse_ticks: u16,
    pub bomb_radius: u8,
    pub max_bombs_per_player: u8,
    pub random_bomb_batch_size: u16,
    pub random_bomb_interval_ticks: u16,
    pub random_bomb_seed: u64,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct PlayerSource {
    pub actor: u8,
    pub spawn: CellSource,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, Ord, PartialEq, PartialOrd, Serialize)]
#[serde(deny_unknown_fields)]
pub struct CellSource {
    pub x: u16,
    pub y: u16,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct BombermanIr {
    pub schema_version: u16,
    pub ruleset_id: String,
    pub arena: ArenaIr,
    pub rules: RulesIr,
    pub players: Vec<PlayerIr>,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct ArenaIr {
    pub width: u16,
    pub height: u16,
    pub indestructible_walls: Vec<CellSource>,
    pub destructible_walls: Vec<CellSource>,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct RulesIr {
    pub tick_hz: u16,
    pub bomb_fuse_ticks: u16,
    pub bomb_radius: u8,
    pub max_bombs_per_player: u8,
    pub random_bomb_batch_size: u16,
    pub random_bomb_interval_ticks: u16,
    pub random_bomb_seed: u64,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct PlayerIr {
    pub actor: u8,
    pub spawn: CellSource,
}

pub const VILLAGE_SCHEMA_VERSION: u16 = 2;
pub const VILLAGE_RULESET_ID: &str = "village_traffic_stage1";

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct VillageSource {
    pub schema_version: u16,
    pub ruleset_id: String,
    pub world: VillageWorldSource,
    pub traffic: VillageTrafficSource,
    pub agents: Vec<VillageAgentSource>,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct VillageWorldSource {
    pub width_m: u32,
    pub height_m: u32,
    pub origin: String,
    pub street_rendering: String,
    pub curve_types: Vec<String>,
    pub street_generation: VillageStreetGenerationSource,
    pub streets: Vec<VillageStreetSource>,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct VillageStreetGenerationSource {
    pub seed: u64,
    pub candidate_count: u16,
    pub width_m: u16,
    pub min_length_m: u32,
    pub max_length_m: u32,
    #[serde(default)]
    pub require_connections: bool,
    pub density_bands: Vec<VillageStreetDensityBandSource>,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct VillageStreetDensityBandSource {
    pub radius_m: u32,
    pub probability_percent: u8,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct VillageStreetSource {
    pub role: String,
    pub id: String,
    pub direction: String,
    pub width_m: u16,
    pub path: VillagePathSource,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct VillagePathSource {
    pub start: VillagePointSource,
    pub curve: String,
    pub end: VillagePointSource,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, Ord, PartialEq, PartialOrd, Serialize)]
#[serde(deny_unknown_fields)]
pub struct VillagePointSource {
    pub x_m: u32,
    pub y_m: u32,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct VillageTrafficSource {
    pub intersections: Vec<VillageIntersectionSource>,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct VillageIntersectionSource {
    pub id: String,
    pub position: VillagePointSource,
    pub incoming_streets: Vec<String>,
    pub traffic_lights: Vec<VillageTrafficLightSource>,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct VillageTrafficLightSource {
    pub id: String,
    pub red: VillageSignalRuleSource,
    pub green: VillageSignalRuleSource,
}

#[derive(Clone, Debug, Default, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct VillageSignalRuleSource {
    #[serde(default)]
    pub stops: Vec<String>,
    #[serde(default)]
    pub gives_way_to: Vec<String>,
    #[serde(default)]
    pub permits: Vec<String>,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct VillageAgentSource {
    pub kind: String,
    pub emoji: String,
    #[serde(default)]
    pub follows_streets: bool,
    #[serde(default)]
    pub stops_at_red: Option<String>,
    #[serde(default)]
    pub crosses_at: Option<String>,
    #[serde(default)]
    pub priority_over: Vec<String>,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct VillageIr {
    pub schema_version: u16,
    pub ruleset_id: String,
    pub world: VillageWorldSource,
    pub traffic: VillageTrafficSource,
    pub agents: Vec<VillageAgentSource>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CompiledVillage {
    pub ir: VillageIr,
    pub canonical_bytes: Vec<u8>,
    pub definition_hash: [u8; 32],
}

#[derive(Clone, Copy, Debug)]
pub struct VillageCompileOptions<'a> {
    pub ruleset_id: &'a str,
    pub expected_width_m: u32,
    pub expected_height_m: u32,
    pub expected_outgoing_streets: usize,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CompiledBomberman {
    pub ir: BombermanIr,
    pub canonical_bytes: Vec<u8>,
    pub definition_hash: [u8; 32],
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum CompileError {
    Json(String),
    UnsupportedSchema(u16),
    RulesetMismatch {
        expected: String,
        actual: String,
    },
    ArenaSize {
        expected_width: u16,
        expected_height: u16,
        actual_width: u16,
        actual_height: u16,
    },
    InvalidRules,
    NoPlayers,
    TooManyPlayers(usize),
    InvalidActor(u8),
    DuplicateActor(u8),
    OutOfBounds(CellSource),
    DuplicateCell(CellSource),
    CanonicalJson(String),
    GameLang(String),
    Village(String),
}

impl fmt::Display for CompileError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "game description compilation failed: {self:?}")
    }
}

impl std::error::Error for CompileError {}

pub fn parse_bomberman(json: &str) -> Result<BombermanSource, CompileError> {
    serde_json::from_str(json).map_err(|error| CompileError::Json(error.to_string()))
}

pub fn parse_village(json: &str) -> Result<VillageSource, CompileError> {
    serde_json::from_str(json).map_err(|error| CompileError::Village(error.to_string()))
}

pub fn compile_village(
    mut source: VillageSource,
    options: VillageCompileOptions<'_>,
) -> Result<CompiledVillage, CompileError> {
    if source.schema_version != VILLAGE_SCHEMA_VERSION {
        return Err(village_error(format!(
            "unsupported village schema {}",
            source.schema_version
        )));
    }
    if source.ruleset_id != options.ruleset_id {
        return Err(village_error(format!(
            "ruleset mismatch: expected {}, got {}",
            options.ruleset_id, source.ruleset_id
        )));
    }
    if source.world.width_m != options.expected_width_m
        || source.world.height_m != options.expected_height_m
    {
        return Err(village_error(format!(
            "world size mismatch: expected {}m by {}m, got {}m by {}m",
            options.expected_width_m,
            options.expected_height_m,
            source.world.width_m,
            source.world.height_m
        )));
    }
    if source.world.origin != "center" {
        return Err(village_error("world origin must be center"));
    }
    if source.world.street_rendering != "lines" {
        return Err(village_error("street_rendering must be lines"));
    }
    let outgoing_count = source
        .world
        .streets
        .iter()
        .filter(|street| street.role == "outgoing")
        .count();
    if outgoing_count != options.expected_outgoing_streets {
        return Err(village_error(format!(
            "expected {} outgoing streets, got {}",
            options.expected_outgoing_streets, outgoing_count
        )));
    }

    let mut density_bands = source.world.street_generation.density_bands.clone();
    density_bands.sort_by_key(|band| band.radius_m);
    if density_bands.is_empty() {
        return Err(village_error("street density requires at least one band"));
    }
    if source.world.street_generation.candidate_count == 0 {
        return Err(village_error(
            "street density candidate_count must be positive",
        ));
    }
    if source.world.street_generation.width_m == 0 {
        return Err(village_error("random street width must be positive"));
    }
    if source.world.street_generation.min_length_m == 0
        || source.world.street_generation.max_length_m < source.world.street_generation.min_length_m
    {
        return Err(village_error(
            "random street lengths must be positive and ordered",
        ));
    }
    for pair in density_bands.windows(2) {
        if pair[0].radius_m == pair[1].radius_m {
            return Err(village_error("street density radii must be unique"));
        }
        if pair[0].probability_percent < pair[1].probability_percent {
            return Err(village_error(
                "street density must decrease away from the center",
            ));
        }
    }
    source.world.street_generation.density_bands = density_bands;

    let allowed_curves = ["gentle_arc", "hairpin", "s_bend", "straight"];
    let expected_curves = allowed_curves
        .iter()
        .map(|curve| (*curve).to_owned())
        .collect::<Vec<_>>();
    let mut curve_types = source.world.curve_types.clone();
    curve_types.sort();
    if curve_types != expected_curves {
        return Err(village_error(format!(
            "curve_types must be exactly {:?}",
            allowed_curves
        )));
    }

    source
        .world
        .streets
        .extend(generate_random_streets(&source.world, &curve_types)?);
    if source.world.street_generation.require_connections {
        validate_street_connections(&source.world.streets)?;
    }

    let center = VillagePointSource {
        x_m: source.world.width_m / 2,
        y_m: source.world.height_m / 2,
    };
    unique_names(
        source.world.streets.iter().map(|street| street.id.as_str()),
        "street",
    )?;
    let street_ids = unique_names(
        source
            .world
            .streets
            .iter()
            .filter(|street| street.role == "outgoing")
            .map(|street| street.id.as_str()),
        "outgoing street",
    )?;
    for street in &source.world.streets {
        if !["outgoing", "local"].contains(&street.role.as_str()) {
            return Err(village_error(format!(
                "street {} has unsupported role {}",
                street.id, street.role
            )));
        }
        if street.width_m == 0 {
            return Err(village_error(format!(
                "street {} has zero width",
                street.id
            )));
        }
        if street.direction.is_empty() {
            return Err(village_error(format!(
                "street {} has no direction",
                street.id
            )));
        }
        if street.role == "outgoing" && street.path.start != center {
            return Err(village_error(format!(
                "street {} must start at the village center",
                street.id
            )));
        }
        if street.path.start == street.path.end {
            return Err(village_error(format!(
                "street {} has an empty path",
                street.id
            )));
        }
        if !curve_types.iter().any(|curve| curve == &street.path.curve) {
            return Err(village_error(format!(
                "street {} uses unknown curve {}",
                street.id, street.path.curve
            )));
        }
        validate_point(street.path.end, source.world.width_m, source.world.height_m)?;
    }

    if source.traffic.intersections.is_empty() {
        return Err(village_error("at least one intersection is required"));
    }
    unique_names(
        source
            .traffic
            .intersections
            .iter()
            .map(|intersection| intersection.id.as_str()),
        "intersection",
    )?;
    let mut traffic_light_ids = BTreeSet::new();
    for intersection in &source.traffic.intersections {
        validate_point(
            intersection.position,
            source.world.width_m,
            source.world.height_m,
        )?;
        let incoming_ids = unique_names(
            intersection.incoming_streets.iter().map(String::as_str),
            "incoming street",
        )?;
        if incoming_ids != street_ids {
            return Err(village_error(format!(
                "intersection {} must reference every outgoing street exactly once",
                intersection.id
            )));
        }
        for light in &intersection.traffic_lights {
            if light.id.is_empty() {
                return Err(village_error("traffic light id cannot be empty"));
            }
            if !traffic_light_ids.insert(light.id.clone()) {
                return Err(village_error(format!(
                    "duplicate traffic light {}",
                    light.id
                )));
            }
            validate_signal_values(&light.red, &light.id)?;
            validate_signal_values(&light.green, &light.id)?;
            for required in ["car", "bicycle", "motorcycle", "moped", "escooter"] {
                require_signal_value(&light.red.stops, required, &light.id, "red stops")?;
            }
            require_signal_value(
                &light.red.gives_way_to,
                "pedestrian",
                &light.id,
                "red gives_way_to",
            )?;
        }
    }

    let street_agent_kinds = ["car", "bicycle", "motorcycle", "moped", "escooter"];
    let required_agent_kinds = [
        "car",
        "bicycle",
        "motorcycle",
        "moped",
        "escooter",
        "pedestrian",
    ];
    let agent_kinds = source
        .agents
        .iter()
        .map(|agent| agent.kind.as_str())
        .collect::<BTreeSet<_>>();
    for required in required_agent_kinds {
        if !agent_kinds.contains(required) {
            return Err(village_error(format!("missing {required} agent")));
        }
    }
    for agent in &source.agents {
        if !required_agent_kinds.contains(&agent.kind.as_str()) {
            return Err(village_error(format!(
                "unsupported agent kind {}",
                agent.kind
            )));
        }
        if agent.emoji.is_empty() {
            return Err(village_error(format!("agent {} has no emoji", agent.kind)));
        }
        if street_agent_kinds.contains(&agent.kind.as_str()) {
            if !agent.follows_streets {
                return Err(village_error(format!(
                    "agent {} must follow streets",
                    agent.kind
                )));
            }
            let Some(light_id) = &agent.stops_at_red else {
                return Err(village_error(format!(
                    "agent {} must stop at a red traffic light",
                    agent.kind
                )));
            };
            if !traffic_light_ids.contains(light_id) {
                return Err(village_error(format!(
                    "agent {} references unknown traffic light {}",
                    agent.kind, light_id
                )));
            }
        }
        if agent.kind == "pedestrian" {
            let Some(light_id) = &agent.crosses_at else {
                return Err(village_error("pedestrian must cross at a traffic light"));
            };
            if !traffic_light_ids.contains(light_id) {
                return Err(village_error(format!(
                    "pedestrian references unknown traffic light {}",
                    light_id
                )));
            }
            for required in street_agent_kinds {
                if !agent.priority_over.iter().any(|kind| kind == required) {
                    return Err(village_error(format!(
                        "pedestrian must have priority over {required}"
                    )));
                }
            }
        }
    }

    source.world.curve_types.sort();
    source
        .world
        .streets
        .sort_by(|left, right| left.id.cmp(&right.id));
    for intersection in &mut source.traffic.intersections {
        intersection.incoming_streets.sort();
        intersection
            .traffic_lights
            .sort_by(|left, right| left.id.cmp(&right.id));
        for light in &mut intersection.traffic_lights {
            normalize_signal(&mut light.red);
            normalize_signal(&mut light.green);
        }
    }
    source
        .traffic
        .intersections
        .sort_by(|left, right| left.id.cmp(&right.id));
    for agent in &mut source.agents {
        agent.priority_over.sort();
    }
    source
        .agents
        .sort_by(|left, right| left.kind.cmp(&right.kind));

    let ir = VillageIr {
        schema_version: source.schema_version,
        ruleset_id: source.ruleset_id,
        world: source.world,
        traffic: source.traffic,
        agents: source.agents,
    };
    let canonical_bytes =
        serde_json::to_vec(&ir).map_err(|error| CompileError::Village(error.to_string()))?;
    let definition_hash: [u8; 32] = Sha256::digest(&canonical_bytes).into();
    Ok(CompiledVillage {
        ir,
        canonical_bytes,
        definition_hash,
    })
}
fn validate_street_connections(streets: &[VillageStreetSource]) -> Result<(), CompileError> {
    for (index, street) in streets.iter().enumerate() {
        let connected = streets.iter().enumerate().any(|(other_index, other)| {
            index != other_index
                && (street.path.start == other.path.start
                    || street.path.start == other.path.end
                    || street.path.end == other.path.start
                    || street.path.end == other.path.end)
        });
        if !connected {
            return Err(village_error(format!(
                "street {} has no connection to another street",
                street.id
            )));
        }
    }
    Ok(())
}

fn generate_random_streets(
    world: &VillageWorldSource,
    curve_types: &[String],
) -> Result<Vec<VillageStreetSource>, CompileError> {
    let generation = &world.street_generation;
    let max_radius = generation
        .density_bands
        .last()
        .map(|band| band.radius_m)
        .ok_or_else(|| village_error("street density requires at least one band"))?;
    let mut random = VillageRandom::new(generation.seed);
    let mut streets = Vec::new();
    let mut connection_points = world
        .streets
        .iter()
        .flat_map(|street| [street.path.start, street.path.end])
        .collect::<Vec<_>>();

    for index in 0..generation.candidate_count {
        let offset_x = random.signed(max_radius);
        let offset_y = random.signed(max_radius);
        let radius = offset_x.unsigned_abs().max(offset_y.unsigned_abs()) as u32;
        let probability = density_for_radius(&generation.density_bands, radius);
        if random.next_u32() % 100 >= u32::from(probability) {
            continue;
        }

        let start = connection_points[random.next_bounded(connection_points.len() as u32) as usize];
        let length_span = generation
            .max_length_m
            .saturating_sub(generation.min_length_m)
            .saturating_add(1);
        let length = generation
            .min_length_m
            .saturating_add(random.next_bounded(length_span));
        let end = VillagePointSource {
            x_m: clamp_coordinate(i64::from(start.x_m) + random.signed(length), world.width_m),
            y_m: clamp_coordinate(i64::from(start.y_m) + random.signed(length), world.height_m),
        };
        if start == end {
            continue;
        }

        let curve = curve_types[random.next_bounded(curve_types.len() as u32) as usize].clone();
        streets.push(VillageStreetSource {
            role: "local".to_owned(),
            id: format!("random_street_{index:03}"),
            direction: "local".to_owned(),
            width_m: generation.width_m,
            path: VillagePathSource { start, curve, end },
        });
        connection_points.push(end);
    }

    Ok(streets)
}

fn density_for_radius(bands: &[VillageStreetDensityBandSource], radius_m: u32) -> u8 {
    bands
        .iter()
        .find(|band| radius_m <= band.radius_m)
        .map(|band| band.probability_percent)
        .unwrap_or_else(|| bands.last().unwrap().probability_percent)
}

fn clamp_coordinate(value: i64, maximum: u32) -> u32 {
    value.clamp(0, i64::from(maximum)) as u32
}

struct VillageRandom {
    state: u64,
}

impl VillageRandom {
    fn new(seed: u64) -> Self {
        Self {
            state: if seed == 0 {
                0x9e37_79b9_7f4a_7c15
            } else {
                seed
            },
        }
    }

    fn next_u64(&mut self) -> u64 {
        self.state ^= self.state << 7;
        self.state ^= self.state >> 9;
        self.state
    }

    fn next_u32(&mut self) -> u32 {
        self.next_u64() as u32
    }

    fn next_bounded(&mut self, exclusive_maximum: u32) -> u32 {
        if exclusive_maximum == 0 {
            0
        } else {
            self.next_u32() % exclusive_maximum
        }
    }

    fn signed(&mut self, magnitude: u32) -> i64 {
        let span = magnitude.saturating_mul(2).saturating_add(1);
        i64::from(self.next_bounded(span)) - i64::from(magnitude)
    }
}

fn village_error(message: impl Into<String>) -> CompileError {
    CompileError::Village(message.into())
}

fn unique_names<'a>(
    names: impl Iterator<Item = &'a str>,
    kind: &str,
) -> Result<BTreeSet<String>, CompileError> {
    let mut unique = BTreeSet::new();
    for name in names {
        if name.is_empty() {
            return Err(village_error(format!("{kind} id cannot be empty")));
        }
        if !unique.insert(name.to_owned()) {
            return Err(village_error(format!("duplicate {kind} id {name}")));
        }
    }
    Ok(unique)
}

fn validate_point(
    point: VillagePointSource,
    width_m: u32,
    height_m: u32,
) -> Result<(), CompileError> {
    if point.x_m > width_m || point.y_m > height_m {
        return Err(village_error(format!(
            "point ({}, {}) is outside {}m by {}m village",
            point.x_m, point.y_m, width_m, height_m
        )));
    }
    Ok(())
}

fn validate_signal_values(
    signal: &VillageSignalRuleSource,
    light_id: &str,
) -> Result<(), CompileError> {
    for (field, values) in [
        ("stops", &signal.stops),
        ("gives_way_to", &signal.gives_way_to),
        ("permits", &signal.permits),
    ] {
        if let Some(value) = values.iter().find(|value| {
            ![
                "car",
                "bicycle",
                "motorcycle",
                "moped",
                "escooter",
                "pedestrian",
            ]
            .contains(&value.as_str())
        }) {
            return Err(village_error(format!(
                "traffic light {light_id} {field} has unsupported value {value}"
            )));
        }
    }
    Ok(())
}

fn require_signal_value(
    values: &[String],
    required: &str,
    light_id: &str,
    field: &str,
) -> Result<(), CompileError> {
    if values.iter().any(|value| value == required) {
        Ok(())
    } else {
        Err(village_error(format!(
            "traffic light {light_id} {field} must contain {required}"
        )))
    }
}

fn normalize_signal(signal: &mut VillageSignalRuleSource) {
    signal.stops.sort();
    signal.stops.dedup();
    signal.gives_way_to.sort();
    signal.gives_way_to.dedup();
    signal.permits.sort();
    signal.permits.dedup();
}

/// Parse the readable `.game` source language used by the Bevy engine adapter.
///
/// The parser deliberately produces the same typed source model as JSON. That
/// keeps compilation, normalization, hashing, and generated runtime data on
/// one path regardless of the authoring syntax.
pub fn parse_bomberman_lang(source: &str) -> Result<BombermanSource, CompileError> {
    GameParser::new(source)?.parse()
}

#[derive(Clone, Debug, Eq, PartialEq)]
enum GameTokenKind {
    Identifier(String),
    Number(u64),
    LeftBrace,
    RightBrace,
    LeftParen,
    RightParen,
    Comma,
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct GameToken {
    kind: GameTokenKind,
    line: usize,
    column: usize,
}

fn lex_game(source: &str) -> Result<Vec<GameToken>, CompileError> {
    let bytes = source.as_bytes();
    let mut tokens = Vec::new();
    let mut index = 0;
    let mut line = 1;
    let mut column = 1;

    while index < bytes.len() {
        let byte = bytes[index];
        if byte.is_ascii_whitespace() {
            if byte == b'\n' {
                line += 1;
                column = 1;
            } else {
                column += 1;
            }
            index += 1;
            continue;
        }
        if byte == b'#' || (byte == b'/' && bytes.get(index + 1) == Some(&b'/')) {
            while index < bytes.len() && bytes[index] != b'\n' {
                index += 1;
                column += 1;
            }
            continue;
        }

        let token_line = line;
        let token_column = column;
        let kind = match byte {
            b'{' => {
                index += 1;
                column += 1;
                GameTokenKind::LeftBrace
            }
            b'}' => {
                index += 1;
                column += 1;
                GameTokenKind::RightBrace
            }
            b'(' => {
                index += 1;
                column += 1;
                GameTokenKind::LeftParen
            }
            b')' => {
                index += 1;
                column += 1;
                GameTokenKind::RightParen
            }
            b',' => {
                index += 1;
                column += 1;
                GameTokenKind::Comma
            }
            b'0' if bytes.get(index + 1) == Some(&b'x') => {
                index += 2;
                column += 2;
                let digits_start = index;
                while index < bytes.len() && bytes[index].is_ascii_hexdigit() {
                    index += 1;
                    column += 1;
                }
                if digits_start == index {
                    return Err(game_error(
                        token_line,
                        token_column,
                        "hex number requires at least one digit",
                    ));
                }
                let value = u64::from_str_radix(&source[digits_start..index], 16)
                    .map_err(|_| game_error(token_line, token_column, "hex number is too large"))?;
                GameTokenKind::Number(value)
            }
            b'0'..=b'9' => {
                let digits_start = index;
                while index < bytes.len() && bytes[index].is_ascii_digit() {
                    index += 1;
                    column += 1;
                }
                let value = source[digits_start..index]
                    .parse::<u64>()
                    .map_err(|_| game_error(token_line, token_column, "number is too large"))?;
                GameTokenKind::Number(value)
            }
            b'a'..=b'z' | b'A'..=b'Z' | b'_' => {
                let ident_start = index;
                while index < bytes.len()
                    && (bytes[index].is_ascii_alphanumeric() || bytes[index] == b'_')
                {
                    index += 1;
                    column += 1;
                }
                GameTokenKind::Identifier(source[ident_start..index].to_owned())
            }
            _ => {
                return Err(game_error(token_line, token_column, "unexpected character"));
            }
        };
        tokens.push(GameToken {
            kind,
            line: token_line,
            column: token_column,
        });
    }
    Ok(tokens)
}

fn game_error(line: usize, column: usize, message: impl Into<String>) -> CompileError {
    CompileError::GameLang(format!("{line}:{column}: {}", message.into()))
}

struct GameParser {
    tokens: Vec<GameToken>,
    position: usize,
}

impl GameParser {
    fn new(source: &str) -> Result<Self, CompileError> {
        Ok(Self {
            tokens: lex_game(source)?,
            position: 0,
        })
    }

    fn parse(mut self) -> Result<BombermanSource, CompileError> {
        self.keyword("game")?;
        let ruleset_id = self.identifier("ruleset id")?;
        self.symbol(GameTokenKind::LeftBrace)?;
        self.keyword("schema")?;
        let schema_version = self.number_u16("schema version")?;
        let arena = self.parse_arena()?;
        let rules = self.parse_rules()?;
        let mut players = Vec::new();
        while !self.take_symbol(GameTokenKind::RightBrace) {
            self.keyword("player")?;
            let actor = self.number_u8("player actor")?;
            self.keyword("at")?;
            players.push(PlayerSource {
                actor,
                spawn: self.parse_cell()?,
            });
        }
        if self.position != self.tokens.len() {
            return Err(self.error("unexpected content after game block"));
        }
        Ok(BombermanSource {
            schema_version,
            ruleset_id,
            arena,
            rules,
            players,
        })
    }

    fn parse_arena(&mut self) -> Result<ArenaSource, CompileError> {
        self.keyword("arena")?;
        self.symbol(GameTokenKind::LeftBrace)?;
        let mut width = None;
        let mut height = None;
        let mut indestructible_walls = None;
        let mut destructible_walls = None;
        while !self.take_symbol(GameTokenKind::RightBrace) {
            if self.take_keyword("size") {
                width = Some(self.number_u16("arena width")?);
                self.keyword("by")?;
                height = Some(self.number_u16("arena height")?);
            } else if self.take_keyword("indestructible_walls") {
                indestructible_walls = Some(self.parse_cells()?);
            } else if self.take_keyword("destructible_walls") {
                destructible_walls = Some(self.parse_cells()?);
            } else {
                return Err(
                    self.error("expected size, indestructible_walls, or destructible_walls")
                );
            }
        }
        Ok(ArenaSource {
            width: width.ok_or_else(|| self.missing("arena size"))?,
            height: height.ok_or_else(|| self.missing("arena size"))?,
            indestructible_walls: indestructible_walls
                .ok_or_else(|| self.missing("indestructible_walls"))?,
            destructible_walls: destructible_walls
                .ok_or_else(|| self.missing("destructible_walls"))?,
        })
    }

    fn parse_rules(&mut self) -> Result<RulesSource, CompileError> {
        self.keyword("rules")?;
        self.symbol(GameTokenKind::LeftBrace)?;
        let mut tick_hz = None;
        let mut bomb_fuse_ticks = None;
        let mut bomb_radius = None;
        let mut max_bombs_per_player = None;
        let mut random_bomb_batch_size = None;
        let mut random_bomb_interval_ticks = None;
        let mut random_bomb_seed = None;
        while !self.take_symbol(GameTokenKind::RightBrace) {
            if self.take_keyword("tick_rate") {
                tick_hz = Some(self.number_u16("tick rate")?);
                self.keyword("hz")?;
            } else if self.take_keyword("bomb") {
                self.symbol(GameTokenKind::LeftBrace)?;
                while !self.take_symbol(GameTokenKind::RightBrace) {
                    if self.take_keyword("fuse") {
                        bomb_fuse_ticks = Some(self.number_u16("bomb fuse")?);
                        self.keyword("ticks")?;
                    } else if self.take_keyword("radius") {
                        bomb_radius = Some(self.number_u8("bomb radius")?);
                        self.keyword("cells")?;
                    } else if self.take_keyword("max_per_player") {
                        max_bombs_per_player = Some(self.number_u8("bomb capacity")?);
                    } else {
                        return Err(self.error("expected fuse, radius, or max_per_player"));
                    }
                }
            } else if self.take_keyword("random_bombs") {
                self.symbol(GameTokenKind::LeftBrace)?;
                while !self.take_symbol(GameTokenKind::RightBrace) {
                    if self.take_keyword("batch") {
                        random_bomb_batch_size = Some(self.number_u16("random bomb batch")?);
                    } else if self.take_keyword("every") {
                        random_bomb_interval_ticks = Some(self.number_u16("random bomb interval")?);
                        self.keyword("ticks")?;
                    } else if self.take_keyword("seed") {
                        random_bomb_seed = Some(self.number_u64("random bomb seed")?);
                    } else {
                        return Err(self.error("expected batch, every, or seed"));
                    }
                }
            } else {
                return Err(self.error("expected tick_rate, bomb, or random_bombs"));
            }
        }
        Ok(RulesSource {
            tick_hz: tick_hz.ok_or_else(|| self.missing("tick_rate"))?,
            bomb_fuse_ticks: bomb_fuse_ticks.ok_or_else(|| self.missing("bomb.fuse"))?,
            bomb_radius: bomb_radius.ok_or_else(|| self.missing("bomb.radius"))?,
            max_bombs_per_player: max_bombs_per_player
                .ok_or_else(|| self.missing("bomb.max_per_player"))?,
            random_bomb_batch_size: random_bomb_batch_size
                .ok_or_else(|| self.missing("random_bombs.batch"))?,
            random_bomb_interval_ticks: random_bomb_interval_ticks
                .ok_or_else(|| self.missing("random_bombs.every"))?,
            random_bomb_seed: random_bomb_seed.ok_or_else(|| self.missing("random_bombs.seed"))?,
        })
    }

    fn parse_cells(&mut self) -> Result<Vec<CellSource>, CompileError> {
        self.symbol(GameTokenKind::LeftBrace)?;
        let mut cells = Vec::new();
        while !self.take_symbol(GameTokenKind::RightBrace) {
            cells.push(self.parse_cell()?);
            self.take_symbol(GameTokenKind::Comma);
        }
        Ok(cells)
    }

    fn parse_cell(&mut self) -> Result<CellSource, CompileError> {
        self.symbol(GameTokenKind::LeftParen)?;
        let x = self.number_u16("cell x")?;
        self.symbol(GameTokenKind::Comma)?;
        let y = self.number_u16("cell y")?;
        self.symbol(GameTokenKind::RightParen)?;
        Ok(CellSource { x, y })
    }

    fn number_u8(&mut self, name: &str) -> Result<u8, CompileError> {
        self.number(name)?
            .try_into()
            .map_err(|_| self.error_value(name))
    }

    fn number_u16(&mut self, name: &str) -> Result<u16, CompileError> {
        self.number(name)?
            .try_into()
            .map_err(|_| self.error_value(name))
    }

    fn number_u64(&mut self, name: &str) -> Result<u64, CompileError> {
        self.number(name)
    }

    fn number(&mut self, name: &str) -> Result<u64, CompileError> {
        match self.tokens.get(self.position) {
            Some(GameToken {
                kind: GameTokenKind::Number(value),
                ..
            }) => {
                self.position += 1;
                Ok(*value)
            }
            _ => Err(self.error_value(name)),
        }
    }

    fn identifier(&mut self, name: &str) -> Result<String, CompileError> {
        match self.tokens.get(self.position) {
            Some(GameToken {
                kind: GameTokenKind::Identifier(value),
                ..
            }) => {
                self.position += 1;
                Ok(value.clone())
            }
            _ => Err(self.error_value(name)),
        }
    }

    fn keyword(&mut self, keyword: &str) -> Result<(), CompileError> {
        if self.take_keyword(keyword) {
            Ok(())
        } else {
            Err(self.error_value(keyword))
        }
    }

    fn take_keyword(&mut self, keyword: &str) -> bool {
        if matches!(
            self.tokens.get(self.position),
            Some(GameToken {
                kind: GameTokenKind::Identifier(value),
                ..
            }) if value == keyword
        ) {
            self.position += 1;
            true
        } else {
            false
        }
    }

    fn symbol(&mut self, symbol: GameTokenKind) -> Result<(), CompileError> {
        if self.take_symbol(symbol) {
            Ok(())
        } else {
            Err(self.error_value("symbol"))
        }
    }

    fn take_symbol(&mut self, symbol: GameTokenKind) -> bool {
        if self.tokens.get(self.position).map(|token| &token.kind) == Some(&symbol) {
            self.position += 1;
            true
        } else {
            false
        }
    }

    fn error(&self, message: &str) -> CompileError {
        self.error_value(message)
    }

    fn missing(&self, name: &str) -> CompileError {
        self.error_value(&format!("missing {name}"))
    }

    fn error_value(&self, expected: &str) -> CompileError {
        match self.tokens.get(self.position) {
            Some(token) => game_error(token.line, token.column, format!("expected {expected}")),
            None => CompileError::GameLang(format!("end of file: expected {expected}")),
        }
    }
}

pub fn compile_bomberman(
    source: BombermanSource,
    options: BombermanCompileOptions<'_>,
) -> Result<CompiledBomberman, CompileError> {
    if source.schema_version != BOMBERMAN_SCHEMA_VERSION {
        return Err(CompileError::UnsupportedSchema(source.schema_version));
    }
    if source.ruleset_id != options.ruleset_id {
        return Err(CompileError::RulesetMismatch {
            expected: options.ruleset_id.to_owned(),
            actual: source.ruleset_id,
        });
    }
    if source.arena.width != options.expected_width
        || source.arena.height != options.expected_height
    {
        return Err(CompileError::ArenaSize {
            expected_width: options.expected_width,
            expected_height: options.expected_height,
            actual_width: source.arena.width,
            actual_height: source.arena.height,
        });
    }
    if source.rules.tick_hz == 0
        || source.rules.bomb_fuse_ticks == 0
        || source.rules.bomb_radius == 0
        || source.rules.max_bombs_per_player == 0
        || source.rules.random_bomb_batch_size == 0
        || source.rules.random_bomb_interval_ticks == 0
    {
        return Err(CompileError::InvalidRules);
    }
    if source.players.is_empty() {
        return Err(CompileError::NoPlayers);
    }
    if source.players.len() > options.max_players {
        return Err(CompileError::TooManyPlayers(source.players.len()));
    }

    let arena = ArenaIr {
        width: source.arena.width,
        height: source.arena.height,
        indestructible_walls: normalize_cells(
            source.arena.indestructible_walls,
            source.arena.width,
            source.arena.height,
        )?,
        destructible_walls: normalize_cells(
            source.arena.destructible_walls,
            source.arena.width,
            source.arena.height,
        )?,
    };

    let mut players = source
        .players
        .into_iter()
        .map(|player| {
            if player.actor == 0 {
                return Err(CompileError::InvalidActor(player.actor));
            }
            if player.spawn.x >= arena.width || player.spawn.y >= arena.height {
                return Err(CompileError::OutOfBounds(player.spawn));
            }
            Ok(PlayerIr {
                actor: player.actor,
                spawn: player.spawn,
            })
        })
        .collect::<Result<Vec<_>, _>>()?;
    players.sort_by_key(|player| player.actor);
    for pair in players.windows(2) {
        if pair[0].actor == pair[1].actor {
            return Err(CompileError::DuplicateActor(pair[0].actor));
        }
    }
    let mut spawn_cells = players
        .iter()
        .map(|player| player.spawn)
        .collect::<Vec<_>>();
    spawn_cells.sort_unstable();
    for pair in spawn_cells.windows(2) {
        if pair[0] == pair[1] {
            return Err(CompileError::DuplicateCell(pair[0]));
        }
    }
    if arena
        .indestructible_walls
        .iter()
        .chain(&arena.destructible_walls)
        .any(|wall| players.iter().any(|player| player.spawn == *wall))
    {
        let cell = players
            .iter()
            .find_map(|player| {
                if arena.indestructible_walls.contains(&player.spawn)
                    || arena.destructible_walls.contains(&player.spawn)
                {
                    Some(player.spawn)
                } else {
                    None
                }
            })
            .expect("wall/spawn overlap was found");
        return Err(CompileError::DuplicateCell(cell));
    }

    let ir = BombermanIr {
        schema_version: source.schema_version,
        ruleset_id: source.ruleset_id,
        arena,
        rules: RulesIr {
            tick_hz: source.rules.tick_hz,
            bomb_fuse_ticks: source.rules.bomb_fuse_ticks,
            bomb_radius: source.rules.bomb_radius,
            max_bombs_per_player: source.rules.max_bombs_per_player,
            random_bomb_batch_size: source.rules.random_bomb_batch_size,
            random_bomb_interval_ticks: source.rules.random_bomb_interval_ticks,
            random_bomb_seed: source.rules.random_bomb_seed,
        },
        players,
    };
    let canonical_bytes =
        serde_json::to_vec(&ir).map_err(|error| CompileError::CanonicalJson(error.to_string()))?;
    let definition_hash: [u8; 32] = Sha256::digest(&canonical_bytes).into();
    Ok(CompiledBomberman {
        ir,
        canonical_bytes,
        definition_hash,
    })
}

fn normalize_cells(
    mut cells: Vec<CellSource>,
    width: u16,
    height: u16,
) -> Result<Vec<CellSource>, CompileError> {
    if let Some(cell) = cells
        .iter()
        .find(|cell| cell.x >= width || cell.y >= height)
    {
        return Err(CompileError::OutOfBounds(*cell));
    }
    cells.sort_unstable();
    if let Some(pair) = cells.windows(2).find(|pair| pair[0] == pair[1]) {
        return Err(CompileError::DuplicateCell(pair[0]));
    }
    Ok(cells)
}

pub fn generate_bomberman_rust(compiled: &CompiledBomberman) -> String {
    let ir = &compiled.ir;
    let mut output = String::new();
    writeln!(
        output,
        "pub const BOMBERMAN_SCHEMA_VERSION: u16 = {};",
        ir.schema_version
    )
    .unwrap();
    writeln!(
        output,
        "pub const BOMBERMAN_RULESET_ID: &str = {:?};",
        ir.ruleset_id
    )
    .unwrap();
    writeln!(
        output,
        "pub const BOMBERMAN_WIDTH: u16 = {};",
        ir.arena.width
    )
    .unwrap();
    writeln!(
        output,
        "pub const BOMBERMAN_HEIGHT: u16 = {};",
        ir.arena.height
    )
    .unwrap();
    writeln!(output, "pub static BOMBERMAN_DEFINITION: crate::BombermanDefinition = crate::BombermanDefinition {{").unwrap();
    writeln!(output, "    schema_version: {},", ir.schema_version).unwrap();
    writeln!(output, "    ruleset_id: {:?},", ir.ruleset_id).unwrap();
    writeln!(
        output,
        "    definition_hash: {:?},",
        compiled.definition_hash
    )
    .unwrap();
    writeln!(output, "    width: {},", ir.arena.width).unwrap();
    writeln!(output, "    height: {},", ir.arena.height).unwrap();
    write_cells(
        &mut output,
        "indestructible_walls",
        &ir.arena.indestructible_walls,
    );
    write_cells(
        &mut output,
        "destructible_walls",
        &ir.arena.destructible_walls,
    );
    writeln!(output, "    rules: crate::RulesetConfig {{").unwrap();
    writeln!(output, "        tick_hz: {},", ir.rules.tick_hz).unwrap();
    writeln!(
        output,
        "        bomb_fuse_ticks: {},",
        ir.rules.bomb_fuse_ticks
    )
    .unwrap();
    writeln!(output, "        bomb_radius: {},", ir.rules.bomb_radius).unwrap();
    writeln!(
        output,
        "        max_bombs_per_player: {},",
        ir.rules.max_bombs_per_player
    )
    .unwrap();
    writeln!(
        output,
        "        random_bomb_batch_size: {},",
        ir.rules.random_bomb_batch_size
    )
    .unwrap();
    writeln!(
        output,
        "        random_bomb_interval_ticks: {},",
        ir.rules.random_bomb_interval_ticks
    )
    .unwrap();
    writeln!(
        output,
        "        random_bomb_seed: {},",
        ir.rules.random_bomb_seed
    )
    .unwrap();
    writeln!(output, "    }},").unwrap();
    writeln!(output, "    players: &[").unwrap();
    for player in &ir.players {
        writeln!(output, "        crate::PlayerDescription {{ actor: {}, spawn: crate::MapCell {{ x: {}, y: {} }} }},", player.actor, player.spawn.x, player.spawn.y).unwrap();
    }
    writeln!(output, "    ],").unwrap();
    writeln!(output, "}};").unwrap();
    output
}

fn write_cells(output: &mut String, name: &str, cells: &[CellSource]) {
    writeln!(output, "    {name}: &[").unwrap();
    for cell in cells {
        writeln!(
            output,
            "        crate::MapCell {{ x: {}, y: {} }},",
            cell.x, cell.y
        )
        .unwrap();
    }
    writeln!(output, "    ],").unwrap();
}

#[cfg(test)]
mod tests {
    use super::*;

    const OPTIONS: BombermanCompileOptions<'static> = BombermanCompileOptions {
        expected_width: 13,
        expected_height: 11,
        max_players: 8,
        ruleset_id: BOMBERMAN_RULESET_ID,
    };
    const VILLAGE_OPTIONS: VillageCompileOptions<'static> = VillageCompileOptions {
        ruleset_id: VILLAGE_RULESET_ID,
        expected_width_m: 1_000,
        expected_height_m: 1_000,
        expected_outgoing_streets: 6,
    };

    #[test]
    fn canonicalization_sorts_unordered_cells_and_players() {
        let first = parse_bomberman(
            r#"{
                "schema_version": 1,
                "ruleset_id": "bomberman_stage1",
                "arena": {"width": 13, "height": 11, "indestructible_walls": [{"x": 2, "y": 2}, {"x": 1, "y": 2}], "destructible_walls": []},
                "rules": {"tick_hz": 64, "bomb_fuse_ticks": 3, "bomb_radius": 2, "max_bombs_per_player": 1, "random_bomb_batch_size": 100, "random_bomb_interval_ticks": 100, "random_bomb_seed": 305419896},
                "players": [{"actor": 2, "spawn": {"x": 11, "y": 9}}, {"actor": 1, "spawn": {"x": 1, "y": 1}}]
            }"#,
        )
        .unwrap();
        let second = parse_bomberman(
            r#"{"schema_version":1,"ruleset_id":"bomberman_stage1","arena":{"width":13,"height":11,"indestructible_walls":[{"x":1,"y":2},{"x":2,"y":2}],"destructible_walls":[]},"rules":{"tick_hz":64,"bomb_fuse_ticks":3,"bomb_radius":2,"max_bombs_per_player":1,"random_bomb_batch_size":100,"random_bomb_interval_ticks":100,"random_bomb_seed":305419896},"players":[{"actor":1,"spawn":{"x":1,"y":1}},{"actor":2,"spawn":{"x":11,"y":9}}]}"#,
        )
        .unwrap();
        let first = compile_bomberman(first, OPTIONS).unwrap();
        let second = compile_bomberman(second, OPTIONS).unwrap();
        assert_eq!(first.ir, second.ir);
        assert_eq!(first.definition_hash, second.definition_hash);
    }

    #[test]
    fn game_language_compiles_to_runtime_definition() {
        let source = parse_bomberman_lang(include_str!(
            "../../demo/bomberman/engine/bevy/lang/bomberman.game"
        ))
        .unwrap();
        let compiled = compile_bomberman(
            source,
            BombermanCompileOptions {
                expected_width: 500,
                expected_height: 500,
                max_players: 8,
                ruleset_id: BOMBERMAN_RULESET_ID,
            },
        )
        .unwrap();
        assert_eq!(compiled.ir.rules.tick_hz, 64);
        assert_eq!(compiled.ir.rules.random_bomb_seed, 0x1234_5678);
        assert_eq!(compiled.ir.players.len(), 2);
    }

    #[test]
    fn generated_definition_is_rust_source() {
        let source = parse_bomberman_lang(include_str!(
            "../../demo/bomberman/engine/bevy/lang/bomberman.game"
        ))
        .unwrap();
        let compiled = compile_bomberman(
            source,
            BombermanCompileOptions {
                expected_width: 500,
                expected_height: 500,
                max_players: 8,
                ruleset_id: BOMBERMAN_RULESET_ID,
            },
        )
        .unwrap();
        let generated = generate_bomberman_rust(&compiled);
        assert!(generated.contains("BOMBERMAN_DEFINITION"));
        assert!(generated.contains("definition_hash"));
    }

    #[test]
    fn unknown_source_fields_are_rejected() {
        let error = parse_bomberman(
            r#"{
                "schema_version": 1,
                "ruleset_id": "bomberman_stage1",
                "arena": {"width": 13, "height": 11},
                "rules": {"tick_hz": 64, "bomb_fuse_ticks": 3, "bomb_radius": 2, "max_bombs_per_player": 1},
                "players": [{"actor": 1, "spawn": {"x": 1, "y": 1}, "color": [0, 180, 255]}]
            }"#,
        )
        .expect_err("presentation-only color must not enter gameplay AST");
        assert!(matches!(error, CompileError::Json(_)));
    }
    #[test]
    fn village_json_compiles_to_normalized_ir() {
        let source = parse_village(include_str!(
            "../../demo/bomberman/engine/bevy/lang/village.json"
        ))
        .unwrap();
        let compiled = compile_village(source, VILLAGE_OPTIONS).unwrap();
        let second = compile_village(
            parse_village(include_str!(
                "../../demo/bomberman/engine/bevy/lang/village.json"
            ))
            .unwrap(),
            VILLAGE_OPTIONS,
        )
        .unwrap();

        assert_eq!(compiled.ir.world.width_m, 1_000);
        assert_eq!(
            compiled
                .ir
                .world
                .streets
                .iter()
                .filter(|street| street.role == "outgoing")
                .count(),
            6
        );
        assert_eq!(
            compiled
                .ir
                .world
                .streets
                .iter()
                .filter(|street| street.role == "local")
                .count(),
            30
        );
        assert_eq!(compiled.ir.world.streets.len(), 36);
        assert!(
            compiled
                .ir
                .world
                .streets
                .iter()
                .enumerate()
                .all(|(index, street)| {
                    compiled
                        .ir
                        .world
                        .streets
                        .iter()
                        .enumerate()
                        .any(|(other_index, other)| {
                            index != other_index
                                && (street.path.start == other.path.start
                                    || street.path.start == other.path.end
                                    || street.path.end == other.path.start
                                    || street.path.end == other.path.end)
                        })
                })
        );
        assert!(
            compiled
                .ir
                .agents
                .iter()
                .filter(|agent| agent.kind == "pedestrian")
                .all(|agent| agent.follows_streets)
        );
        assert_eq!(compiled.ir.traffic.intersections.len(), 1);
        assert_eq!(compiled.ir.agents.len(), 12);
        assert_eq!(
            compiled.ir.world.curve_types,
            vec!["gentle_arc", "hairpin", "s_bend", "straight"]
        );
        assert_ne!(compiled.definition_hash, [0; 32]);
        assert_eq!(compiled.definition_hash, second.definition_hash);
    }

    #[test]
    fn village_validation_requires_pedestrian_priority() {
        let mut source = parse_village(include_str!(
            "../../demo/bomberman/engine/bevy/lang/village.json"
        ))
        .unwrap();
        source
            .agents
            .iter_mut()
            .find(|agent| agent.kind == "pedestrian")
            .expect("village fixture has a pedestrian")
            .priority_over
            .clear();

        let error = compile_village(source, VILLAGE_OPTIONS).unwrap_err();

        assert!(matches!(error, CompileError::Village(_)));
    }

    #[test]
    fn village_density_must_decrease_away_from_center() {
        let mut source = parse_village(include_str!(
            "../../demo/bomberman/engine/bevy/lang/village.json"
        ))
        .unwrap();
        source.world.street_generation.density_bands[0].probability_percent = 5;

        let error = compile_village(source, VILLAGE_OPTIONS).unwrap_err();

        assert!(matches!(error, CompileError::Village(_)));
    }
}
