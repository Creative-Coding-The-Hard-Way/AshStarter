//! This module defines traits and implementations for managing device (gpu)
//! memory.

mod allocation;
mod buffer_allocator;
mod device_allocator;
mod passthrough;

use crate::vulkan::{Buffer, RenderDevice};
use ash::vk;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum DeviceAllocatorError {
    #[error("failed to allocate memory using the Vulkan device")]
    LogicalDeviceAllocationFailed(#[source] vk::Result),

    #[error("no memory type could be found for flags {:?} and requirements {:?}", .0, .1)]
    MemoryTypeNotFound(vk::MemoryPropertyFlags, vk::MemoryRequirements),

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

    #[error("Unable to bind device memory to buffer")]
    UnableToBindDeviceMemory(#[source] vk::Result),
}

/// A single allocated piece of device memory.
#[derive(Clone, Debug, Copy, PartialEq, Eq)]
pub struct Allocation {
    pub memory: vk::DeviceMemory,
    pub offset: vk::DeviceSize,
    pub byte_size: vk::DeviceSize,
    memory_type_index: u32,
}

/// The external device memory allocation interface. This is the api used by
/// applications to allocate and free memory on the gpu.
pub trait DeviceAllocator {
    /// Allocate device memory with the provided type index and size.
    ///
    /// # unsafe because
    ///
    /// - it is the responsibility of the caller to free the returned memory
    ///   when it is no longer in use
    /// - implementations do not generally check that the memory type index in
    ///   allocate_info is the correct memory type index, the arguments are
    ///   assumed to be correct
    unsafe fn allocate(
        &mut self,
        vk_dev: &RenderDevice,
        allocate_info: vk::MemoryAllocateInfo,
        alignment: u64,
    ) -> Result<Allocation, DeviceAllocatorError>;

    /// Free an allocated piece of device memory.
    ///
    /// # unsafe because
    ///
    /// - it is the responsibility of the caller to know when the GPU is no
    ///   longer using the allocation
    unsafe fn free(
        &mut self,
        vk_dev: &RenderDevice,
        allocation: &Allocation,
    ) -> Result<(), DeviceAllocatorError>;
}

/// Types which implement this trait can allocate memory when given specific
/// requirements and properties.
pub trait BufferAllocator {
    /// Allocate a chunk of memory with the given requirements.
    unsafe fn allocate_memory(
        &mut self,
        vk_dev: &RenderDevice,
        memory_requirements: vk::MemoryRequirements,
        property_flags: vk::MemoryPropertyFlags,
    ) -> Result<Allocation, DeviceAllocatorError>;

    /// Create a Vulkan buffer with associated memory.
    fn create_buffer(
        &mut self,
        vk_dev: &RenderDevice,
        buffer_usage_flags: vk::BufferUsageFlags,
        memory_property_flags: vk::MemoryPropertyFlags,
        size_in_bytes: u64,
    ) -> Result<Buffer, DeviceAllocatorError>;

    /// Destroy a Vulkan buffer.
    ///
    /// # unsafe
    ///
    /// - because the application must synchronize both GPU and CPU access to
    ///   this buffer to ensure it's not in use when destroyed.
    unsafe fn destroy_buffer(
        &mut self,
        vk_dev: &RenderDevice,
        buffer: &mut Buffer,
    ) -> Result<(), DeviceAllocatorError>;
}

/// Create the default system memory allocator.
pub fn create_default_allocator() -> Box<dyn DeviceAllocator> {
    Box::new(passthrough::PassthroughAllocator::new())
}
