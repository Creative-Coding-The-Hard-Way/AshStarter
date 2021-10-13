use super::{Buffer, BufferError, GpuVec};

use crate::vulkan::{BufferAllocator, RenderDevice};

use ::ash::vk;

impl<T: Copy> GpuVec<T> {
    //
    pub fn new(
        vk_dev: &RenderDevice,
        vk_alloc: &mut impl BufferAllocator,
        buffer_usage_flags: vk::BufferUsageFlags,
        initial_capacity: u64,
    ) -> Result<Self, BufferError> {
        let mut buffer = vk_alloc.create_buffer(
            vk_dev,
            buffer_usage_flags,
            vk::MemoryPropertyFlags::HOST_VISIBLE
                | vk::MemoryPropertyFlags::HOST_COHERENT,
            Self::element_count_to_bytes(initial_capacity),
        )?;
        buffer.map(vk_dev)?;
        Ok(Self {
            buffer,
            usage_flags: buffer_usage_flags,
            _phantom_data: std::marker::PhantomData::default(),
        })
    }

    /// Destroy the buffer
    pub unsafe fn destroy(
        &mut self,
        vk_dev: &RenderDevice,
        vk_alloc: &mut impl BufferAllocator,
    ) -> Result<(), BufferError> {
        vk_alloc.destroy_buffer(vk_dev, &mut self.buffer)?;
        Ok(())
    }
}

impl<T: Copy> GpuVec<T> {
    /// Return the number of bytes required to hold a given number of elements.
    fn element_count_to_bytes(count: u64) -> u64 {
        count * std::mem::size_of::<T>() as u64
    }
}
