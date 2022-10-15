mod allocation;
mod passthrough_allocator;

use {crate::graphics::vulkan_api::VulkanError, ash::vk};

pub use self::{
    allocation::Allocation, passthrough_allocator::PassthroughAllocator,
};

pub(super) type Allocator = PassthroughAllocator;

/// Create the system allocator to be used by the RenderDevice for getting and
/// freeing device memory.
pub(super) fn create_system_allocator(
    logical_device: ash::Device,
) -> Allocator {
    PassthroughAllocator::new(logical_device)
}

/// The interface for composable GPU Memory Allocators.
pub(super) trait GPUMemoryAllocator {
    /// Allocate a block of device memory.
    ///
    /// # Safety
    ///
    /// Unsafe because the caller is responsible for calling free when the
    /// memory is no longer needed.
    unsafe fn allocate(
        &mut self,
        allocate_info: vk::MemoryAllocateInfo,
        alignment: u64,
    ) -> Result<Allocation, VulkanError>;

    /// Free an allocated piece of device memory.
    ///
    /// # Safety
    ///
    /// Unsafe because the caller must ensure that no GPU operations refer to
    /// the allocation.
    unsafe fn free(
        &mut self,
        allocation: &Allocation,
    ) -> Result<(), VulkanError>;
}

impl GPUMemoryAllocator for Box<dyn GPUMemoryAllocator> {
    unsafe fn allocate(
        &mut self,
        allocate_info: vk::MemoryAllocateInfo,
        size_in_bytes: u64,
    ) -> std::result::Result<Allocation, VulkanError> {
        self.as_mut().allocate(allocate_info, size_in_bytes)
    }

    unsafe fn free(
        &mut self,
        allocation: &super::Allocation,
    ) -> Result<(), VulkanError> {
        self.as_mut().free(allocation)
    }
}
