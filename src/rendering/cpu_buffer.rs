use super::Device;

use anyhow::{Context, Result};
use ash::{
    version::{DeviceV1_0, InstanceV1_0},
    vk,
};
use std::sync::Arc;

/// A CPU-accessible buffer.
///
/// Data is allocated directly, so every instance of this buffer contributes
/// to the driver-specified limit on the number of allocations supported by
/// the device.
pub struct CpuBuffer {
    /// the number of bytes allocated on the gpu for vertex_buffer_memory
    bytes_allocated: u64,

    vertex_buffer: vk::Buffer,
    vertex_buffer_memory: vk::DeviceMemory,

    device: Arc<Device>,
}

impl CpuBuffer {
    /// Create an empty vertex buffer.
    pub fn new(device: Arc<Device>) -> Result<Self> {
        let vertex_buffer = vk::Buffer::null();
        let vertex_buffer_memory = vk::DeviceMemory::null();
        let bytes_allocated = 0;
        Ok(Self {
            bytes_allocated,
            vertex_buffer,
            vertex_buffer_memory,
            device,
        })
    }

    /// The raw vertex buffer handle
    ///
    /// Unsafe because the buffer handle can become invalid when `write_data`
    /// is called. The application should not keep any long lived references to
    /// this handle.
    pub unsafe fn raw_buffer(&self) -> vk::Buffer {
        self.vertex_buffer
    }

    /// Write the provided data into the vertex buffer.
    ///
    /// Unsafe because this method can replace both the buffer and the backing
    /// memory. It is the responsibility of the application to ensure that
    /// neither resource is being used when this method is called.
    pub unsafe fn write_data<T>(&mut self, data: &[T]) -> Result<()>
    where
        T: Sized,
    {
        let byte_size = (std::mem::size_of::<T>() * data.len()) as u64;
        if byte_size > self.bytes_allocated {
            self.free_memory();
            self.allocate_memory(byte_size)?;
        }

        let ptr = self.device.logical_device.map_memory(
            self.vertex_buffer_memory,
            0,
            byte_size,
            vk::MemoryMapFlags::empty(),
        )?;

        std::ptr::copy_nonoverlapping(data.as_ptr(), ptr as *mut T, data.len());

        self.device
            .logical_device
            .unmap_memory(self.vertex_buffer_memory);

        Ok(())
    }

    /// Allocate buffer memory for the current size
    ///
    /// Unsafe because the vertex buffer is rebound automatically. The caller
    /// must ensure that the existing (if any) buffer and memory are freed.
    unsafe fn allocate_memory(&mut self, size: u64) -> Result<()> {
        let create_info = vk::BufferCreateInfo::builder()
            .size(size)
            .usage(vk::BufferUsageFlags::VERTEX_BUFFER)
            .sharing_mode(vk::SharingMode::EXCLUSIVE);

        self.vertex_buffer = self
            .device
            .logical_device
            .create_buffer(&create_info, None)?;

        let buffer_memory_requirements = self
            .device
            .logical_device
            .get_buffer_memory_requirements(self.vertex_buffer);

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
                let properties_supported = memory_type.property_flags.contains(
                    vk::MemoryPropertyFlags::HOST_VISIBLE
                        | vk::MemoryPropertyFlags::HOST_COHERENT,
                );
                type_supported & properties_supported
            })
            .map(|(i, _memory_type)| i as u32)
            .with_context(|| {
                "unable to find a suitable memory type for the vertex buffer!"
            })?;

        let allocate_info = vk::MemoryAllocateInfo::builder()
            .allocation_size(buffer_memory_requirements.size)
            .memory_type_index(memory_type_index);

        self.vertex_buffer_memory = self
            .device
            .logical_device
            .allocate_memory(&allocate_info, None)?;

        self.device.logical_device.bind_buffer_memory(
            self.vertex_buffer,
            self.vertex_buffer_memory,
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
        if self.vertex_buffer != vk::Buffer::null() {
            self.device
                .logical_device
                .destroy_buffer(self.vertex_buffer, None);
            self.vertex_buffer = vk::Buffer::null();
        }

        if self.vertex_buffer_memory != vk::DeviceMemory::null() {
            self.device
                .logical_device
                .free_memory(self.vertex_buffer_memory, None);
            self.vertex_buffer_memory = vk::DeviceMemory::null();
        }
    }
}

impl Drop for CpuBuffer {
    /// No checking is done to verify that the memory is done being used.
    ///
    /// It's the responsibility of the application to ensure all usage of the
    /// buffer has completed prior to dropping.
    fn drop(&mut self) {
        unsafe {
            self.free_memory();
        }
    }
}
