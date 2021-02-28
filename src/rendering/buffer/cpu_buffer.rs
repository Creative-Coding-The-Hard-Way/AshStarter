use super::GpuBuffer;
use crate::rendering::Device;

use anyhow::Result;
use ash::{version::DeviceV1_0, vk};
use std::sync::Arc;

/// A CPU-accessible buffer.
///
/// Data is allocated directly, so every instance of this buffer contributes
/// to the driver-specified limit on the number of allocations supported by
/// the device.
pub struct CpuBuffer {
    buffer: GpuBuffer,
    device: Arc<Device>,
}

impl CpuBuffer {
    /// Create an empty buffer which can be written from the CPU.
    pub fn new(
        device: Arc<Device>,
        usage: vk::BufferUsageFlags,
    ) -> Result<Self> {
        Ok(Self {
            buffer: GpuBuffer::create(
                device.clone(),
                usage,
                vk::MemoryPropertyFlags::HOST_VISIBLE
                    | vk::MemoryPropertyFlags::HOST_COHERENT,
            )?,
            device,
        })
    }

    /// The raw vertex buffer handle
    ///
    /// Unsafe because the buffer handle can become invalid when `write_data`
    /// is called. The application should not keep any long lived references to
    /// this handle.
    pub unsafe fn raw_buffer(&self) -> vk::Buffer {
        self.buffer.raw
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
        if byte_size > self.buffer.bytes_allocated {
            self.buffer.allocate_memory(byte_size)?;
        }

        let ptr = self.device.logical_device.map_memory(
            self.buffer.memory,
            0,
            byte_size,
            vk::MemoryMapFlags::empty(),
        )?;

        std::ptr::copy_nonoverlapping(data.as_ptr(), ptr as *mut T, data.len());

        self.device.logical_device.unmap_memory(self.buffer.memory);

        Ok(())
    }
}
