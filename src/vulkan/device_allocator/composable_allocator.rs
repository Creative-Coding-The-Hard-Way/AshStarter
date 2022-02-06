use ::ash::vk;

use super::{Allocation, AllocatorError, ComposableAllocator};

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
