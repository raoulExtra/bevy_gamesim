# Requirement: Work with contracts

Bevy GameSim MUST be designed and implemented around explicit contracts at every boundary.

## Rules

- Every pipe, adapter, interpreter, replay, network, and presentation boundary MUST define its input, output, ownership, version, and fault behavior.
- Prefer typed, deterministic, serializable schemas for data that crosses a boundary.
- Contracts MUST state ordering, limits, capabilities, authority, and determinism requirements where relevant.
- Hidden global state, undocumented side effects, and engine-specific object pointers MUST NOT cross simulation boundaries.
- The authoritative simulation MUST accept only validated commands that satisfy the declared contract.
- Contract changes MUST update every affected producer, consumer, fixture, and document in the same change.
- Incompatible changes MUST use an explicit version or migration path; silent reinterpretation is not allowed.
- Contract tests and deterministic fixtures SHOULD prove the observable behavior at important boundaries.

## Acceptance

A change satisfies this requirement when its boundary contract is written down, versioned where needed, validated by the receiving authority, and exercised by the affected consumers or fixtures.
