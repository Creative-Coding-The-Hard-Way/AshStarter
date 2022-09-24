use ash::vk;

use super::GPUMemoryAllocator;
use crate::graphics::vulkan_api::{Allocation, VulkanError};

/// A memory allocator which calls directly allocates GPU memory using the
/// logical device.
///
/// # Safety
///
/// This struct retains a reference to the Vulkan logical device. It is the
/// responsibility of the application to ensure that this allocator is dropped
/// before the device is destroyed.
///
/// It's a non-concern in this application because the RenderDevice owns both
/// the logical device and the allocators.
pub struct PassthroughAllocator {
    logical_device: ash::Device,
}

impl PassthroughAllocator {
    pub fn new(logical_device: ash::Device) -> Self {
        Self { logical_device }
    }
}

impl GPUMemoryAllocator for PassthroughAllocator {
    unsafe fn allocate(
        &mut self,
        allocate_info: vk::MemoryAllocateInfo,
        _alignment: u64,
    ) -> Result<Allocation, VulkanError> {
        // Alignment is unused because per the Vulkan spec:
        // https://registry.khronos.org/vulkan/specs/1.3-extensions/man/html/vkAllocateMemory.html
        // The memory returned by the device will always meet the system
        // alignment requirements.
        let memory = self
            .logical_device
            .allocate_memory(&allocate_info, None)
            .map_err(VulkanError::UnableToAllocateDeviceMemory)?;
        Ok(Allocation::new(
            memory,                          // memory
            0,                               // offset_in_bytes
            allocate_info.allocation_size,   // size_in_bytes
            allocate_info.memory_type_index, // memory index
        ))
    }

    unsafe fn free(
        &mut self,
        allocation: &Allocation,
    ) -> Result<(), VulkanError> {
        self.logical_device
            .free_memory(allocation.device_memory(), None);
        Ok(())
    }
}
