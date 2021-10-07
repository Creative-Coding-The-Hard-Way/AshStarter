use super::{Allocation, DeviceAllocator, DeviceAllocatorError};

use crate::vulkan::RenderDevice;

use ash::vk;

impl dyn DeviceAllocator {
    /// Allocate memory given a set of requirements and desired properties.
    pub unsafe fn allocate_memory(
        &mut self,
        vk_dev: &RenderDevice,
        memory_requirements: vk::MemoryRequirements,
        property_flags: vk::MemoryPropertyFlags,
    ) -> Result<Allocation, DeviceAllocatorError> {
        use ash::version::InstanceV1_0;

        let memory_properties = vk_dev
            .instance
            .ash
            .get_physical_device_memory_properties(vk_dev.physical_device);
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
                DeviceAllocatorError::MemoryTypeNotFound(
                    property_flags,
                    memory_requirements,
                )
            })?;
        let allocate_info = vk::MemoryAllocateInfo {
            memory_type_index,
            allocation_size: memory_requirements.size,
            ..Default::default()
        };

        self.allocate(vk_dev, allocate_info, memory_requirements.alignment)
    }
}
