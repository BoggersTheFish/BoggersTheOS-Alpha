//! BoggersTheOS — TS-driven OS entry point.
//! Builds the full hierarchy: Kernel → Drivers → Syscall → LibOS → GUI → Apps.
//! Run with: cargo run -p boggers_os

use boggers_apps::AppNode;
use boggers_drivers::init_drivers;
use boggers_gui::GuiNode;
use boggers_kernel::{
    node::{NodeKind, TsRegistry},
    MemoryManager, SecurityMonitor, Scheduler,
};
use boggers_libos::alloc;
use boggers_syscall::{SyscallHandler, SyscallNumber};
use std::sync::Arc;

fn main() {
    println!("BoggersTheOS — TS-driven kernel starting.\n");

    // 1. Strongest node: kernel core
    let registry = Arc::new(TsRegistry::new());
    let scheduler = Arc::new(Scheduler::new(registry.clone()));
    let memory = Arc::new(MemoryManager::new(16 * 1024 * 1024)); // 16 MiB heap
    let security = Arc::new(SecurityMonitor::new());

    // 2. Syscall layer (references kernel)
    let syscall_node_id = registry
        .register(NodeKind::Library, 0.7, "syscall".into())
        .expect("register syscall node");
    let syscall = SyscallHandler {
        node_id: syscall_node_id,
        security: security.clone(),
        scheduler: scheduler.clone(),
        memory: memory.clone(),
    };

    // 3. Drivers (HAL)
    let _hal = init_drivers(registry.clone()).expect("init drivers");

    // 4. GUI node
    let gui = GuiNode::new(registry.clone()).expect("init gui");
    println!("GUI node id: {}", gui.node_id);

    // 5. App nodes
    let app1 = AppNode::new(registry.clone(), "shell".into()).expect("app shell");
    let app2 = AppNode::new(registry.clone(), "init".into()).expect("app init");

    // 6. Spawn processes (kernel processes for demo)
    let pid1 = scheduler.spawn(app1.node_id, "shell".into()).unwrap();
    let pid2 = scheduler.spawn(app2.node_id, "init".into()).unwrap();
    println!("Spawned processes: {} (shell), {} (init)", pid1, pid2);

    // 7. Run scheduler a few steps (TS: higher-weight nodes scheduled first)
    for step in 0..4 {
        let current = scheduler.schedule();
        if let Some(pid) = current {
            let proc = scheduler.get_process(pid).unwrap();
            println!("  Step {}: running PID {} (node {}: {})", step, pid, proc.node_id, proc.name);
            // Simulate syscall: yield
            let ctx = boggers_kernel::SecurityContext::user(proc.node_id);
            let _ = syscall.dispatch(&ctx, SyscallNumber::Yield, &[]);
        }
    }

    // 8. Alloc via syscall (user context)
    let ctx = boggers_kernel::SecurityContext::user(app1.node_id);
    match alloc(&syscall, &ctx, app1.node_id, 4096) {
        Ok(addr) => println!("\nAllocated 4096 bytes at base {:x}", addr),
        Err(e) => println!("\nAlloc failed: {}", e),
    }

    // 9. TS node dump
    println!("\n--- TS node hierarchy (strongest first) ---");
    let mut nodes = registry.all_nodes();
    nodes.sort_by(|a, b| b.weight.partial_cmp(&a.weight).unwrap_or(std::cmp::Ordering::Equal));
    for n in nodes {
        println!("  id={} weight={:.2} kind={:?} name={}", n.id, n.weight, n.kind, n.name);
    }

    println!("\nBoggersTheOS run complete.");
}
