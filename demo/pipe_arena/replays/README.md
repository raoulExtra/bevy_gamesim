# Replays

Store recorded input streams and small deterministic fixtures here.

A replay should include:

- protocol version;
- ruleset identifier;
- initial-state identifier;
- tick-indexed input frames;
- periodic state hashes; and
- optional snapshots for seeking.

Do not store generated captures or large binary recordings without documenting their size and purpose. Replays are test inputs for every engine client and must be playable by the same Rust simulation used in live sessions.
