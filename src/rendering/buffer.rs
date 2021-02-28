mod cpu_buffer;

pub use self::cpu_buffer::CpuBuffer;
use crate::rendering::Device;

use anyhow::{Context, Result};
use ash::{
    version::{DeviceV1_0, InstanceV1_0},
    vk,
};
use std::sync::Arc;

/// A gpu buffer which can be destroyed and reallocated on demand.
pub struct GpuBuffer {
    /// the raw vulkan buffer handle
    raw: vk::Buffer,

    /// the gpu memory owned by this buffer instance
    memory: vk::DeviceMemory,

    /// the number of bytes allocated on the gpu for vertex_buffer_memory
    bytes_allocated: u64,

    usage: vk::BufferUsageFlags,
    properties: vk::MemoryPropertyFlags,

    /// the device used to create this buffer
    device: Arc<Device>,
}

impl GpuBuffer {
    pub fn create(
        device: Arc<Device>,
        usage: vk::BufferUsageFlags,
        properties: vk::MemoryPropertyFlags,
    ) -> Result<Self> {
        Ok(Self {
            bytes_allocated: 0,
            raw: vk::Buffer::null(),
            memory: vk::DeviceMemory::null(),
            usage,
            properties,
            device,
        })
    }

    /// Allocate buffer memory for the current size
    ///
    /// Unsafe because the vertex buffer is rebound automatically. The caller
    /// must ensure that the existing (if any) buffer and memory are freed.
    pub unsafe fn allocate_memory(&mut self, size: u64) -> Result<()> {
        self.free_memory();

        let create_info = vk::BufferCreateInfo::builder()
            .size(size)
            .usage(self.usage)
            .sharing_mode(vk::SharingMode::EXCLUSIVE);

        self.raw = self
            .device
            .logical_device
            .create_buffer(&create_info, None)?;

        let buffer_memory_requirements = self
            .device
            .logical_device
            .get_buffer_memory_requirements(self.raw);

        let memory_properties = self
            .device
            .instance
            .ash
            .get_physical_device_memory_properties(self.device.physical_device);

        let memory_type_index = memory_properties
            .memory_types
            .iter()
            .enumerate()
            .find(|(i, memory_type)| {
                let type_supported =
                    buffer_memory_requirements.memory_type_bits & (1 << i) != 0;
                let properties_supported =
                    memory_type.property_flags.contains(self.properties);
                type_supported & properties_supported
            })
            .map(|(i, _memory_type)| i as u32)
            .with_context(|| {
                "unable to find a suitable memory type for this gpu buffer!"
            })?;

        let allocate_info = vk::MemoryAllocateInfo::builder()
            .allocation_size(buffer_memory_requirements.size)
            .memory_type_index(memory_type_index);

        self.memory = self
            .device
            .logical_device
            .allocate_memory(&allocate_info, None)?;

        self.device.logical_device.bind_buffer_memory(
            self.raw,
            self.memory,
            0,
        )?;
        self.bytes_allocated = buffer_memory_requirements.size;

        Ok(())
    }

    /// Free the GPU memory associated with this buffer.
    ///
    /// Unsafe because the caller must ensure that the memory is not in use
    /// when it is freed.
    unsafe fn free_memory(&mut self) {
        if self.raw != vk::Buffer::null() {
            self.device.logical_device.destroy_buffer(self.raw, None);
            self.raw = vk::Buffer::null();
        }

        if self.memory != vk::DeviceMemory::null() {
            self.device.logical_device.free_memory(self.memory, None);
            self.memory = vk::DeviceMemory::null();
        }
    }
}

impl Drop for GpuBuffer {
    /// Free the buffer and any memory which is allocated.
    ///
    /// It is the responsibility of the application to synchronize this drop
    /// with any ongoing GPU actions.
    fn drop(&mut self) {
        unsafe {
            self.free_memory();
        }
    }
}
