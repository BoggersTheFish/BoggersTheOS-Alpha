//! Hardware Abstraction Layer traits. All hardware is a node; the kernel
//! interacts only through these interfaces to maximise throughput and minimise latency.

use crate::error::KernelError;

/// CPU node: core compute. Implementations can be real (bare metal) or simulated.
pub trait CpuNode: Send + Sync {
    /// Number of logical cores (for scheduling).
    fn core_count(&self) -> u32;
    /// Halt until next interrupt (optional).
    fn halt(&self) {}
    /// Yield to scheduler (optional).
    fn yield_now(&self) {}
}

/// Storage node: block read/write. Used by file system and swap.
pub trait StorageNode: Send + Sync {
    /// Block size in bytes.
    fn block_size(&self) -> u64;
    /// Total size in bytes.
    fn size(&self) -> u64;
    /// Read block at offset (in blocks). Returns bytes.
    fn read_blocks(&self, block_offset: u64, blocks: u64) -> Result<Vec<u8>, KernelError>;
    /// Write blocks.
    fn write_blocks(&self, block_offset: u64, data: &[u8]) -> Result<(), KernelError>;
}

/// RAM node: physical memory. On hosted/simulated OS we may use a virtual view.
pub trait RamNode: Send + Sync {
    /// Total usable RAM in bytes.
    fn total_bytes(&self) -> u64;
    /// Available (free) bytes.
    fn available_bytes(&self) -> u64;
}

/// Network interface node: send/receive packets.
pub trait NetworkNode: Send + Sync {
    /// Max transmission unit.
    fn mtu(&self) -> u16;
    /// Send raw packet. Returns bytes sent.
    fn send(&self, data: &[u8]) -> Result<usize, KernelError>;
    /// Receive into buffer. Returns bytes read, or 0 if none.
    fn receive(&self, buffer: &mut [u8]) -> Result<usize, KernelError>;
}

/// Aggregate HAL: the kernel holds one HAL implementation. All drivers plug in as nodes.
pub trait Hal: Send + Sync {
    fn cpu(&self) -> &dyn CpuNode;
    fn storage(&self) -> Option<&dyn StorageNode> {
        None
    }
    fn ram(&self) -> Option<&dyn RamNode> {
        None
    }
    fn network(&self) -> Option<&dyn NetworkNode> {
        None
    }
}
