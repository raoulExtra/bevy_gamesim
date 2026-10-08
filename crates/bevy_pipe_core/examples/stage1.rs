use bevy_pipe_core::{Replay, Simulation, scripted_frames};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let frames = scripted_frames();
    let first_run = Simulation::new().run_hashes(&frames)?;
    let second_run = Simulation::new().run_hashes(&frames)?;
    assert_eq!(first_run, second_run, "repeated runs diverged");

    let replay = Replay::record(&frames)?;
    replay.verify()?;

    println!(
        "stage1 deterministic: {} ticks, final hash {:016x}, replay verified",
        first_run.len(),
        first_run.last().copied().unwrap_or_default()
    );
    Ok(())
}
