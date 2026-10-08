# Multiplayer authority

## Authority model

The server or designated authority owns:

- accepted input frames;
- authoritative tick progression;
- command validation;
- collision and damage results;
- entity lifecycle; and
- state hashes or snapshots used for correction.

Client-generated intents are requests until accepted by the authority.

## Client flow

```text
client input
    -> input frame
    -> local prediction
    -> server validation
    -> authoritative command/state
    -> client reconciliation or rollback
```

Clients MAY predict local input, but prediction is never authority. A client MUST NOT create authoritative entities or state changes without an accepted command.

## Protocol separation

The simulation protocol should remain independent of transport so local replay and server play use the same pipe contracts.

The network layer may transmit:

- compact input frames;
- accepted commands;
- snapshots;
- state hashes;
- corrections; or
- confirmed presentation events.

The choice depends on game mode, bandwidth, latency, and rollback window. The authoritative simulation must not depend on packet arrival order or transport-specific behavior.

## Validation

Authority validation must check at least:

- source ownership;
- tick validity and acceptable input window;
- capability and permission;
- entity existence and identity;
- spatial and gameplay constraints;
- command size and rate limits; and
- schema and protocol version.

Rejected commands are structured data. They must not partially mutate authoritative state.

## Prediction and reconciliation

A client may run the same deterministic simulation locally using its known inputs and predicted remote inputs. When authoritative state or commands arrive:

1. compare the authoritative tick and hash;
2. retain or restore the matching snapshot;
3. apply the authority correction;
4. replay buffered inputs; and
5. publish corrected presentation state.

The client must not treat a local prediction as confirmed merely because it was locally valid.

## Convergence requirement

Given the same initial authoritative state, protocol version, ruleset, and accepted input stream, clients and server MUST converge to the same authoritative state hashes.

A failed convergence check must identify the first divergent tick and expose the input, pipe output, command, event, and state-hash sequence for that tick.
