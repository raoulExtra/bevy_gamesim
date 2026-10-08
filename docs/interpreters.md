# Protected interpreters

## Principle

Interpreters are first-class pipe stages, not special exceptions.

An interpreted part may be implemented as:

- trusted safe Rust;
- a Rust-hosted bytecode interpreter;
- a sandboxed WASM module; or
- a separate process using the pipe protocol.

All implementations use the same input, intent, command, capability, and fault contracts.

## Rust protection

Safe Rust provides memory and ownership safety for the host and native systems. It is not, by itself, a security sandbox for arbitrary code.

Therefore:

- trusted native gameplay MAY run as safe Rust behind the typed interfaces;
- untrusted or user-authored logic MUST run in a WASM sandbox or a separate process; and
- the pipe boundary MUST enforce capabilities, quotas, cancellation, and message validation in both cases.

A Rust GDExtension or native plugin is not automatically isolated from the host process. Isolation requires WASM or process boundaries with an explicit protocol.

## Interpreter context

An interpreter receives a capability-scoped context:

```rust
pub struct InterpreterContext<'a> {
    pub tick: u64,
    pub view: &'a WorldView<'a>,
    pub events: &'a [SimulationEvent],
    pub capabilities: CapabilitySet,
    pub budget: ExecutionBudget,
}
```

The context may expose:

- actor transforms;
- nearby-entity queries;
- deterministic random streams;
- approved simulation events;
- request movement;
- request an animation cue;
- request a spawn through authority; and
- emit a gameplay event.

The concrete API must expose stable query results rather than unrestricted `World`, `Entity`, or resource access.

## Prohibited direct access

The interpreter MUST NOT directly:

- mutate ECS components;
- allocate unbounded memory;
- access the filesystem or network;
- use wall-clock time;
- use process-global randomness;
- call presentation or operating-system services without a capability; or
- retain invalid entity references across ticks without validation.

It emits intents into a bounded output buffer. Authority validates those intents and produces authoritative commands.

## Budgets and cancellation

Every interpreted part receives limits for:

- instruction or fuel count;
- wall-clock execution time;
- output command count;
- output payload size;
- memory;
- event subscriptions; and
- retained state.

The host must be able to cancel a part at a tick boundary and must not wait indefinitely for an interpreter.

## Faults

Faults are data. A timeout, invalid command, budget exhaustion, schema mismatch, or interpreter trap produces a structured fault and a defined policy:

- reject output;
- disable the part for the tick; or
- terminate the session.

An interpreter fault MUST NOT partially mutate authoritative state.

## Pipe contract

A stage MUST declare:

- input schema and schema version;
- output schema and schema version;
- read capabilities;
- write capabilities;
- instruction, time, and output budgets;
- deterministic ordering rules; and
- fault behavior.

Pipe stages MUST NOT communicate through hidden global state. Replacing an in-process Rust interpreter with WASM or a separate process must not change gameplay contracts.

## Determinism

An interpreter participating in authoritative simulation must use:

- supplied tick time, not wall-clock time;
- supplied random streams, not process-global randomness;
- deterministic input ordering;
- bounded state; and
- deterministic serialization for outputs.

A nondeterministic interpreter can be used for presentation or tooling, but it must not emit unverified authoritative commands.
