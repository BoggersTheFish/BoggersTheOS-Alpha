//! System call interface: user ↔ kernel boundary. All syscalls reference the kernel.
//! TS: every syscall checks caller's node weight vs required min_weight; kernel always allowed.

use boggers_kernel::{
    error::KernelError,
    node::{NodeId, KERNEL_NODE_ID},
    process::ProcessId,
    security::{Privilege, SecurityContext},
};
use std::sync::Arc;

/// Syscall numbers. Each has a TS min_weight (see min_weight_for_syscall).
#[repr(u32)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SyscallNumber {
    Exit = 0,
    GetPid = 1,
    Spawn = 2,
    Yield = 3,
    Alloc = 4,
    Dealloc = 5,
    Log = 6,
    GetNodeWeight = 7,
    YieldToStronger = 8,
    Print = 9,
}

/// TS: minimum node weight required to perform this syscall. Kernel (1.0) always allowed.
fn min_weight_for_syscall(num: SyscallNumber) -> f64 {
    use SyscallNumber::*;
    match num {
        Exit | GetPid | Yield | YieldToStronger => 0.2,
        Alloc | Dealloc | GetNodeWeight => 0.3,
        Print => 0.4,
        Log => 0.5,
        Spawn => 1.0, // kernel only
    }
}

/// Result of a system call.
pub type SyscallResult = Result<SyscallReturn, KernelError>;

#[derive(Debug, Clone)]
pub enum SyscallReturn {
    Unit,
    Pid(ProcessId),
    Address(u64),
    Size(usize),
    Weight(f64),
}

/// System call handler. TS: before any operation we check caller weight >= min_weight for that syscall.
pub struct SyscallHandler {
    pub node_id: NodeId,
    pub registry: Arc<boggers_kernel::TsRegistry>,
    pub security: Arc<boggers_kernel::SecurityMonitor>,
    pub scheduler: Arc<boggers_kernel::Scheduler>,
    pub memory: Arc<boggers_kernel::MemoryManager>,
}

impl SyscallHandler {
    /// TS: reject if caller's weight is below the syscall's min_weight. Kernel (id 0) always passes.
    /// No override: we never allow a lower-weight node to perform a syscall requiring higher weight.
    fn check_weight(&self, ctx: &SecurityContext, num: SyscallNumber) -> Result<(), KernelError> {
        if ctx.node_id == KERNEL_NODE_ID {
            return Ok(());
        }
        let min_w = min_weight_for_syscall(num);
        let caller_w = self.registry.get_weight(ctx.node_id).unwrap_or(0.0);
        if caller_w < min_w {
            let msg = format!(
                "syscall {:?} denied: node {} weight {:.3} < min {:.3}",
                num, ctx.node_id, caller_w, min_w
            );
            self.security.log_violation(&msg);
            return Err(KernelError::PermissionDenied);
        }
        Ok(())
    }

    pub fn dispatch(
        &self,
        ctx: &SecurityContext,
        num: SyscallNumber,
        args: &[u64],
    ) -> SyscallResult {
        self.check_weight(ctx, num)?;

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
            SyscallNumber::Log => Ok(SyscallReturn::Unit),
            SyscallNumber::Spawn => {
                if ctx.privilege != Privilege::Kernel {
                    return Err(KernelError::PermissionDenied);
                }
                if args.len() < 2 {
                    return Err(KernelError::InvalidArgument);
                }
                let node_id = args[0] as NodeId;
                let pid = self.scheduler.spawn(node_id, "user".into())?;
                Ok(SyscallReturn::Pid(pid))
            }
            SyscallNumber::GetNodeWeight => {
                let query_id = args.get(0).copied().unwrap_or(ctx.node_id as u64) as NodeId;
                let w = self.registry.get_weight(query_id).unwrap_or(0.0);
                Ok(SyscallReturn::Weight(w))
            }
            SyscallNumber::YieldToStronger => {
                // TS: yield is always to the scheduler; "stronger" is enforced by scheduler picking by weight.
                let _ = self.scheduler.schedule();
                Ok(SyscallReturn::Unit)
            }
            SyscallNumber::Print => {
                // Simple print: args could be length + pointer in real OS; here we just allow and return.
                Ok(SyscallReturn::Unit)
            }
        }
    }
}
