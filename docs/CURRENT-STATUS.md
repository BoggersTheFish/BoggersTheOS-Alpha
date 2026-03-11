# Current status and recent changes (Alpha)

## Implemented

- **TsRegistry** is the central authority: `NodeInfo` (weight f32, parent, dependencies, status); `register_node`, `get_weight`, `resolve_conflict(node_a, node_b)` — kernel always wins.
- **Scheduler**: TS-weighted selection (highest weight first), round-robin within same-weight tier; scheduling decisions logged with weights for audit.
- **Security**: Every syscall checks caller's node weight vs a per-syscall `min_weight`; kernel (1.0) always allowed; violations logged.
- **HAL**: Simulated devices `sim-uart` and `sim-timer`; drivers crate registers them in TsRegistry with weights 0.78 (parent = kernel).
- **New syscalls**: `GetNodeWeight`, `YieldToStronger`, `Print`; toy app demos querying weight and failing to call kernel-only `Spawn`.
- **Hierarchy dump**: Tree visualization with kernel at top (★), children by parent, weights and kinds shown.
- **No-override enforcement**: Assertions and comments in registry (no weight >= 1.0), scheduler (only pick from highest tier), syscall (min_weight check); kernel weight asserted 1.0 in `get_weight`.

## Suggested weights for future nodes

| Node / subsystem | Suggested weight | Note |
|------------------|------------------|------|
| Kernel           | 1.0              | Fixed; nothing may equal or exceed. |
| Drivers (core)   | 0.85             | CPU, RAM, storage. |
| Sim devices      | 0.78             | sim-uart, sim-timer. |
| Syscall / lib    | 0.7              | Boundary and OS lib. |
| Networking       | 0.6              | When added. |
| GUI              | 0.5              | Emergent UI. |
| Apps             | 0.3–0.4          | User applications. |
