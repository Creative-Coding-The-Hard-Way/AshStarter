//! This module defines traits and implementations for managing device (gpu)
//! memory.

mod allocation;
mod allocator_error;
mod composable_allocator;
mod locked_memory_allocator;
mod passthrough_allocator;

use ::{ash::vk, std::sync::Arc};

pub use self::{
    allocation::Allocation, allocator_error::AllocatorError,
    composable_allocator::ComposableAllocator,
    locked_memory_allocator::LockedMemoryAllocator,
    passthrough_allocator::PassthroughAllocator,
};
use crate::vulkan::RenderDevice;

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
