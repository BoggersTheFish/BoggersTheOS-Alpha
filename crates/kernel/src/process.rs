//! Process and scheduler. TS: scheduling is strictly by node weight (higher first);
//! within the same weight tier we use round-robin. All decisions are logged for audit.

use crate::error::KernelError;
use crate::node::{NodeId, TsRegistry};
use std::collections::{HashMap, VecDeque};
use std::sync::RwLock;

pub type ProcessId = u64;

/// Max number of scheduling decision log entries to keep (TS audit trail).
const SCHED_LOG_CAP: usize = 64;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ProcessState {
    Ready,
    Running,
    Blocked,
    Terminated,
}

/// Minimal process control block. TS: node_id determines scheduling tier.
#[derive(Debug, Clone)]
pub struct Process {
    pub id: ProcessId,
    pub state: ProcessState,
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

/// TS-weighted scheduler: picks by node weight (higher first), round-robin within same weight.
/// No override: we never bypass weight order; kernel-owned processes always win vs lower-weight nodes.
pub struct Scheduler {
    registry: std::sync::Arc<TsRegistry>,
    processes: RwLock<HashMap<ProcessId, Process>>,
    ready_queue: RwLock<VecDeque<ProcessId>>,
    current: RwLock<Option<ProcessId>>,
    next_pid: RwLock<ProcessId>,
    /// TS: log of scheduling decisions (weight used so we can audit no-override).
    schedule_log: RwLock<VecDeque<String>>,
}

impl Scheduler {
    pub fn new(registry: std::sync::Arc<TsRegistry>) -> Self {
        Self {
            registry,
            processes: RwLock::new(HashMap::new()),
            ready_queue: RwLock::new(VecDeque::new()),
            current: RwLock::new(None),
            next_pid: RwLock::new(1),
            schedule_log: RwLock::new(VecDeque::new()),
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

    /// TS: pick next process by (1) highest node weight among ready, (2) round-robin within that tier.
    /// Queue order preserves FIFO per tier; we take the first pid with max weight.
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

        let reg = self.registry.clone();
        let procs = self.processes.read().unwrap();
        let weights: Vec<(ProcessId, f64)> = queue
            .iter()
            .filter_map(|&pid| {
                procs.get(&pid).and_then(|p| {
                    reg.get_weight(p.node_id).map(|w| (pid, w))
                })
            })
            .collect();
        let max_weight = weights.iter().map(|(_, w)| *w).fold(0.0_f64, f64::max);
        // TS no-override: we only ever pick from the highest weight tier; never demote kernel.
        // Round-robin within tier: first pid in queue that has max_weight
        let next_pid = queue
            .iter()
            .find(|&&pid| {
                weights.iter().any(|(p, w)| *p == pid && (*w - max_weight).abs() < 1e-9)
            })
            .copied();
        drop(procs);

        let next = next_pid.map(|pid| {
            queue.retain(|&x| x != pid);
            pid
        });

        if let Some(pid) = next {
            if let Some(p) = self.processes.write().unwrap().get(&pid) {
                let w = reg.get_weight(p.node_id).unwrap_or(0.0);
                self.log_schedule(pid, p.node_id, w, &p.name);
                let mut proc = self.processes.write().unwrap();
                if let Some(pr) = proc.get_mut(&pid) {
                    pr.state = ProcessState::Running;
                }
            }
            *current = Some(pid);
        }
        next
    }

    /// TS: append one scheduling decision to the log (weight used for audit).
    fn log_schedule(&self, pid: ProcessId, node_id: NodeId, weight: f64, name: &str) {
        let mut log = self.schedule_log.write().unwrap();
        log.push_back(format!(
            "scheduled pid={} node_id={} weight={:.3} name={}",
            pid, node_id, weight, name
        ));
        while log.len() > SCHED_LOG_CAP {
            log.pop_front();
        }
    }

    /// Retrieve recent scheduling log (for os binary or debug).
    pub fn schedule_log(&self) -> Vec<String> {
        self.schedule_log.read().unwrap().iter().cloned().collect()
    }

    pub fn current(&self) -> Option<ProcessId> {
        *self.current.read().unwrap()
    }

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
