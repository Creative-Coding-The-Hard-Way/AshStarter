mod buffer;
mod gpu_vec;

use crate::vulkan::{
    errors::AllocatorError, Allocation, MemoryAllocator, RenderDevice,
};

use ::{ash::vk, std::sync::Arc, thiserror::Error};

#[derive(Debug, Error)]
pub enum BufferError {
    #[error("Unable to map device memory")]
    UnableToMapDeviceMemory(#[source] vk::Result),

    #[error(
        "Device memory pointer was not found, did you try calling .map()?"
    )]
    NoMappedPointerFound,

    #[error(
        "Unable to create a new device buffer for {} bytes with flags {:?}",
        .size,
        .usage
    )]
    UnableToCreateBuffer {
        size: u64,
        usage: vk::BufferUsageFlags,
        source: vk::Result,
    },

    #[error(transparent)]
    UnableToAllocateBufferMemory(#[from] AllocatorError),

    #[error("Unable to bind device memory to buffer")]
    UnableToBindDeviceMemory(#[source] vk::Result),
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

    /// The allocator implementation used to provision memory from the Vulkan
    /// device.
    pub vk_alloc: Arc<dyn MemoryAllocator>,

    /// The device used to create and destroy this buffer's memory.
    pub vk_dev: Arc<RenderDevice>,
}

/// A resizable GPU Buffer which holds a contiguous slice of T's.
pub struct GpuVec<T: Copy> {
    /// The device buffer which holds the actual data.
    pub buffer: Buffer,

    /// The number of elements (not bytes) which this buffer is able to hold.
    capacity: u32,

    /// The number of elements (not bytes) which this buffer is currently
    /// holding.
    length: u32,

    /// Buffer usage flags - used when the underlying buffer needs to be
    /// reallocated.
    usage_flags: vk::BufferUsageFlags,

    _phantom_data: std::marker::PhantomData<T>,
}
