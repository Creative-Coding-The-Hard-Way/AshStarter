//! This module defines traits and implementations for managing device (gpu)
//! memory.

mod allocation;
mod composable_allocator;
mod locked_memory_allocator;
mod passthrough;

use crate::vulkan::RenderDevice;

use ::{
    ash::vk,
    std::sync::{Arc, Mutex},
    thiserror::Error,
};

#[derive(Debug, Error)]
pub enum AllocatorError {
    #[error("failed to allocate memory using the Vulkan device")]
    LogicalDeviceAllocationFailed(#[source] vk::Result),

    #[error("no memory type could be found for flags {:?} and requirements {:?}", .0, .1)]
    MemoryTypeNotFound(vk::MemoryPropertyFlags, vk::MemoryRequirements),
}

/// A single allocated piece of device memory.
#[derive(Clone, Debug, Copy, PartialEq, Eq)]
pub struct Allocation {
    pub memory: vk::DeviceMemory,
    pub offset: vk::DeviceSize,
    pub byte_size: vk::DeviceSize,
    memory_type_index: u32,
}

pub trait MemoryAllocator {
    /// Allocate GPU memory based on a given set of requirements.
    ///
    /// # unsafe
    ///
    /// - it is the responsibility of the caller to free the returned memory
    ///   when it is no longer in use
    unsafe fn allocate_memory(
        &self,
        memory_requirements: vk::MemoryRequirements,
        property_flags: vk::MemoryPropertyFlags,
    ) -> Result<Allocation, AllocatorError>;

    /// Free an allocated piece of device memory.
    ///
    /// # unsafe because
    ///
    /// - it is the responsibility of the caller to know when the GPU is no
    ///   longer using the allocation
    unsafe fn free(
        &self,
        allocation: &Allocation,
    ) -> Result<(), AllocatorError>;
}

/// The device memory allocation interface. This is the compositional API for
/// GPU memory allocation.
pub trait ComposableAllocator {
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
        allocate_info: vk::MemoryAllocateInfo,
        alignment: u64,
    ) -> Result<Allocation, AllocatorError>;

    /// Free an allocated piece of device memory.
    ///
    /// # unsafe because
    ///
    /// - it is the responsibility of the caller to know when the GPU is no
    ///   longer using the allocation
    unsafe fn free(
        &mut self,
        allocation: &Allocation,
    ) -> Result<(), AllocatorError>;
}

/// Create the default system memory allocator.
pub fn create_default_allocator(
    vk_dev: Arc<RenderDevice>,
) -> Arc<dyn MemoryAllocator> {
    let locked_allocator = LockedMemoryAllocator::new(
        vk_dev.clone(),
        PassthroughAllocator::new(vk_dev.clone()),
    );
    Arc::new(locked_allocator)
}

/// A memory allocator implementation which decorates a composed allocator
/// with a mutex.
pub struct LockedMemoryAllocator<Alloc: ComposableAllocator> {
    composed_allocator: Mutex<Alloc>,
    vk_dev: Arc<RenderDevice>,
}

/// A composable allocator which just defers all allocation to the underlying
/// Vulkan device.
pub struct PassthroughAllocator {
    vk_dev: Arc<RenderDevice>,
}
