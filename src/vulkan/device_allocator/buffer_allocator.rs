use crate::vulkan::{Allocation, Buffer, RenderDevice};

use super::{DeviceAllocator, DeviceAllocatorError};

use ash::{version::DeviceV1_0, vk};

impl dyn DeviceAllocator {
    /// Create a new Vulkan device buffer.
    pub fn create_buffer(
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
    pub unsafe fn destroy_buffer(
        &mut self,
        vk_dev: &RenderDevice,
        cpu_buffer: &mut Buffer,
    ) -> Result<(), DeviceAllocatorError> {
        if cpu_buffer.raw != vk::Buffer::null() {
            vk_dev.logical_device.destroy_buffer(cpu_buffer.raw, None);
            cpu_buffer.raw = vk::Buffer::null();
        }
        if cpu_buffer.allocation != Allocation::null() {
            self.free(vk_dev, &cpu_buffer.allocation)?;
            cpu_buffer.allocation = Allocation::null();
        }
        Ok(())
    }
}
