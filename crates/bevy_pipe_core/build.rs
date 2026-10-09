use bevy_gamesim_ast::{
    BOMBERMAN_RULESET_ID, BombermanCompileOptions, compile_bomberman, generate_bomberman_rust,
    parse_bomberman,
};
use std::{env, fs, path::PathBuf};

fn main() {
    let manifest_dir = PathBuf::from(env::var_os("CARGO_MANIFEST_DIR").expect("manifest dir"));
    let source_path = manifest_dir.join("../../demo/bomberman/bomberman.json");
    println!("cargo:rerun-if-changed={}", source_path.display());

    let source = fs::read_to_string(&source_path).expect("read Bomberman description");
    let source = parse_bomberman(&source).expect("parse Bomberman description");
    let expected_width = source.arena.width;
    let expected_height = source.arena.height;
    let compiled = compile_bomberman(
        source,
        BombermanCompileOptions {
            expected_width,
            expected_height,
            max_players: 4,
            ruleset_id: BOMBERMAN_RULESET_ID,
        },
    )
    .expect("compile Bomberman description");
    let generated = generate_bomberman_rust(&compiled);
    let output_path =
        PathBuf::from(env::var_os("OUT_DIR").expect("out dir")).join("bomberman_definition.rs");
    fs::write(output_path, generated).expect("write generated Bomberman definition");
}
