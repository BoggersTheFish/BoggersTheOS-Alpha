//! Kernel error type. TS: errors propagate from strongest node.

use std::fmt;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum KernelError {
    /// Invalid node or reference
    InvalidNode,
    /// Resource exhausted (memory, handles)
    ResourceExhausted,
    /// Permission or security violation
    PermissionDenied,
    /// Invalid argument from caller
    InvalidArgument,
    /// Device or driver error
    DeviceError,
    /// Operation not supported
    Unsupported,
    /// Internal consistency (integrity of strongest node violated)
    InternalError,
}

impl fmt::Display for KernelError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidNode => write!(f, "invalid node"),
            Self::ResourceExhausted => write!(f, "resource exhausted"),
            Self::PermissionDenied => write!(f, "permission denied"),
            Self::InvalidArgument => write!(f, "invalid argument"),
            Self::DeviceError => write!(f, "device error"),
            Self::Unsupported => write!(f, "unsupported"),
            Self::InternalError => write!(f, "internal error"),
        }
    }
}

impl std::error::Error for KernelError {}
