use {
    crate::graphics::{vulkan_api::RenderDevice, GraphicsError},
    ash::vk,
    ccthw_ash_allocator::Allocation,
    std::sync::Arc,
};

/// RAII Vulkan Buffer.
pub struct Buffer {
    buffer: vk::Buffer,
    allocation: Allocation,
    render_device: Arc<RenderDevice>,
}

impl Buffer {
    /// Create a new Vulkan descriptor pool.
    ///
    /// # Safety
    ///
    /// Unsafe because:
    ///   - command pools must be destroyed before the Vulkan device is dropped.
    pub unsafe fn new(
        render_device: Arc<RenderDevice>,
        create_info: &vk::BufferCreateInfo,
        memory_property_flags: vk::MemoryPropertyFlags,
    ) -> Result<Self, GraphicsError> {
        let (buffer, allocation) = unsafe {
            render_device
                .memory()
                .allocate_buffer(create_info, memory_property_flags)?
        };
        Ok(Self {
            buffer,
            allocation,
            render_device,
        })
    }

    /// Set the name which shows up in Vulkan debug logs for this resource.
    pub fn set_debug_name(&self, name: impl Into<String>) {
        self.render_device.set_debug_name(
            self.buffer,
            vk::ObjectType::BUFFER,
            name,
        );
    }

    /// Get the backing memory allocation for the Buffer.
    pub fn allocation(&self) -> &Allocation {
        &self.allocation
    }

    /// Get the raw Vulkan command pool handle.
    pub fn raw(&self) -> vk::Buffer {
        self.buffer
    }
}

impl Drop for Buffer {
    fn drop(&mut self) {
        unsafe {
            self.render_device
                .memory()
                .free_buffer(self.buffer, self.allocation.clone());
        }
    }
}

impl std::fmt::Debug for Buffer {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Buffer")
            .field("buffer", &self.buffer)
            .field("allocation", &self.allocation)
            .finish()
    }
}
