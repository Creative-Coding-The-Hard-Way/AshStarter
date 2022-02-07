use ::{
    ash::vk,
    std::sync::{Arc, Mutex},
};

use crate::vulkan::{
    device_allocator::{
        Allocation, AllocatorError, ComposableAllocator, MemoryAllocator,
    },
    RenderDevice,
};

/// A memory allocator implementation which decorates a composed allocator
/// with a mutex.
pub struct LockedMemoryAllocator<Alloc: ComposableAllocator> {
    composed_allocator: Mutex<Alloc>,
    vk_dev: Arc<RenderDevice>,
}

impl<Alloc: ComposableAllocator> LockedMemoryAllocator<Alloc> {
    pub fn new(vk_dev: Arc<RenderDevice>, allocater: Alloc) -> Self {
        Self {
            composed_allocator: Mutex::new(allocater),
            vk_dev,
        }
    }
}

impl<Alloc: ComposableAllocator> MemoryAllocator
    for LockedMemoryAllocator<Alloc>
{
    /// Lock the composed allocator's mutex and dispatch the memory request.
    unsafe fn allocate_memory(
        &self,
        memory_requirements: vk::MemoryRequirements,
        property_flags: vk::MemoryPropertyFlags,
    ) -> Result<Allocation, AllocatorError> {
        let memory_properties = self
            .vk_dev
            .instance
            .ash
            .get_physical_device_memory_properties(self.vk_dev.physical_device);
        let memory_type_index = memory_properties
            .memory_types
            .iter()
            .enumerate()
            .find(|(i, memory_type)| {
                let type_supported =
                    memory_requirements.memory_type_bits & (1 << i) != 0;
                let properties_supported =
                    memory_type.property_flags.contains(property_flags);
                type_supported & properties_supported
            })
            .map(|(i, _memory_type)| i as u32)
            .ok_or_else(|| {
                AllocatorError::MemoryTypeNotFound(
                    property_flags,
                    memory_requirements,
                )
            })?;
        let allocate_info = vk::MemoryAllocateInfo {
            memory_type_index,
            allocation_size: memory_requirements.size,
            ..Default::default()
        };

        let mut allocator = self
            .composed_allocator
            .lock()
            .expect("unable to acquire the composed memory allocator lock");
        allocator.allocate(allocate_info, memory_requirements.alignment)
    }

    /// Lock the composed allocator's mutex and free the memory.
    unsafe fn free(
        &self,
        allocation: &Allocation,
    ) -> Result<(), AllocatorError> {
        let mut allocator = self
            .composed_allocator
            .lock()
            .expect("unable to acquire the composed memory allocator lock");
        allocator.free(allocation)
    }
}
