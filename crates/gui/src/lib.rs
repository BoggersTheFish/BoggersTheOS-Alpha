//! GUI: emergent UI node. Minimalism and modularity emerge from kernel rules.
//! TS: each user input propagates as a weighted node affecting OS state; kernel integrity preserved.

use boggers_kernel::node::{NodeKind, TsRegistry, NodeId};
use boggers_kernel::KernelError;
use std::sync::Arc;

const UI_WEIGHT: f64 = 0.5;

/// GUI subsystem node. Registers with TS and provides minimal event handling.
pub struct GuiNode {
    pub node_id: NodeId,
    #[allow(dead_code)]
    registry: Arc<TsRegistry>,
}

impl GuiNode {
    pub fn new(registry: Arc<TsRegistry>) -> Result<Self, KernelError> {
        let node_id = registry.register(
            NodeKind::Ui,
            UI_WEIGHT,
            "gui".into(),
        )?;
        Ok(Self { node_id, registry })
    }

    /// Process a user input event (e.g. key press). Returns whether the event was consumed.
    /// TS: input propagates through this node; higher layers can prioritise by weight.
    pub fn on_input(&self, _key: &str) -> bool {
        // Minimal: no actual windowing yet. Just acknowledge.
        true
    }

    /// Render one frame (placeholder). In full OS would draw to framebuffer or compositor.
    pub fn render(&self) {
        // No-op for skeleton
    }
}
