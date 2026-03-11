//! System call interface: user ↔ kernel boundary. All syscalls reference the kernel.
//! TS: syscalls are weighted nodes that propagate requests to the strongest node.

use boggers_kernel::{
    error::KernelError,
    node::NodeId,
    process::ProcessId,
    security::{Privilege, SecurityContext},
};
use std::sync::Arc;

/// Syscall numbers. Each is a distinct entry point into the kernel.
#[repr(u32)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SyscallNumber {
    /// Exit current process
    Exit = 0,
    /// Get process id
    GetPid = 1,
    /// Spawn process (kernel node only in full impl)
    Spawn = 2,
    /// Yield to scheduler
    Yield = 3,
    /// Allocate memory (node_id, size)
    Alloc = 4,
    /// Deallocate memory (base)
    Dealloc = 5,
    /// Log message (debug)
    Log = 6,
}

/// Result of a system call.
pub type SyscallResult = Result<SyscallReturn, KernelError>;

#[derive(Debug, Clone)]
pub enum SyscallReturn {
    Unit,
    Pid(ProcessId),
    Address(u64),
    Size(usize),
}

/// System call handler: holds references to kernel subsystems and dispatches syscalls.
/// TS: every invocation is evaluated relative to the kernel; security context is checked first.
pub struct SyscallHandler {
    pub node_id: NodeId,
    pub security: Arc<boggers_kernel::SecurityMonitor>,
    pub scheduler: Arc<boggers_kernel::Scheduler>,
    pub memory: Arc<boggers_kernel::MemoryManager>,
}

impl SyscallHandler {
    /// Dispatch a syscall. Returns result or error. Caller must be the current process.
    pub fn dispatch(
        &self,
        ctx: &SecurityContext,
        num: SyscallNumber,
        args: &[u64],
    ) -> SyscallResult {
        match num {
            SyscallNumber::Exit => {
                let pid = self.scheduler.current().ok_or(KernelError::InvalidNode)?;
                self.scheduler.terminate(pid)?;
                Ok(SyscallReturn::Unit)
            }
            SyscallNumber::GetPid => {
                let pid = self.scheduler.current().ok_or(KernelError::InvalidNode)?;
                Ok(SyscallReturn::Pid(pid))
            }
            SyscallNumber::Yield => {
                let _ = self.scheduler.schedule();
                Ok(SyscallReturn::Unit)
            }
            SyscallNumber::Alloc => {
                if args.len() < 2 {
                    return Err(KernelError::InvalidArgument);
                }
                let node_id = args[0] as NodeId;
                let size = args[1];
                self.security.check_access(ctx, 0, "memory")?;
                let base = self.memory.allocate(size, node_id, true, false)?;
                Ok(SyscallReturn::Address(base))
            }
            SyscallNumber::Dealloc => {
                if args.is_empty() {
                    return Err(KernelError::InvalidArgument);
                }
                let base = args[0];
                self.security.check_access(ctx, 0, "memory")?;
                self.memory.deallocate(base)?;
                Ok(SyscallReturn::Unit)
            }
            SyscallNumber::Log => {
                // In real OS we'd pass a message pointer/size; here we ignore.
                Ok(SyscallReturn::Unit)
            }
            SyscallNumber::Spawn => {
                if ctx.privilege != Privilege::Kernel {
                    return Err(KernelError::PermissionDenied);
                }
                if args.len() < 2 {
                    return Err(KernelError::InvalidArgument);
                }
                let node_id = args[0] as NodeId;
                // name would come from user buffer in real impl
                let pid = self.scheduler.spawn(node_id, "user".into())?;
                Ok(SyscallReturn::Pid(pid))
            }
        }
    }
}
