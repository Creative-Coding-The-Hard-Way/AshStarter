use std::{marker::PhantomData, sync::Arc};

use ash::vk;

use crate::graphics::vulkan_api::{
    Allocation, RenderDevice, VulkanDebug, VulkanError,
};

/// A Vulkan Device buffer which is mapped to host-coherent memory.
pub struct HostCoherentBuffer<T> {
    buffer: vk::Buffer,
    allocation: Allocation,
    render_device: Arc<RenderDevice>,
    _phantom_data: PhantomData<T>,
}

impl<T> HostCoherentBuffer<T> {
    /// Get the raw Vulkan handle to the buffer.
    ///
    /// # Safety
    ///
    /// Unsafe because:
    ///   - Ownership is not transferred. The caller must ensure no references
    ///     to the buffer exist after this HostCoherentBuffer is dropped.
    pub unsafe fn raw(&self) -> &vk::Buffer {
        &self.buffer
    }

    /// Flush the host caches so data is visible on the device.
    ///
    /// Generally this method does not need to be called explicitly because
    /// the memory is allocated to be host coherent.
    pub fn flush(&self) -> Result<(), VulkanError> {
        unsafe { self.allocation.flush_mapped_memory(&self.render_device) }
    }
}

impl<T> HostCoherentBuffer<T>
where
    T: Copy,
{
    /// Create a new Device buffer that the host can read and write.
    ///
    /// len is the number of elements to be stored in the buffer.
    pub fn new(
        render_device: Arc<RenderDevice>,
        usage: vk::BufferUsageFlags,
        len: usize,
    ) -> Result<Self, VulkanError> {
        let size_in_bytes = len * std::mem::size_of::<T>();
        let create_info = vk::BufferCreateInfo {
            size: size_in_bytes as u64,
            usage,
            sharing_mode: vk::SharingMode::EXCLUSIVE,
            ..Default::default()
        };
        let buffer = unsafe { render_device.create_buffer(&create_info)? };
        let mut allocation = unsafe {
            let memory_requirements =
                render_device.get_buffer_memory_requirements(&buffer);
            render_device.allocate_memory(
                memory_requirements,
                vk::MemoryPropertyFlags::HOST_COHERENT
                    | vk::MemoryPropertyFlags::HOST_VISIBLE,
            )?
        };

        // safe because the memory is allocated with the HOST_VISIBLE bit
        unsafe { allocation.map(&render_device)? };

        // safe because the buffer and allocation are held together in this
        // object
        unsafe { render_device.bind_buffer_memory(&buffer, &allocation)? };

        Ok(Self {
            buffer,
            allocation,
            render_device,
            _phantom_data: PhantomData::default(),
        })
    }

    /// Access the underlying memory as if it were a slice of T.
    ///
    /// # Safety
    ///
    /// Unsafe because:
    ///   - If the buffer is newly created then the values of T inside will have
    ///     undefined values.
    ///   - Reading from the buffer before writing data is unsafe and undefined
    ///     behavior.
    ///   - The caller must synchronize reads/writes to the buffer externally.
    pub unsafe fn as_slice(&self) -> Result<&[T], VulkanError> {
        // safe because the allocation was created with the HOST_VISIBLE bit
        // and is mapped when the buffer is created
        unsafe { self.allocation.as_slice() }
    }

    /// Access the underlying memory as if it were a mut slice of T.
    ///
    /// # Safety
    ///
    /// Unsafe because:
    ///   - If the buffer is newly created then the values of T inside will have
    ///     undefined values.
    ///   - Reading from the buffer before writing data is unsafe and undefined
    ///     behavior.
    ///   - The caller must synchronize reads/writes to the buffer externally.
    pub unsafe fn as_slice_mut(&mut self) -> Result<&mut [T], VulkanError> {
        // safe because the allocation was created with the HOST_VISIBLE bit
        // and is mapped when the buffer is created
        unsafe { self.allocation.as_slice_mut() }
    }
}

impl<T> VulkanDebug for HostCoherentBuffer<T> {
    fn set_debug_name(&self, debug_name: impl Into<String>) {
        self.render_device.name_vulkan_object(
            debug_name,
            vk::ObjectType::BUFFER,
            self.buffer,
        );
    }
}

impl<T> Drop for HostCoherentBuffer<T> {
    /// # Safety
    ///
    /// The application must ensure no Vulkan Device operations reference this
    /// buffer when it is dropped.
    fn drop(&mut self) {
        unsafe {
            self.render_device.destroy_buffer(self.buffer);
            self.allocation.unmap(&self.render_device);
            self.render_device
                .free_memory(&self.allocation)
                .expect("Unable to free the buffer's memory");
        }
    }
}