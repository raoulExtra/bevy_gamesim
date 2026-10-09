use bevy_pipe_core::{
    BOMBERMAN_DEFINITION, ExpectedRunDocument, InputFixtureDocument, Simulation, TickResult,
};
use std::{env, fs, path::PathBuf};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..");
    let input_path = root.join("demo/bomberman/replays/stage1-input.json");
    let expected_path = root.join("demo/bomberman/replays/stage1-expected.json");
    let write_expected = env::args()
        .skip(1)
        .any(|argument| argument == "--write-expected");

    let input = InputFixtureDocument::load_json(&fs::read_to_string(&input_path)?)?;
    let frames = input.to_frames()?;

    let mut simulation = Simulation::from_definition(&BOMBERMAN_DEFINITION)?;
    let initial_state_hash = simulation.state().hash();
    let results = frames
        .iter()
        .map(|frame| simulation.step(frame))
        .collect::<Result<Vec<_>, _>>()?;

    if write_expected {
        let expected = ExpectedRunDocument::from_results(
            BOMBERMAN_DEFINITION.definition_hash,
            initial_state_hash,
            &results,
        );
        fs::write(
            &expected_path,
            serde_json::to_string_pretty(&expected)? + "\n",
        )?;
        println!(
            "wrote {} ticks to {}",
            expected.ticks.len(),
            expected_path.display()
        );
        return Ok(());
    }

    let expected = ExpectedRunDocument::load_json(&fs::read_to_string(&expected_path)?)?;
    compare_run(&expected, initial_state_hash, &results)?;

    let mut repeat = Simulation::from_definition(&BOMBERMAN_DEFINITION)?;
    for frame in &frames {
        repeat.step(frame)?;
    }
    if repeat.state().hash() != simulation.state().hash() {
        return Err("first divergence: repeated run final state hash differs".into());
    }

    println!(
        "stage1 pipeline: generated definition loaded, {} ticks matched, final hash {:016x}",
        results.len(),
        simulation.state().hash()
    );
    Ok(())
}

fn compare_run(
    expected: &ExpectedRunDocument,
    initial_state_hash: u64,
    actual: &[TickResult],
) -> Result<(), Box<dyn std::error::Error>> {
    if expected.protocol_version != 1 {
        return Err(format!(
            "fixture metadata mismatch: protocol version {}",
            expected.protocol_version
        )
        .into());
    }
    if expected.ruleset_id != "bomberman_stage1" {
        return Err(format!("fixture metadata mismatch: ruleset {}", expected.ruleset_id).into());
    }
    if expected.definition_hash != BOMBERMAN_DEFINITION.definition_hash {
        return Err("fixture metadata mismatch: definition hash".into());
    }
    if expected.initial_state_hash != initial_state_hash {
        return Err(format!(
            "first divergence: initial_state_hash expected {:016x}, actual {:016x}",
            expected.initial_state_hash, initial_state_hash
        )
        .into());
    }
    if expected.ticks.len() != actual.len() {
        return Err(format!(
            "first divergence: tick count expected {}, actual {}",
            expected.ticks.len(),
            actual.len()
        )
        .into());
    }

    for (index, (expected_tick, actual_tick)) in expected.ticks.iter().zip(actual).enumerate() {
        if expected_tick.tick != actual_tick.tick {
            return Err(first_divergence(
                index,
                "tick",
                &expected_tick.tick,
                &actual_tick.tick,
            ));
        }
        if expected_tick.commands != actual_tick.commands {
            return Err(first_divergence(
                index,
                "commands",
                &expected_tick.commands,
                &actual_tick.commands,
            ));
        }
        if expected_tick.events != actual_tick.events {
            return Err(first_divergence(
                index,
                "events",
                &expected_tick.events,
                &actual_tick.events,
            ));
        }
        if expected_tick.rejections != actual_tick.rejections {
            return Err(first_divergence(
                index,
                "rejections",
                &expected_tick.rejections,
                &actual_tick.rejections,
            ));
        }
        if expected_tick.state_hash != actual_tick.state_hash {
            return Err(first_divergence(
                index,
                "state_hash",
                &format!("{:016x}", expected_tick.state_hash),
                &format!("{:016x}", actual_tick.state_hash),
            ));
        }
    }
    Ok(())
}

fn first_divergence<T: std::fmt::Debug>(
    index: usize,
    field: &str,
    expected: &T,
    actual: &T,
) -> Box<dyn std::error::Error> {
    format!(
        "first divergence at tick index {index}, field {field}: expected {expected:?}, actual {actual:?}"
    )
    .into()
}
