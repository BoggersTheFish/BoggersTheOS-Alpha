//! # BoggersTheOS Kernel — Strongest Node (TS Core)
//!
//! Every other component emerges and self-organises around this core.
//! All node weighting, scheduling, and resource logic flows from here.

pub mod node;
pub mod process;
pub mod memory;
pub mod hal_traits;
pub mod security;
pub mod error;

pub use node::{NodeId, NodeWeight, TsNode, TsRegistry};
pub use process::{ProcessId, ProcessState, Scheduler};
pub use memory::{MemoryRegion, MemoryManager};
pub use hal_traits::Hal;
pub use security::{SecurityContext, SecurityMonitor};
pub use error::KernelError;
