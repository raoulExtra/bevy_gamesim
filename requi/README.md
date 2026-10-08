# Requirements

Project-wide requirements for Bevy GameSim.

## General

- [Reuse as much as we can](general/01-reuse-as-much-as-we-can.md) — prefer existing Bevy, Rust, protocol, and tooling work over parallel abstractions.
- [Work with contracts](general/02-work-with-contracts.md) — define explicit, typed, versioned boundaries for pipes, adapters, interpreters, replay, networking, and presentation.

These requirements apply to implementation, documentation, tests, tools, and engine adapters. A change that introduces a new mechanism must identify the existing work it reuses or explain the concrete gap. A boundary change must update affected producers, consumers, fixtures, and documents together.
