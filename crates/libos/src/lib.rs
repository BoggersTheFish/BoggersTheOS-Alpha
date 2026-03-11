//! OS library: user-facing API. All functions go through syscall (which references kernel).
//! TS: each lib call is a weighted node affecting OS state while preserving kernel integrity.

use boggers_kernel::{
    node::NodeId,
    security::SecurityContext,
};
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

    /// Exit the current process (via syscall).
    pub fn exit(handler: &SyscallHandler, ctx: &SecurityContext) -> Result<(), boggers_kernel::KernelError> {
        handler.dispatch(ctx, SyscallNumber::Exit, &[]).map(|_| ())
    }

    /// Yield to scheduler.
    pub fn yield_to_scheduler(handler: &SyscallHandler, ctx: &SecurityContext) -> Result<(), boggers_kernel::KernelError> {
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
