//! Drivers: secondary nodes. Each driver references kernel + HAL and registers as a TS node.
//! TS: drivers register device nodes (sim-uart, sim-timer) with weights 0.7–0.85; kernel is parent.

use boggers_hal::{DefaultHal, SimTimer, SimUart};
use boggers_kernel::node::{NodeKind, TsRegistry, KERNEL_NODE_ID};
use boggers_kernel::KernelError;
use std::sync::Arc;

/// Driver layer weight: below kernel, above syscall.
const DRIVER_WEIGHT: f32 = 0.85;
/// Sim devices slightly lower than core drivers but still high (TS hierarchy).
const SIM_DEVICE_WEIGHT: f32 = 0.78;

/// Register all skeleton drivers and sim devices in TsRegistry; return HAL.
/// TS: kernel is parent for all driver nodes; no override of kernel.
pub fn init_drivers(registry: Arc<TsRegistry>) -> Result<Arc<DefaultHal>, KernelError> {
    let parent = Some(KERNEL_NODE_ID);

    let _cpu = registry.register_node(
        NodeKind::Driver,
        "cpu".into(),
        DRIVER_WEIGHT,
        parent,
        vec![KERNEL_NODE_ID],
    )?;
    let _storage = registry.register_node(
        NodeKind::Driver,
        "storage".into(),
        DRIVER_WEIGHT,
        parent,
        vec![KERNEL_NODE_ID],
    )?;
    let _ram = registry.register_node(
        NodeKind::Driver,
        "ram".into(),
        DRIVER_WEIGHT,
        parent,
        vec![KERNEL_NODE_ID],
    )?;

    let _sim_uart = registry.register_node(
        NodeKind::Driver,
        "sim-uart".into(),
        SIM_DEVICE_WEIGHT,
        parent,
        vec![KERNEL_NODE_ID],
    )?;
    let _sim_timer = registry.register_node(
        NodeKind::Driver,
        "sim-timer".into(),
        SIM_DEVICE_WEIGHT,
        parent,
        vec![KERNEL_NODE_ID],
    )?;

    let _uart = SimUart::new();
    let _timer = SimTimer::new();

    Ok(Arc::new(DefaultHal::new()))
}
