# Pipe Arena test scenarios

These scenarios are deterministic fixtures for the Rust simulation and every presentation client.

## Scenario format

Each scenario should define:

- protocol and ruleset versions;
- initial state;
- tick-indexed input frames;
- expected authoritative state hashes;
- expected accepted and rejected commands; and
- expected presentation events.

## Initial scenarios

1. Two players move on an empty 13×11 arena.
2. A player places a bomb and the fixed fuse expires.
3. A blast stops at an indestructible wall.
4. A blast destroys a destructible wall and emits a presentation event.
5. Invalid movement and bomb-placement commands are rejected without mutation.
6. Replaying the same input stream reproduces every expected state hash.
