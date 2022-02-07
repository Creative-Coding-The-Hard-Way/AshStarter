use ::ash::vk;

use crate::vulkan::device_allocator::{Allocation, AllocatorError};

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

impl ComposableAllocator for Box<dyn ComposableAllocator> {
    unsafe fn allocate(
        &mut self,
        allocate_info: vk::MemoryAllocateInfo,
        size_in_bytes: u64,
    ) -> std::result::Result<Allocation, AllocatorError> {
        self.as_mut().allocate(allocate_info, size_in_bytes)
    }

    unsafe fn free(
        &mut self,
        allocation: &super::Allocation,
    ) -> Result<(), AllocatorError> {
        self.as_mut().free(allocation)
    }
}
