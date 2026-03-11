# What BoggersTheOS Actually Does and Doesn’t Do

Use this when reviewing the repo (e.g. with another AI) to see what’s implemented vs. what’s design-only.

---

## What it DOES do

1. **Runs as a normal program**  
   It’s a single binary (`boggers_os`) that runs on Windows/macOS/Linux. It does **not** boot a real machine or replace the host OS.

2. **TS “strongest node” model**  
   - One central **kernel** (strongest node, weight 1.0).  
   - Other components register as **nodes** with lower weights (drivers 0.85, syscall 0.7, GUI 0.5, apps 0.3).  
   - A **TsRegistry** holds all nodes; scheduling uses **node weight** (higher weight runs first among ready processes).

3. **Processes and scheduling**  
   - **Scheduler** can spawn processes, mark them Ready/Running/Terminated, and **schedule** by picking the ready process with the **highest node weight** (not strict round-robin).  
   - Processes are in-memory structs (PCBs); there is **no** real context switch or separate address spaces.

4. **“System calls”**  
   - **SyscallHandler** implements: Exit, GetPid, Yield, Alloc, Dealloc, Log, Spawn.  
   - They operate on the in-process kernel state (scheduler, memory, security). There are **no** real traps, no user/kernel boundary, no separate kernel process.

5. **Memory “allocation”**  
   - **MemoryManager** hands out regions from a single contiguous “heap” (base + size), tagged by `node_id`.  
   - No virtual memory, no paging, no real isolation; it’s a bookkeeping layer over one big buffer.

6. **Security layer**  
   - **SecurityContext** (node_id + privilege) and **SecurityMonitor** (check_access, log violations).  
   - Used inside syscall dispatch; no real privilege levels or hardware enforcement.

7. **Hardware abstraction (simulated)**  
   - **HAL traits** in the kernel: `CpuNode`, `RamNode`, `StorageNode`, `NetworkNode`, `Hal`.  
   - **hal** crate implements them with in-memory/simulated data (e.g. sim block storage in a `Vec<u8>`). No real hardware is touched.

8. **Layered crates**  
   - Dependency flow: kernel → hal → drivers; kernel → syscall → libos; libos/gui/apps used by the **os** binary.  
   - The **os** binary wires everything and runs a short demo: register nodes, spawn processes, run a few scheduler steps, one alloc, then print the TS node list.

9. **Documentation**  
   - **TS-HIERARCHY.md**: conceptual TS graph and why each layer exists.  
   - **MASTER-PLAN.md**: how to build, run, and optional expansions (bare metal, networking, graphics, FS).

---

## What it DOESN’T do

1. **No boot or bare metal**  
   - No bootloader, no multiboot, no `no_std`, no real kernel at a fixed physical address.  
   - Doesn’t run on real or emulated hardware as an OS; it’s a userspace program.

2. **No real multitasking**  
   - No threads, no preemption, no timer interrupts.  
   - “Scheduling” is just choosing the next process in a loop; everything runs in one process, one thread.

3. **No real system call mechanism**  
   - No software interrupt (e.g. `int 0x80`), no syscall instruction, no user/kernel mode switch.  
   - “Syscalls” are normal function calls into `SyscallHandler`.

4. **No real memory protection**  
   - No page tables, no MMU, no separate address spaces.  
   - All “processes” share the same memory; “alloc” is just kernel bookkeeping.

5. **No real drivers or hardware**  
   - No VGA, no disk, no NIC. Only simulated HAL implementations in process memory.

6. **No file system, no network stack**  
   - No files, no sockets, no networking. Only traits/placeholders where they could go.

7. **No real GUI**  
   - **gui** crate only registers a node and has stub `on_input` / `render`; no window, no pixels, no input handling.

8. **No persistence**  
   - Nothing is saved to disk; all state is in memory and disappears when the binary exits.

---

## One-line summary

**It’s an in-process simulation of a TS-shaped OS design (kernel as strongest node, weighted scheduling, syscall-style API, simulated HAL and memory) that runs as a normal executable and does not boot, multitask, or touch real hardware.**

You can push this repo to GitHub and ask another AI to “review what this OS project actually does and doesn’t do” and point it at this file and the code.
