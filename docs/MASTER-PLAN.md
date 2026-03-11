# Master Plan: Build, Compile, and Deploy BoggersTheOS

## Prerequisites

- **Rust** (stable): `rustup default stable`
- **OS**: Windows, macOS, or Linux (skeleton runs as a normal process on the host)
- For optional **bare-metal** work: cross-compiler, GRUB or other bootloader, QEMU (see Optional expansions)

---

## Step 1: Build the workspace

From the repo root that contains `BoggersTheOS`:

```bash
cd path/to/BoggersTheOS
cargo build
```

This builds all crates in TS order (kernel first, then hal, drivers, syscall, libos, gui, apps, os).

---

## Step 2: Run the OS skeleton

```bash
cargo run -p boggers_os
```

You should see:

- Kernel startup message
- GUI and app node registration
- Spawned processes (e.g. shell, init)
- Scheduler steps (which PID runs, by node weight) and a scheduling log (weights used)
- A successful alloc via syscall
- Toy app: get_node_weight for shell/init; Spawn denied for app (weight too low)
- A tree dump of the TS node hierarchy (kernel ★ at top, children by parent)

The binary is a **hosted** OS personality: it runs as a normal process and simulates the TS stack (kernel, drivers, syscall, libos, gui, apps) so you can develop and test logic without bare metal.

---

## Step 3: Run tests (when added)

```bash
cargo test
```

(Add unit tests in each crate as needed; kernel and syscall are the highest priority.)

---

## Step 4: Deploy (current skeleton)

**Deployment** today means:

1. **Build release**: `cargo build --release -p boggers_os`
2. **Run** the produced binary (e.g. `target/release/boggers_os.exe` on Windows) on any machine with the same OS and a compatible Rust runtime (or ship the binary; no Rust needed on target if you use a fully static build where possible).

The skeleton does **not** yet:

- Boot on bare metal
- Replace the host OS
- Provide a real filesystem or network stack

It **does**:

- Demonstrate the full TS hierarchy
- Allow adding and testing new syscalls, drivers, and apps in one place

---

## Optional expansions

### A. Bare-metal boot (x86_64)

1. **New crate** (e.g. `boot` or `kernel_bare`):
   - `#![no_std]`, no `std`.
   - Entry point: `_start` (assembly or `#[no_mangle]`), call into a minimal Rust kernel.
   - Link script: kernel at fixed physical address; multiboot header if using GRUB.

2. **Kernel core**:
   - Port `boggers_kernel` to `no_std` (replace `std::sync::RwLock` with `spin::RwLock`, `std::collections` with `alloc` + a no_std-friendly allocator).
   - Keep the same TS types (TsRegistry, NodeId, weights, Scheduler, MemoryManager, SecurityMonitor) so the design stays TS-driven.

3. **HAL**:
   - Implement `Hal` + `CpuNode`, `RamNode`, `StorageNode` with real hardware (e.g. VGA, serial, ATA or AHCI, memory map from boot params).
   - Register these as driver nodes with the kernel’s `TsRegistry`.

4. **Boot flow**:
   - Bootloader loads kernel → jump to `_start` → init GDT/IDT, paging, heap → create `TsRegistry`, register kernel node → init HAL/drivers → init scheduler and syscall handler → jump to first user process or shell.

5. **Build and run**:
   - Target: `x86_64-boggers-theos` or `x86_64-unknown-none`; use `cargo build -p kernel_bare --target x86_64-unknown-none`.
   - Create a disk image with GRUB + kernel; run in QEMU: `qemu-system-x86_64 -drive file=os.iso,format=raw ...`.

### B. Networking

1. **HAL**: Add `NetworkNode` (already in kernel `hal_traits`) and implement it (e.g. for a specific NIC or a simulated one).
2. **Driver**: Register a network driver node; kernel (or a network stack crate) uses it for send/receive.
3. **Syscall**: Add syscalls for socket-like operations (e.g. send, recv, bind) that go through the kernel and the network driver node.
4. **TS**: Network driver is a high-weight node; under load the kernel can prioritise it over low-weight apps.

### C. Graphics / GUI

1. **HAL**: Add a `FramebufferNode` or `DisplayNode` trait in the kernel (or in HAL) for “write pixels here”.
2. **Driver**: VGA or a real GPU driver implements it; register as a node.
3. **GUI crate**: Use the framebuffer (via kernel/HAL) to draw windows, compositing, and handle input; keep GUI as an emergent node (weight 0.5) so it doesn’t override kernel/drivers.
4. **Optional**: Add a simple compositor that aggregates app surfaces and respects TS weights for who gets focus or priority.

### D. File system

1. **Kernel**: Add a `Filesystem` abstraction (e.g. trait or module) that uses `StorageNode` (block read/write) and is owned by the kernel.
2. **Driver**: Block storage driver (already in HAL) feeds the FS.
3. **Syscall**: Open, read, write, close, mkdir, etc., all go through the kernel; security context and node weights apply.
4. **TS**: FS is a kernel-level or high-weight service; user apps (low weight) get access only via syscalls.

---

## Summary checklist

| Step | Action |
|------|--------|
| 1 | `cargo build` in `BoggersTheOS` |
| 2 | `cargo run -p boggers_os` to run the skeleton |
| 3 | Add and run `cargo test` as you add tests |
| 4 | Deploy: `cargo build --release -p boggers_os` and ship the binary |
| 5 (optional) | Bare metal: new crate, no_std kernel, HAL impl, bootloader + QEMU |
| 6 (optional) | Networking: NetworkNode + driver + syscalls |
| 7 (optional) | Graphics: DisplayNode + driver + GUI/compositor |
| 8 (optional) | File system: FS on StorageNode + syscalls |

All design and implementation choices stay **TS-compliant**: strongest node (kernel) first, strict hierarchy, no fallback that overrides node weighting.
