use crate::vulkan::RenderDevice;

use super::{
    Allocation, AllocatorError, ComposableAllocator, PassthroughAllocator,
};

use ::{
    ash::{version::DeviceV1_0, vk},
    std::sync::Arc,
};

impl PassthroughAllocator {
    pub fn new(vk_dev: Arc<RenderDevice>) -> Self {
        Self { vk_dev }
    }
}

impl ComposableAllocator for PassthroughAllocator {
    /// Directly allocate device memory onto the heap indicated by the
    /// memory type index of the `allocate_info` struct.
    unsafe fn allocate(
        &mut self,
        allocate_info: vk::MemoryAllocateInfo,
        _alignment: u64,
    ) -> Result<Allocation, AllocatorError> {
        // No special attention is required for handling alignment because
        // memory allocated by the logical device will always be aligned to the
        // LCM of the memory alignment requirements for this system. See the
        // notes here:
        // https://www.khronos.org/registry/vulkan/specs/1.2-extensions/man/html/vkAllocateMemory.html
        Ok(Allocation {
            memory: self
                .vk_dev
                .logical_device
                .allocate_memory(&allocate_info, None)
                .map_err(AllocatorError::LogicalDeviceAllocationFailed)?,
            offset: 0,
            byte_size: allocate_info.allocation_size,
            memory_type_index: allocate_info.memory_type_index,
        })
    }

    /// Free the device memory backing the allocation.
    unsafe fn free(
        &mut self,
        allocation: &Allocation,
    ) -> Result<(), AllocatorError> {
        self.vk_dev
            .logical_device
            .free_memory(allocation.memory, None);
        Ok(())
    }
}
