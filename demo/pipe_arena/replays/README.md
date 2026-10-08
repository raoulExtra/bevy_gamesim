# Replays

Store recorded input streams and small deterministic fixtures here.

The checked-in Stage 1 fixture pair is `stage1-input.json` and `stage1-expected.json`. The input loader converts the reviewable JSON frames into canonical binary `TickInputFrame` values; the expected file pins per-tick commands, presentation events, rejections, and state hashes. Run the comparison with:

```sh
cargo run -p bevy_pipe_core --example stage1
```

A replay should include:

- protocol version;
- ruleset identifier;
- initial-state identifier;
- tick-indexed input frames;
- periodic state hashes; and
- optional snapshots for seeking.

Do not store generated captures or large binary recordings without documenting their size and purpose. Replays are test inputs for every engine client and must be playable by the same Rust simulation used in live sessions.
