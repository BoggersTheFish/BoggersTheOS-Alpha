//! Process and scheduler. TS: scheduling decisions are node-weighted;
//! higher-weight nodes (closer to kernel) get prioritisation when needed.

use crate::error::KernelError;
use crate::node::{NodeId, TsRegistry};
use std::collections::VecDeque;
use std::sync::RwLock;

pub type ProcessId = u64;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ProcessState {
    Ready,
    Running,
    Blocked,
    Terminated,
}

/// Minimal process control block. Real implementation would add memory map, fd table, etc.
#[derive(Debug, Clone)]
pub struct Process {
    pub id: ProcessId,
    pub state: ProcessState,
    /// TS: which subsystem node this process belongs to (for weighted scheduling).
    pub node_id: NodeId,
    pub name: String,
}

impl Process {
    pub fn new(id: ProcessId, node_id: NodeId, name: String) -> Self {
        Self {
            id,
            state: ProcessState::Ready,
            node_id,
            name,
        }
    }
}

/// Simple round-robin scheduler with TS weighting: when choosing among equal readiness,
/// higher node weight gets preference (kernel integrity first).
pub struct Scheduler {
    registry: std::sync::Arc<TsRegistry>,
    processes: RwLock<HashMap<ProcessId, Process>>,
    ready_queue: RwLock<VecDeque<ProcessId>>,
    current: RwLock<Option<ProcessId>>,
    next_pid: RwLock<ProcessId>,
}

use std::collections::HashMap;

impl Scheduler {
    pub fn new(registry: std::sync::Arc<TsRegistry>) -> Self {
        Self {
            registry,
            processes: RwLock::new(HashMap::new()),
            ready_queue: RwLock::new(VecDeque::new()),
            current: RwLock::new(None),
            next_pid: RwLock::new(1),
        }
    }

    /// Spawn a process under the given TS node.
    pub fn spawn(&self, node_id: NodeId, name: String) -> Result<ProcessId, KernelError> {
        let mut next = self.next_pid.write().unwrap();
        let id = *next;
        *next = next.saturating_add(1);
        drop(next);
        let process = Process::new(id, node_id, name);
        self.processes.write().unwrap().insert(id, process);
        self.ready_queue.write().unwrap().push_back(id);
        Ok(id)
    }

    /// Yield current process; select next by TS-weighted ready queue (higher weight first among ready).
    pub fn schedule(&self) -> Option<ProcessId> {
        let mut current = self.current.write().unwrap();
        let mut queue = self.ready_queue.write().unwrap();
        if let Some(prev) = *current {
            if let Some(p) = self.processes.read().unwrap().get(&prev) {
                if p.state == ProcessState::Running {
                    let mut proc = self.processes.write().unwrap();
                    if let Some(pr) = proc.get_mut(&prev) {
                        pr.state = ProcessState::Ready;
                    }
                    queue.push_back(prev);
                }
            }
        }
        *current = None;
        // TS: pick ready process with highest node weight (strongest first)
        let reg = self.registry.clone();
        let procs = self.processes.read().unwrap();
        let best = queue
            .iter()
            .filter_map(|&pid| procs.get(&pid).map(|p| (pid, reg.weight_of(p.node_id).unwrap_or(0.0))))
            .max_by(|a, b| a.1.partial_cmp(&b.1).unwrap_or(std::cmp::Ordering::Equal));
        drop(procs);
        let next = best.map(|(pid, _)| {
            queue.retain(|&x| x != pid);
            pid
        });
        if let Some(pid) = next.as_ref().copied() {
            if let Some(p) = self.processes.write().unwrap().get_mut(&pid) {
                p.state = ProcessState::Running;
            }
            *current = next;
        }
        next
    }

    /// Current running process.
    pub fn current(&self) -> Option<ProcessId> {
        *self.current.read().unwrap()
    }

    /// Terminate process.
    pub fn terminate(&self, pid: ProcessId) -> Result<(), KernelError> {
        let mut procs = self.processes.write().unwrap();
        if let Some(p) = procs.get_mut(&pid) {
            p.state = ProcessState::Terminated;
            return Ok(());
        }
        Err(KernelError::InvalidNode)
    }

    pub fn get_process(&self, pid: ProcessId) -> Option<Process> {
        self.processes.read().unwrap().get(&pid).cloned()
    }
}
