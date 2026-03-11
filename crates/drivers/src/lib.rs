//! Drivers: secondary nodes. Each driver references kernel + HAL and registers as a TS node.
//! TS: drivers maximise throughput and minimise latency under kernel control.

use boggers_hal::DefaultHal;
use boggers_kernel::{node::{NodeKind, TsRegistry}, KernelError};
use std::sync::Arc;

/// Driver layer weight: below kernel, above syscall.
const DRIVER_WEIGHT: f64 = 0.85;

/// Register all skeleton drivers with the TS registry and return the HAL.
/// In a full OS, each driver would register itself and provide a HAL implementation.
pub fn init_drivers(registry: Arc<TsRegistry>) -> Result<Arc<DefaultHal>, KernelError> {
    let _cpu = registry.register(
        NodeKind::Driver,
        DRIVER_WEIGHT,
        "cpu".into(),
    )?;
    let _storage = registry.register(
        NodeKind::Driver,
        DRIVER_WEIGHT,
        "storage".into(),
    )?;
    let _ram = registry.register(
        NodeKind::Driver,
        DRIVER_WEIGHT,
        "ram".into(),
    )?;
    Ok(Arc::new(DefaultHal::new()))
}
