//! TS Node types and registry. The kernel is the strongest node; all others
//! are weighted relative to it and must justify existence through TS logic.

use crate::error::KernelError;
use std::collections::HashMap;
use std::sync::RwLock;

/// Unique identifier for a TS node. Kernel has a reserved id (0).
pub type NodeId = u32;

/// Weight of a node relative to the strongest node (kernel). Range [0.0, 1.0].
/// Kernel = 1.0; dependent subsystems have lower weights.
pub type NodeWeight = f64;

/// Reserved node id for the kernel (strongest node).
pub const KERNEL_NODE_ID: NodeId = 0;

/// Maximum weight (kernel).
pub const WEIGHT_STRONGEST: NodeWeight = 1.0;

#[derive(Debug, Clone, PartialEq)]
pub enum NodeKind {
    /// The core: hardware abstraction, memory, scheduling.
    Kernel,
    /// Driver (CPU, GPU, storage, RAM, network).
    Driver,
    /// System call interface.
    Syscall,
    /// Library / runtime.
    Library,
    /// UI subsystem.
    Ui,
    /// User application.
    Application,
}

/// A single TS node: identity, kind, and weight relative to kernel.
#[derive(Debug, Clone)]
pub struct TsNode {
    pub id: NodeId,
    pub kind: NodeKind,
    /// Weight relative to strongest node. Used for prioritisation and resource allocation.
    pub weight: NodeWeight,
    /// Human-readable name for debugging and docs.
    pub name: String,
}

impl TsNode {
    pub fn kernel() -> Self {
        Self {
            id: KERNEL_NODE_ID,
            kind: NodeKind::Kernel,
            weight: WEIGHT_STRONGEST,
            name: "kernel".into(),
        }
    }

    /// Returns whether this node is the strongest (kernel).
    #[inline]
    pub fn is_strongest(&self) -> bool {
        self.id == KERNEL_NODE_ID
    }
}

/// Global registry of TS nodes. All modules register here; kernel is always present.
/// Used for self-evaluation and optimisation relative to the strongest node.
pub struct TsRegistry {
    nodes: RwLock<HashMap<NodeId, TsNode>>,
    next_id: RwLock<NodeId>,
}

impl TsRegistry {
    pub fn new() -> Self {
        let nodes = RwLock::new(HashMap::new());
        let mut map = nodes.write().unwrap();
        map.insert(KERNEL_NODE_ID, TsNode::kernel());
        drop(map);
        Self {
            nodes,
            next_id: RwLock::new(KERNEL_NODE_ID + 1),
        }
    }

    /// Register a new node. Kernel (id 0) is already registered.
    pub fn register(&self, kind: NodeKind, weight: NodeWeight, name: String) -> Result<NodeId, KernelError> {
        let mut next = self.next_id.write().unwrap();
        let id = *next;
        *next = next.saturating_add(1);
        drop(next);
        let node = TsNode { id, kind, weight, name };
        self.nodes.write().unwrap().insert(id, node);
        Ok(id)
    }

    /// Get node by id. Used by subsystems to self-evaluate relative to kernel.
    pub fn get(&self, id: NodeId) -> Option<TsNode> {
        self.nodes.read().unwrap().get(&id).cloned()
    }

    /// Weight of node relative to strongest. Kernel returns 1.0.
    pub fn weight_of(&self, id: NodeId) -> Option<NodeWeight> {
        self.nodes.read().unwrap().get(&id).map(|n| n.weight)
    }

    /// Iterate all nodes (e.g. for optimisation passes).
    pub fn all_nodes(&self) -> Vec<TsNode> {
        self.nodes.read().unwrap().values().cloned().collect()
    }
}

impl Default for TsRegistry {
    fn default() -> Self {
        Self::new()
    }
}
