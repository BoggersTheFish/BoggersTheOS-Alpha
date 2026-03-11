//! OS library: user-facing API. All functions go through syscall (which references kernel).
//! TS: each lib call is a weighted node affecting OS state while preserving kernel integrity.

use boggers_kernel::{node::NodeId, security::SecurityContext};
use boggers_syscall::{SyscallHandler, SyscallNumber, SyscallReturn};

/// High-level process handle. Uses syscall layer.
pub struct Process {
    pub pid: boggers_kernel::ProcessId,
    #[allow(dead_code)]
    ctx: SecurityContext,
}

impl Process {
    pub fn current_pid(handler: &SyscallHandler) -> Option<boggers_kernel::ProcessId> {
        handler.scheduler.current()
    }

    pub fn exit(
        handler: &SyscallHandler,
        ctx: &SecurityContext,
    ) -> Result<(), boggers_kernel::KernelError> {
        handler.dispatch(ctx, SyscallNumber::Exit, &[]).map(|_| ())
    }

    pub fn yield_to_scheduler(
        handler: &SyscallHandler,
        ctx: &SecurityContext,
    ) -> Result<(), boggers_kernel::KernelError> {
        handler.dispatch(ctx, SyscallNumber::Yield, &[]).map(|_| ())
    }
}

/// Memory allocation via kernel (syscall).
pub fn alloc(
    handler: &SyscallHandler,
    ctx: &SecurityContext,
    node_id: NodeId,
    size: u64,
) -> Result<u64, boggers_kernel::KernelError> {
    match handler.dispatch(ctx, SyscallNumber::Alloc, &[node_id as u64, size])? {
        SyscallReturn::Address(addr) => Ok(addr),
        _ => Err(boggers_kernel::KernelError::InternalError),
    }
}

pub fn dealloc(
    handler: &SyscallHandler,
    ctx: &SecurityContext,
    base: u64,
) -> Result<(), boggers_kernel::KernelError> {
    handler.dispatch(ctx, SyscallNumber::Dealloc, &[base]).map(|_| ())
}

/// Get TS weight for a node (or caller if node_id not provided). Fails if weight check denies.
pub fn get_node_weight(
    handler: &SyscallHandler,
    ctx: &SecurityContext,
    node_id: Option<NodeId>,
) -> Result<f64, boggers_kernel::KernelError> {
    let arg = node_id.map(|n| n as u64).unwrap_or(ctx.node_id as u64);
    match handler.dispatch(ctx, SyscallNumber::GetNodeWeight, &[arg])? {
        SyscallReturn::Weight(w) => Ok(w),
        _ => Err(boggers_kernel::KernelError::InternalError),
    }
}

/// Yield to scheduler; next process chosen by TS weight (stronger wins). No override.
pub fn yield_to_stronger(
    handler: &SyscallHandler,
    ctx: &SecurityContext,
) -> Result<(), boggers_kernel::KernelError> {
    handler.dispatch(ctx, SyscallNumber::YieldToStronger, &[]).map(|_| ())
}

/// Print syscall (allowed if caller weight >= min for Print). Message content not passed in skeleton.
pub fn print(
    handler: &SyscallHandler,
    ctx: &SecurityContext,
) -> Result<(), boggers_kernel::KernelError> {
    handler.dispatch(ctx, SyscallNumber::Print, &[]).map(|_| ())
}
