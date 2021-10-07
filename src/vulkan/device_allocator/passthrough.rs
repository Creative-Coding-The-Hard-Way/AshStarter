use crate::vulkan::RenderDevice;

use super::{Allocation, DeviceAllocator, DeviceAllocatorError};

use ash::{version::DeviceV1_0, vk};

/// An allocator implementation which directly allocates and frees memory using
/// the vulkan device.
#[derive(Clone)]
pub struct PassthroughAllocator {}

impl PassthroughAllocator {
    pub fn new() -> Self {
        Self {}
    }
}

impl DeviceAllocator for PassthroughAllocator {
    /// Directly allocate device memory onto the heap indicated by the
    /// memory type index of the `allocate_info` struct.
    unsafe fn allocate(
        &mut self,
        vk_dev: &RenderDevice,
        allocate_info: vk::MemoryAllocateInfo,
        _alignment: u64,
    ) -> Result<Allocation, DeviceAllocatorError> {
        // No special attention is required for handling alignment because
        // memory allocated by the logical device will always be aligned to the
        // LCM of the memory alignment requirements for this system. See the
        // notes here:
        // https://www.khronos.org/registry/vulkan/specs/1.2-extensions/man/html/vkAllocateMemory.html
        Ok(Allocation {
            memory: vk_dev
                .logical_device
                .allocate_memory(&allocate_info, None)
                .map_err(DeviceAllocatorError::LogicalDeviceAllocationFailed)?,
            offset: 0,
            byte_size: allocate_info.allocation_size,
            memory_type_index: allocate_info.memory_type_index,
        })
    }

    /// Free the allocation's underlying memory.
    ///
    /// # unsafe because
    ///
    /// - it is the responsibility of the caller to ensure the allocation's
    ///   device memory is no longer in use
    /// - this *includes* other allocations which reference the same piece of
    ///   memory! Don't double-free!
    ///
    unsafe fn free(
        &mut self,
        vk_dev: &RenderDevice,
        allocation: &Allocation,
    ) -> Result<(), DeviceAllocatorError> {
        vk_dev.logical_device.free_memory(allocation.memory, None);
        Ok(())
    }
}
