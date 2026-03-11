//! Hardware Abstraction Layer. All hardware is a node; drivers implement these traits.
//! TS: HAL exists to maximise throughput and minimise latency under kernel control.

use boggers_kernel::hal_traits::{CpuNode, Hal, RamNode, StorageNode};
use boggers_kernel::KernelError;
use std::sync::Arc;

/// Simulated CPU for hosted/skeleton OS. Real HAL would read from /proc or bare metal.
pub struct SimCpu {
    cores: u32,
}

impl SimCpu {
    pub fn new(cores: u32) -> Self {
        Self { cores: cores.max(1) }
    }
}

impl CpuNode for SimCpu {
    fn core_count(&self) -> u32 {
        self.cores
    }
}

/// Simulated RAM node (e.g. from host process).
pub struct SimRam {
    total_bytes: u64,
    available_bytes: u64,
}

impl SimRam {
    pub fn new(total_bytes: u64, available_bytes: u64) -> Self {
        Self {
            total_bytes,
            available_bytes,
        }
    }
}

impl RamNode for SimRam {
    fn total_bytes(&self) -> u64 {
        self.total_bytes
    }
    fn available_bytes(&self) -> u64 {
        self.available_bytes
    }
}

/// Simulated block storage (in-memory for skeleton).
pub struct SimStorage {
    block_size: u64,
    data: std::sync::RwLock<Vec<u8>>,
}

impl SimStorage {
    pub fn new(block_size: u64, size_blocks: u64) -> Self {
        let len = (block_size * size_blocks) as usize;
        Self {
            block_size,
            data: std::sync::RwLock::new(vec![0u8; len]),
        }
    }
}

impl StorageNode for SimStorage {
    fn block_size(&self) -> u64 {
        self.block_size
    }
    fn size(&self) -> u64 {
        self.data.read().unwrap().len() as u64
    }
    fn read_blocks(&self, block_offset: u64, blocks: u64) -> Result<Vec<u8>, KernelError> {
        let start = (block_offset * self.block_size) as usize;
        let len = (blocks * self.block_size) as usize;
        let data = self.data.read().unwrap();
        if start + len > data.len() {
            return Err(KernelError::InvalidArgument);
        }
        Ok(data[start..start + len].to_vec())
    }
    fn write_blocks(&self, block_offset: u64, buf: &[u8]) -> Result<(), KernelError> {
        let start = (block_offset * self.block_size) as usize;
        let mut data = self.data.write().unwrap();
        if start + buf.len() > data.len() {
            return Err(KernelError::InvalidArgument);
        }
        data[start..start + buf.len()].copy_from_slice(buf);
        Ok(())
    }
}

/// Default HAL implementation for the OS skeleton (hosted).
pub struct DefaultHal {
    cpu: Arc<SimCpu>,
    ram: Option<Arc<SimRam>>,
    storage: Option<Arc<SimStorage>>,
}

impl DefaultHal {
    pub fn new() -> Self {
        Self {
            cpu: Arc::new(SimCpu::new(4)),
            ram: Some(Arc::new(SimRam::new(512 * 1024 * 1024, 256 * 1024 * 1024))),
            storage: Some(Arc::new(SimStorage::new(512, 4096))),
        }
    }
}

impl Default for DefaultHal {
    fn default() -> Self {
        Self::new()
    }
}

impl Hal for DefaultHal {
    fn cpu(&self) -> &dyn CpuNode {
        self.cpu.as_ref()
    }
    fn ram(&self) -> Option<&dyn RamNode> {
        self.ram.as_ref().map(|r| r.as_ref() as &dyn RamNode)
    }
    fn storage(&self) -> Option<&dyn StorageNode> {
        self.storage.as_ref().map(|s| s.as_ref() as &dyn StorageNode)
    }
}
