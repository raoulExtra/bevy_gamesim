# Replay and rollback

## Replay model

A replay is a stream of versioned input frames and authority metadata, not a recording of rendered frames.

Replay playback runs the same simulation pipe as live play. It MUST NOT use a second gameplay implementation.

## Replay contents

Minimum replay contents:

- simulation protocol version;
- game and ruleset identifier;
- initial state or initial-state hash plus package reference;
- tick-indexed input frames;
- authoritative corrections, if any;
- periodic state hashes; and
- optional snapshots for seeking.

The replay format should support sequential playback, seeking from a snapshot, integrity checks, and schema migration where practical.

## Replay tooling

Required tooling:

- record inputs and hashes;
- replay from a selected tick;
- compare two runs at the first divergent tick;
- inspect commands and events for a tick; and
- export a minimal divergence artifact.

The command and event trace should identify source, tick, schema version, validation result, and application order.

## Rollback model

Rollback maintains a ring of authoritative snapshots and input frames:

```text
snapshot[N-rollback_window]
        ...
 snapshot[N]
        ↓ late input or correction
 restore snapshot[K]
 replay inputs K..N
 replace predicted state
```

A rollback implementation MUST define:

- snapshot ownership and format;
- maximum rollback window;
- state excluded because it is presentation-only;
- entity identity preservation rules;
- side-effect suppression during resimulation; and
- behavior when a command is rejected after prediction.

The restored state and input frames MUST be sufficient to run the ordinary simulation pipe. Rollback MUST NOT introduce a separate mutation path.

## Pipe state ownership

An authoritative pipe MUST be stateless or expose deterministic snapshot and restore state. Any mutable interpreter state that can affect future commands belongs in the rollback snapshot and replay contract. Engine objects, presentation handles, wall-clock values, packet order, and process-global randomness are never rollback state.

Restoring tick `K` restores the authoritative simulation state and every stateful authoritative pipe to the same logical point before tick `K` is replayed. Presentation systems are rebuilt from confirmed state and events rather than restored as authoritative state.

## Presentation and side effects

Audio, particles, visual effects, analytics, and network sends MUST be driven through confirmation-aware presentation events so rollback does not duplicate irreversible side effects.

Presentation events should declare whether they are:

- speculative and safe to replace;
- confirmed and safe to commit; or
- irreversible and delayed until authority confirmation.

Simulation state and presentation state must have separate ownership and lifetimes.

## Acceptance properties

A correct rollback implementation satisfies both properties:

1. Correcting an input at tick `K` and resimulating produces the same final state as a clean run that received the corrected input at tick `K`.
2. Repeated rollback and replay do not duplicate irreversible side effects.

## Debugging

When rollback or replay diverges, diagnostics should expose:

- snapshot hash before restore;
- corrected input frame;
- resimulated tick range;
- accepted and rejected commands;
- state hash after each tick; and
- first component-level difference.
