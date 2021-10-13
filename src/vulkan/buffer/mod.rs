mod buffer;
mod gpu_vec;

use crate::vulkan::{errors::DeviceAllocatorError, Allocation};

use ash::vk;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum BufferError {
    #[error("Unable to map device memory")]
    UnableToMapDeviceMemory(#[source] vk::Result),

    #[error(
        "Device memory pointer was not found, did you try calling .map()?"
    )]
    NoMappedPointerFound,

    #[error(transparent)]
    UnableToAllocateBuffer(#[from] DeviceAllocatorError),
}

/// A Vulkan buffer and it's associated device memory.
pub struct Buffer {
    /// The underlying Vulkan buffer type
    pub raw: vk::Buffer,

    /// The actual memory alloctaion for this buffer
    pub allocation: Allocation,

    /// The pointer to the cpu-accessible memory-mapped region of memory for
    /// this buffer. Only valid after a call to map().
    pub mapped_ptr: Option<*mut std::ffi::c_void>,
}

/// A resizable GPU Buffer which holds a contiguous slice of T's.
pub struct GpuVec<T: Copy> {
    buffer: Buffer,
    usage_flags: vk::BufferUsageFlags,
    _phantom_data: std::marker::PhantomData<T>,
}
