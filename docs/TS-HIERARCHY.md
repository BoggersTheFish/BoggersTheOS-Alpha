# TS Hierarchy — Strongest-Node Architecture

## Conceptual graph

Every component is a **node**. The **kernel** is the **strongest node** (weight 1.0). All other nodes have weights in (0, 1] and reference the kernel. No fallback logic overrides this: conflicts are resolved by TS hierarchy (strongest node first).

```
                    ┌─────────────────────────────────────┐
                    │         KERNEL (weight = 1.0)        │
                    │  TsRegistry • Scheduler • Memory •   │
                    │  HAL traits • SecurityMonitor        │
                    └─────────────────┬───────────────────┘
                                       │
         ┌─────────────────────────────┼─────────────────────────────┐
         │                             │                             │
         ▼                             ▼                             ▼
  ┌──────────────┐            ┌──────────────┐              ┌──────────────┐
  │   DRIVERS    │            │   SYSCALL    │              │  SECURITY    │
  │ (weight 0.85)│            │ (weight 0.7) │              │  (emergent)  │
  │ CPU • RAM •  │            │ User↔Kernel │              │ Protect      │
  │ Storage • …  │            │ boundary     │              │ kernel first │
  └──────┬───────┘            └──────┬───────┘              └──────────────┘
         │                          │
         └──────────────┬───────────┘
                        ▼
               ┌──────────────┐
               │   LIBOS      │
               │ (lib weight) │
               │ alloc • exit │
               └──────┬───────┘
                      │
         ┌────────────┴────────────┐
         ▼                         ▼
  ┌──────────────┐         ┌──────────────┐
  │     GUI      │         │    APPS      │
  │ (weight 0.5) │         │ (weight 0.3) │
  │ Input •      │         │ shell • init │
  │ Render       │         │ user procs   │
  └──────────────┘         └──────────────┘
```

## Why each decision exists

| Layer | Why it exists | How it references the kernel | Node-weight logic |
|-------|----------------|------------------------------|--------------------|
| **Kernel** | Single point of truth: hardware abstraction, memory, scheduling, security. All state and policy live here. | N/A (it is the core) | Weight 1.0; all priorities and resource allocation derive from this. |
| **HAL** | Hardware as nodes: CPU, RAM, storage, network. Kernel talks only to HAL traits; drivers implement them. | Kernel defines `Hal`, `CpuNode`, `StorageNode`, etc.; kernel owns policy, HAL owns mechanism. | Driver nodes register with `TsRegistry`; weight 0.85 so they are favoured over user code under load. |
| **Drivers** | Implement HAL for concrete hardware (or sim). Register as TS nodes so scheduler and security know them. | `init_drivers(registry)` registers driver nodes; HAL is used by kernel (or by code the kernel trusts). | Same as HAL: secondary nodes, high weight. |
| **Syscall** | Only sanctioned user↔kernel boundary. Every user request becomes a syscall that the kernel validates and executes. | `SyscallHandler` holds `Scheduler`, `MemoryManager`, `SecurityMonitor`; each `dispatch()` checks security then calls kernel. | Syscall layer is a node (e.g. 0.7); each invocation carries caller’s node_id for weighted behaviour. |
| **LibOS** | User-facing API (alloc, exit, yield). No direct kernel access; all via syscall. | Calls `SyscallHandler::dispatch()` with appropriate `SecurityContext` and syscall number. | Every lib call is on behalf of a process/node; that node’s weight affects scheduling and resource allocation. |
| **GUI** | Emergent UI: inputs are events that propagate as weighted nodes; rendering is a low-weight activity. | Registers a UI node with the registry; input handlers can trigger syscalls or kernel-mediated actions. | Weight 0.5; below drivers/syscall, above apps, so UI stays responsive without overriding kernel integrity. |
| **Apps** | Top-level user processes. Each app is a node. | Created via kernel (spawn) or syscall; run under scheduler; use libos/syscall for all privileges. | Weight 0.3; lowest so kernel, drivers, and system services are prioritised under TS-weighted scheduling. |

## Self-optimisation and feedback

- **Scheduler**: Chooses the next process by **node weight** (higher weight first among ready). Kernel integrity (and driver/syscall work) is favoured over arbitrary user processes.
- **Memory**: Allocation is per-node; under pressure the kernel could evict or delay lower-weight nodes first (skeleton does not implement this yet).
- **Security**: `SecurityContext` carries `node_id` and `privilege`. Access checks protect the kernel first (only kernel node can touch kernel resources), then enforce privilege order.

## Conflict resolution

If a conflict arises (e.g. two subsystems claim the same resource, or a security decision is ambiguous), the rule is:

1. **Prioritise the TS hierarchy**: stronger node wins.
2. **Prioritise the strongest node**: kernel integrity over everything else.
3. No silent fallback that bypasses this order; if something cannot be satisfied, return an error rather than overriding TS logic.

## Current status and suggested weights

See [CURRENT-STATUS.md](CURRENT-STATUS.md) for what is implemented and suggested weights for future nodes (e.g. networking 0.6, GUI 0.5, apps 0.3–0.4).
