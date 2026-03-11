//! Security as emergent behaviour. TS: security nodes protect the strongest node
//! (kernel) first, then dependent subsystems by weight.

use crate::error::KernelError;
use crate::node::{NodeId, KERNEL_NODE_ID};
use std::sync::RwLock;

/// Permission level. Kernel is highest; applications lowest.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum Privilege {
    Kernel = 0,
    Driver = 1,
    Syscall = 2,
    Library = 3,
    User = 4,
}

/// Security context for a process or operation. Used to allow/deny access.
#[derive(Debug, Clone)]
pub struct SecurityContext {
    pub node_id: NodeId,
    pub privilege: Privilege,
    /// Optional: which resource (e.g. path, device) is being accessed.
    pub resource: Option<String>,
}

impl SecurityContext {
    pub fn kernel() -> Self {
        Self {
            node_id: KERNEL_NODE_ID,
            privilege: Privilege::Kernel,
            resource: None,
        }
    }

    pub fn user(node_id: NodeId) -> Self {
        Self {
            node_id,
            privilege: Privilege::User,
            resource: None,
        }
    }

    /// Check whether this context may access a resource owned by another node.
    /// TS: kernel (strongest) can access all; others are restricted by privilege and policy.
    pub fn can_access(&self, resource_node_id: NodeId, _resource: &str) -> bool {
        if self.node_id == KERNEL_NODE_ID {
            return true;
        }
        if resource_node_id == KERNEL_NODE_ID {
            return self.privilege == Privilege::Kernel;
        }
        self.privilege <= Privilege::User
    }
}

/// Simple security monitor: logs and enforces. Can be extended with threat detection.
pub struct SecurityMonitor {
    violations: RwLock<Vec<String>>,
}

impl SecurityMonitor {
    pub fn new() -> Self {
        Self {
            violations: RwLock::new(Vec::new()),
        }
    }

    /// Check access: if denied, records violation and returns error.
    pub fn check_access(
        &self,
        ctx: &SecurityContext,
        resource_node_id: NodeId,
        resource: &str,
    ) -> Result<(), KernelError> {
        if ctx.can_access(resource_node_id, resource) {
            return Ok(());
        }
        let msg = format!(
            "access denied: node {} to resource {} (owner {})",
            ctx.node_id, resource, resource_node_id
        );
        self.violations.write().unwrap().push(msg.clone());
        Err(KernelError::PermissionDenied)
    }

    pub fn violations(&self) -> Vec<String> {
        self.violations.read().unwrap().clone()
    }
}

impl Default for SecurityMonitor {
    fn default() -> Self {
        Self::new()
    }
}
