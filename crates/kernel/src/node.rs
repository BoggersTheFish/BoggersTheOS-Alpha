//! TS Node types and registry. The kernel is the strongest node; all others
//! are weighted relative to it. TsRegistry is the central authority: every
//! component must register and conflicts are resolved by weight (kernel always wins).

use crate::error::KernelError;
use std::collections::HashMap;
use std::sync::RwLock;

/// Unique identifier for a TS node. Kernel has a reserved id (0).
pub type NodeId = u32;

/// Weight of a node relative to the strongest node (kernel). Range [0.0, 1.0].
/// Stored as f32 in NodeInfo for consistency; kernel = 1.0.
pub type NodeWeight = f64;

/// Reserved node id for the kernel (strongest node).
pub const KERNEL_NODE_ID: NodeId = 0;

/// Maximum weight (kernel). Nothing can equal or exceed this.
pub const WEIGHT_STRONGEST: NodeWeight = 1.0;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NodeStatus {
    Active,
    Inactive,
}

#[derive(Debug, Clone, PartialEq)]
pub enum NodeKind {
    Kernel,
    Driver,
    Syscall,
    Library,
    Ui,
    Application,
}

/// Full node information held by the registry. TsRegistry is the single source of truth.
/// Weight is f32 internally; kernel is 1.0, all others strictly less.
#[derive(Debug, Clone)]
pub struct NodeInfo {
    pub id: NodeId,
    pub kind: NodeKind,
    pub name: String,
    /// TS: weight relative to kernel. Kernel = 1.0; no other node may be 1.0.
    pub weight: f32,
    /// Parent in the hierarchy (e.g. driver's parent might be kernel). Kernel has no parent.
    pub parent: Option<NodeId>,
    /// Nodes this one depends on (e.g. syscall depends on kernel).
    pub dependencies: Vec<NodeId>,
    pub status: NodeStatus,
}

impl NodeInfo {
    /// Returns true iff this node is the strongest (kernel). Used to enforce no-override.
    #[inline]
    pub fn is_strongest(&self) -> bool {
        self.id == KERNEL_NODE_ID
    }
}

/// Lightweight view for backward compatibility and iteration.
#[derive(Debug, Clone)]
pub struct TsNode {
    pub id: NodeId,
    pub kind: NodeKind,
    pub weight: NodeWeight,
    pub name: String,
}

impl TsNode {
    pub fn from_info(info: &NodeInfo) -> Self {
        Self {
            id: info.id,
            kind: info.kind.clone(),
            weight: info.weight as NodeWeight,
            name: info.name.clone(),
        }
    }

    #[inline]
    pub fn is_strongest(&self) -> bool {
        self.id == KERNEL_NODE_ID
    }
}

/// Central authority for all TS nodes. Holds HashMap<NodeId, NodeInfo>.
/// Kernel is auto-inserted at weight 1.0; all other nodes must register and have weight < 1.0.
pub struct TsRegistry {
    nodes: RwLock<HashMap<NodeId, NodeInfo>>,
    next_id: RwLock<NodeId>,
}

impl TsRegistry {
    pub fn new() -> Self {
        let nodes = RwLock::new(HashMap::new());
        let mut map = nodes.write().unwrap();
        map.insert(
            KERNEL_NODE_ID,
            NodeInfo {
                id: KERNEL_NODE_ID,
                kind: NodeKind::Kernel,
                name: "kernel".into(),
                weight: 1.0_f32,
                parent: None,
                dependencies: vec![],
                status: NodeStatus::Active,
            },
        );
        drop(map);
        Self {
            nodes,
            next_id: RwLock::new(KERNEL_NODE_ID + 1),
        }
    }

    /// Register a new node with full info. Kernel (id 0) already exists; cannot register again.
    /// TS: weight must be in (0.0, 1.0); kernel is the only 1.0. No override — nothing may equal kernel.
    pub fn register_node(
        &self,
        kind: NodeKind,
        name: String,
        weight: f32,
        parent: Option<NodeId>,
        dependencies: Vec<NodeId>,
    ) -> Result<NodeId, KernelError> {
        if weight >= 1.0_f32 {
            return Err(KernelError::InvalidArgument);
        }
        assert!(weight < 1.0_f32, "TS no-override: no node may have weight >= 1.0 except kernel");
        let mut next = self.next_id.write().unwrap();
        let id = *next;
        *next = next.saturating_add(1);
        drop(next);
        let info = NodeInfo {
            id,
            kind,
            name,
            weight,
            parent,
            dependencies,
            status: NodeStatus::Active,
        };
        self.nodes.write().unwrap().insert(id, info);
        Ok(id)
    }

    /// Convenience: register with just kind, weight, name (parent=None, deps=[]).
    pub fn register(&self, kind: NodeKind, weight: NodeWeight, name: String) -> Result<NodeId, KernelError> {
        let w = weight as f32;
        if w >= 1.0_f32 {
            return Err(KernelError::InvalidArgument);
        }
        self.register_node(kind, name, w, None, vec![])
    }

    /// Get full node info. Used by resolve_conflict and hierarchy dump.
    pub fn get_info(&self, id: NodeId) -> Option<NodeInfo> {
        self.nodes.read().unwrap().get(&id).cloned()
    }

    /// Get node by id (as TsNode for backward compat).
    pub fn get(&self, id: NodeId) -> Option<TsNode> {
        self.nodes.read().unwrap().get(&id).map(TsNode::from_info)
    }

    /// Weight of node relative to strongest. Kernel returns 1.0. Used for scheduling and security.
    /// TS no-override: kernel weight is always 1.0 and must never be changed.
    pub fn get_weight(&self, id: NodeId) -> Option<NodeWeight> {
        let nodes = self.nodes.read().unwrap();
        let w = nodes.get(&id).map(|n| n.weight as NodeWeight);
        if id == KERNEL_NODE_ID {
            assert!(w == Some(WEIGHT_STRONGEST), "TS: kernel weight must remain 1.0");
        }
        w
    }

    /// Alias for get_weight (backward compatibility).
    pub fn weight_of(&self, id: NodeId) -> Option<NodeWeight> {
        self.get_weight(id)
    }

    /// TS conflict resolution: returns the winning node id.
    /// No override: kernel always wins if involved; otherwise higher weight wins (tie: first wins).
    pub fn resolve_conflict(&self, node_a: NodeId, node_b: NodeId) -> NodeId {
        if node_a == node_b {
            return node_a;
        }
        if node_a == KERNEL_NODE_ID {
            return KERNEL_NODE_ID;
        }
        if node_b == KERNEL_NODE_ID {
            return KERNEL_NODE_ID;
        }
        let nodes = self.nodes.read().unwrap();
        let wa = nodes.get(&node_a).map(|n| n.weight).unwrap_or(0.0);
        let wb = nodes.get(&node_b).map(|n| n.weight).unwrap_or(0.0);
        if wa >= wb {
            node_a
        } else {
            node_b
        }
    }

    /// All nodes as NodeInfo (for tree dump and internal use).
    pub fn all_node_infos(&self) -> Vec<NodeInfo> {
        self.nodes.read().unwrap().values().cloned().collect()
    }

    /// Iterate all nodes as TsNode (backward compat).
    pub fn all_nodes(&self) -> Vec<TsNode> {
        self.nodes
            .read()
            .unwrap()
            .values()
            .map(TsNode::from_info)
            .collect()
    }
}

impl Default for TsRegistry {
    fn default() -> Self {
        Self::new()
    }
}
