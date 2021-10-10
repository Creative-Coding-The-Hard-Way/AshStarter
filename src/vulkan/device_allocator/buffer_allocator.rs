use super::{
    Allocation, BufferAllocator, DeviceAllocator, DeviceAllocatorError,
};

use crate::vulkan::{Buffer, RenderDevice};

use ash::{version::DeviceV1_0, vk};

impl<T> BufferAllocator for T
where
    T: DeviceAllocator,
{
    /// Allocate memory given a set of requirements and desired properties.
    unsafe fn allocate_memory(
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

    /// Create a new Vulkan device buffer.
    fn create_buffer(
        &mut self,
        vk_dev: &RenderDevice,
        buffer_usage_flags: vk::BufferUsageFlags,
        memory_property_flags: vk::MemoryPropertyFlags,
        size_in_bytes: u64,
    ) -> Result<Buffer, DeviceAllocatorError> {
        let create_info = vk::BufferCreateInfo {
            size: size_in_bytes,
            usage: buffer_usage_flags,
            sharing_mode: vk::SharingMode::EXCLUSIVE,
            ..Default::default()
        };

        let buffer_handle = unsafe {
            vk_dev
                .logical_device
                .create_buffer(&create_info, None)
                .map_err(|err| DeviceAllocatorError::UnableToCreateBuffer {
                    size: size_in_bytes,
                    usage: buffer_usage_flags,
                    source: err,
                })?
        };
        let allocation = unsafe {
            let buffer_memory_requirements = vk_dev
                .logical_device
                .get_buffer_memory_requirements(buffer_handle);
            self.allocate_memory(
                vk_dev,
                buffer_memory_requirements,
                memory_property_flags,
            )?
        };

        unsafe {
            vk_dev
                .logical_device
                .bind_buffer_memory(
                    buffer_handle,
                    allocation.memory,
                    allocation.offset,
                )
                .map_err(DeviceAllocatorError::UnableToBindDeviceMemory)?;
        }

        Ok(Buffer {
            raw: buffer_handle,
            allocation,
        })
    }

    /// Destroy a Vulkan buffer.
    ///
    /// UNSAFE: because the application must synchronize access to the buffer.
    /// There must be no ongoing operations which reference this buffer when
    /// it is destroyed.
    unsafe fn destroy_buffer(
        &mut self,
        vk_dev: &RenderDevice,
        buffer: &mut Buffer,
    ) -> Result<(), DeviceAllocatorError> {
        if buffer.raw != vk::Buffer::null() {
            vk_dev.logical_device.destroy_buffer(buffer.raw, None);
            buffer.raw = vk::Buffer::null();
        }
        if buffer.allocation != Allocation::null() {
            self.free(vk_dev, &buffer.allocation)?;
            buffer.allocation = Allocation::null();
        }
        Ok(())
    }
}
