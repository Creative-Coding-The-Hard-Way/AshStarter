use super::{Allocation, DeviceAllocator, DeviceAllocatorError};

use crate::vulkan::RenderDevice;

use ::ash::vk;

impl DeviceAllocator for Box<dyn DeviceAllocator> {
    unsafe fn allocate(
        &mut self,
        vk_dev: &RenderDevice,
        allocate_info: vk::MemoryAllocateInfo,
        size_in_bytes: u64,
    ) -> std::result::Result<Allocation, DeviceAllocatorError> {
        self.as_mut().allocate(vk_dev, allocate_info, size_in_bytes)
    }

    unsafe fn free(
        &mut self,
        vk_dev: &crate::vulkan::RenderDevice,
        allocation: &super::Allocation,
    ) -> Result<(), DeviceAllocatorError> {
        self.as_mut().free(vk_dev, allocation)
    }
}
