use bevy_gamesim_ast::{VILLAGE_RULESET_ID, VillageCompileOptions, compile_village, parse_village};

fn main() {
    let source = parse_village(include_str!(
        "../../demo/bomberman/engine/bevy/lang/village.json"
    ))
    .expect("village JSON must parse");
    let compiled = compile_village(
        source,
        VillageCompileOptions {
            ruleset_id: VILLAGE_RULESET_ID,
            expected_width_m: 1_000,
            expected_height_m: 1_000,
            expected_outgoing_streets: 6,
        },
    )
    .expect("village JSON must compile");
    let primary_streets = compiled
        .ir
        .world
        .streets
        .iter()
        .filter(|street| street.role == "outgoing")
        .count();
    let local_streets = compiled.ir.world.streets.len() - primary_streets;

    println!(
        "village compiler: {} primary streets, {} generated local streets, {} intersections, {} agents",
        primary_streets,
        local_streets,
        compiled.ir.traffic.intersections.len(),
        compiled.ir.agents.len()
    );
    println!("canonical bytes: {}", compiled.canonical_bytes.len());
    println!("definition hash: {:02x?}", compiled.definition_hash);
}
