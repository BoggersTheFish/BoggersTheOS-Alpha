//! Memory management. TS: kernel owns allocation policy; regions are nodes
//! weighted by proximity to kernel (e.g. kernel heap > user heap).

use crate::error::KernelError;
use crate::node::NodeId;
use std::sync::RwLock;

/// A contiguous memory region. Physical addresses would be used on bare metal.
#[derive(Debug, Clone)]
pub struct MemoryRegion {
    pub base: u64,
    pub size: u64,
    /// TS: node that owns or primarily uses this region (for isolation and prioritisation).
    pub node_id: NodeId,
    pub writable: bool,
    pub executable: bool,
}

impl MemoryRegion {
    pub fn contains(&self, addr: u64) -> bool {
        addr >= self.base && addr < self.base.saturating_add(self.size)
    }
}

/// Central memory manager. Tracks regions and allocates from a simple pool.
/// TS: allocation requests carry node_id; kernel (node 0) gets highest priority under pressure.
pub struct MemoryManager {
    regions: RwLock<Vec<MemoryRegion>>,
    /// Simple heap simulation: next alloc offset. Real impl would have proper allocator.
    next_alloc: RwLock<u64>,
    heap_end: u64,
}

impl MemoryManager {
    /// heap_end: end of available "heap" for this manager (e.g. 0x1000_0000 for 16MiB).
    pub fn new(heap_end: u64) -> Self {
        Self {
            regions: RwLock::new(Vec::new()),
            next_alloc: RwLock::new(0x1000),
            heap_end,
        }
    }

    /// Allocate a region for the given node. Returns base address.
    pub fn allocate(&self, size: u64, node_id: NodeId, writable: bool, executable: bool) -> Result<u64, KernelError> {
        let size = size.max(8).next_power_of_two();
        let mut next = self.next_alloc.write().unwrap();
        let base = *next;
        *next = base.saturating_add(size);
        if *next > self.heap_end {
            return Err(KernelError::ResourceExhausted);
        }
        drop(next);
        let region = MemoryRegion {
            base,
            size,
            node_id,
            writable,
            executable,
        };
        self.regions.write().unwrap().push(region);
        Ok(base)
    }

    /// Deallocate region at base (simplified: mark for reuse; full impl would coalesce).
    pub fn deallocate(&self, base: u64) -> Result<(), KernelError> {
        let mut regions = self.regions.write().unwrap();
        if let Some(pos) = regions.iter().position(|r| r.base == base) {
            regions.remove(pos);
            return Ok(());
        }
        Err(KernelError::InvalidArgument)
    }

    /// All regions (e.g. for security or dump).
    pub fn regions(&self) -> Vec<MemoryRegion> {
        self.regions.read().unwrap().clone()
    }

    /// Find region containing address. Used for permission checks.
    pub fn find_region(&self, addr: u64) -> Option<MemoryRegion> {
        self.regions.read().unwrap().iter().find(|r| r.contains(addr)).cloned()
    }
}
