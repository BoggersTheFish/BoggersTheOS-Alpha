//! Applications: top-level nodes. Depend on libos and optionally gui.
//! TS: every app is a node that self-evaluates relative to the strongest node (kernel).

use boggers_kernel::node::{NodeKind, TsRegistry, NodeId};
use boggers_kernel::KernelError;
use std::sync::Arc;

const APP_WEIGHT: f64 = 0.3;

/// Application node. Registers with TS and runs in user space.
pub struct AppNode {
    pub node_id: NodeId,
    pub name: String,
    #[allow(dead_code)]
    registry: Arc<TsRegistry>,
}

impl AppNode {
    pub fn new(registry: Arc<TsRegistry>, name: String) -> Result<Self, KernelError> {
        let node_id = registry.register(
            NodeKind::Application,
            APP_WEIGHT,
            name.clone(),
        )?;
        Ok(Self {
            node_id,
            name,
            registry,
        })
    }
}
