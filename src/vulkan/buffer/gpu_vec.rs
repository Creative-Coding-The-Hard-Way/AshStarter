use ::{ash::vk, std::sync::Arc};

use crate::vulkan::{
    buffer::{Buffer, BufferError},
    errors::VulkanDebugError,
    MemoryAllocator, RenderDevice, VulkanDebug,
};

/// A resizable GPU Buffer which holds a contiguous slice of T's.
pub struct GpuVec<T: Copy> {
    /// The device buffer which holds the actual data.
    pub buffer: Buffer,

    /// The number of elements (not bytes) which this buffer is able to hold.
    capacity: u32,

    /// The number of elements (not bytes) which this buffer is currently
    /// holding.
    length: u32,

    /// Buffer usage flags - used when the underlying buffer needs to be
    /// reallocated.
    usage_flags: vk::BufferUsageFlags,

    _phantom_data: std::marker::PhantomData<T>,
}

impl<T: Copy> GpuVec<T> {
    pub fn new(
        vk_dev: Arc<RenderDevice>,
        vk_alloc: Arc<dyn MemoryAllocator>,
        buffer_usage_flags: vk::BufferUsageFlags,
        initial_capacity: u32,
    ) -> Result<Self, BufferError> {
        let mut buffer = Buffer::new(
            vk_dev,
            vk_alloc,
            buffer_usage_flags,
            vk::MemoryPropertyFlags::HOST_VISIBLE
                | vk::MemoryPropertyFlags::HOST_COHERENT,
            Self::element_count_to_bytes(initial_capacity),
        )?;
        buffer.map()?;
        Ok(Self {
            buffer,
            capacity: initial_capacity,
            length: 0,
            usage_flags: buffer_usage_flags,
            _phantom_data: std::marker::PhantomData::default(),
        })
    }

    /// Append a value to the data in the buffer. Grows the buffer
    /// automatically if more memory is needed.
    ///
    /// # Returns
    ///
    /// * true if the underlying buffer needed to be reallocated
    /// * false if no change was required for the underlying buffer
    pub fn push_back(&mut self, value: T) -> Result<bool, BufferError> {
        let mut replaced = false;
        if self.length == self.capacity {
            self.grow(self.length * 2)?;
            replaced = true;
        }
        let data = self.buffer.data_mut()?;
        data[self.len()] = value;
        self.length = self.length + 1;
        Ok(replaced)
    }

    /// Reset the length without any change to the underlying buffer.
    pub fn clear(&mut self) {
        self.length = 0;
    }

    /// The number of elements in the buffer.
    pub fn len(&self) -> usize {
        self.length as usize
    }

    /// The number of bytes in use by the buffer.
    /// e.g. `self.len() * size_of<T>()`
    pub fn len_bytes(&self) -> u64 {
        Self::element_count_to_bytes(self.length)
    }
}

impl<T: Copy> VulkanDebug for GpuVec<T> {
    fn set_debug_name(
        &self,
        debug_name: impl Into<String>,
    ) -> Result<(), VulkanDebugError> {
        self.buffer.set_debug_name(debug_name)
    }
}

impl<T: Copy> GpuVec<T> {
    /// Return the number of bytes required to hold a given number of elements.
    fn element_count_to_bytes(count: u32) -> u64 {
        count as u64 * std::mem::size_of::<T>() as u64
    }

    /// Grow the buffer by allocating a new buffer, copying the old data into
    /// the new buffer, and releasing the old.
    fn grow(&mut self, desired_capacity: u32) -> Result<(), BufferError> {
        let mut buffer = Buffer::new(
            self.buffer.vk_dev.clone(),
            self.buffer.vk_alloc.clone(),
            self.usage_flags,
            vk::MemoryPropertyFlags::HOST_VISIBLE
                | vk::MemoryPropertyFlags::HOST_COHERENT,
            Self::element_count_to_bytes(desired_capacity),
        )?;
        buffer.map()?;
        self.capacity = desired_capacity;

        // copy the contents of the existing buffer to the new buffer
        {
            let new_data = buffer.data_mut::<T>()?;
            let old_data = self.buffer.data::<T>()?;
            new_data[..old_data.len()].copy_from_slice(old_data);
        }

        // replace the internal buffer with the new one
        std::mem::swap(&mut self.buffer, &mut buffer);
        Ok(())
    }
}
