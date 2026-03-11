//! BoggersTheOS — TS-driven OS entry point.
//! Builds the full hierarchy: Kernel → Drivers → Syscall → LibOS → GUI → Apps.
//! Run with: cargo run -p boggers_os

use boggers_apps::AppNode;
use boggers_drivers::init_drivers;
use boggers_gui::GuiNode;
use boggers_kernel::node::{NodeKind, NodeId, TsRegistry, KERNEL_NODE_ID};
use boggers_kernel::{MemoryManager, SecurityMonitor, Scheduler};
use boggers_libos::{alloc, get_node_weight};
use boggers_syscall::{SyscallHandler, SyscallNumber};
use std::sync::Arc;

/// TS: print tree of nodes with kernel at top; then children by parent, then rest by weight.
fn print_hierarchy_tree(registry: &TsRegistry) {
    use boggers_kernel::NodeInfo;
    let infos = registry.all_node_infos();
    let mut by_parent: std::collections::HashMap<Option<NodeId>, Vec<NodeInfo>> =
        std::collections::HashMap::new();
    for n in infos {
        by_parent.entry(n.parent).or_default().push(n);
    }
    for v in by_parent.values_mut() {
        v.sort_by(|a, b| b.weight.partial_cmp(&a.weight).unwrap_or(std::cmp::Ordering::Equal));
    }
    fn visit(by_parent: &std::collections::HashMap<Option<NodeId>, Vec<NodeInfo>>, parent: Option<NodeId>, indent: usize) {
        let Some(children) = by_parent.get(&parent) else { return };
        for n in children {
            let prefix = "  ".repeat(indent);
            let marker = if n.id == KERNEL_NODE_ID { "★ " } else { "  " };
            println!(
                "{}{}id={} weight={:.3} kind={:?} name={}",
                prefix, marker, n.id, n.weight, n.kind, n.name
            );
            visit(by_parent, Some(n.id), indent + 1);
        }
    }
    println!("\n--- TS node hierarchy (tree, kernel ★ at top) ---");
    visit(&by_parent, None, 0);
}

fn main() {
    println!("BoggersTheOS — TS-driven kernel starting.\n");

    let registry = Arc::new(TsRegistry::new());
    let scheduler = Arc::new(Scheduler::new(registry.clone()));
    let memory = Arc::new(MemoryManager::new(16 * 1024 * 1024));
    let security = Arc::new(SecurityMonitor::new());

    let syscall_node_id = registry
        .register(NodeKind::Library, 0.7, "syscall".into())
        .expect("register syscall node");
    let syscall = SyscallHandler {
        node_id: syscall_node_id,
        registry: registry.clone(),
        security: security.clone(),
        scheduler: scheduler.clone(),
        memory: memory.clone(),
    };

    let _hal = init_drivers(registry.clone()).expect("init drivers");

    let gui = GuiNode::new(registry.clone()).expect("init gui");
    println!("GUI node id: {}", gui.node_id);

    let app1 = AppNode::new(registry.clone(), "shell".into()).expect("app shell");
    let app2 = AppNode::new(registry.clone(), "init".into()).expect("app init");

    let pid1 = scheduler.spawn(app1.node_id, "shell".into()).unwrap();
    let pid2 = scheduler.spawn(app2.node_id, "init".into()).unwrap();
    println!("Spawned processes: {} (shell), {} (init)", pid1, pid2);

    for step in 0..4 {
        let current = scheduler.schedule();
        if let Some(pid) = current {
            let proc = scheduler.get_process(pid).unwrap();
            println!(
                "  Step {}: running PID {} (node {}: {})",
                step, pid, proc.node_id, proc.name
            );
            let ctx = boggers_kernel::SecurityContext::user(proc.node_id);
            let _ = syscall.dispatch(&ctx, SyscallNumber::Yield, &[]);
        }
    }

    // Scheduling log (TS: weights used for each decision)
    println!("\n--- Scheduling log (last few) ---");
    for line in scheduler.schedule_log().iter().rev().take(5) {
        println!("  {}", line);
    }

    let ctx1 = boggers_kernel::SecurityContext::user(app1.node_id);
    match alloc(&syscall, &ctx1, app1.node_id, 4096) {
        Ok(addr) => println!("\nAllocated 4096 bytes at base {:x}", addr),
        Err(e) => println!("\nAlloc failed: {}", e),
    }

    // Toy app: query own weight
    match get_node_weight(&syscall, &ctx1, Some(app1.node_id)) {
        Ok(w) => println!("App 'shell' node weight: {:.3}", w),
        Err(e) => println!("get_node_weight failed: {}", e),
    }
    let ctx2 = boggers_kernel::SecurityContext::user(app2.node_id);
    match get_node_weight(&syscall, &ctx2, Some(app2.node_id)) {
        Ok(w) => println!("App 'init' node weight: {:.3}", w),
        Err(e) => println!("get_node_weight failed: {}", e),
    }

    // Toy app: try to override kernel (Spawn requires weight 1.0) — must fail
    println!("\n--- Toy app tries Spawn (kernel-only); expect denial ---");
    match syscall.dispatch(&ctx1, SyscallNumber::Spawn, &[app1.node_id as u64, 0]) {
        Ok(_) => println!("  (unexpected success)"),
        Err(e) => println!("  Denied as expected: {}", e),
    }
    let violations = security.violations();
    if !violations.is_empty() {
        println!("  Violations logged: {}", violations.len());
    }

    print_hierarchy_tree(&registry);

    println!("\nBoggersTheOS run complete.");
}
