use {
    crate::graphics::{vulkan_api::RenderDevice, GraphicsError},
    ash::vk,
    std::sync::Arc,
};

/// RAII Vulkan CommandPool.
pub struct CommandPool {
    command_pool: vk::CommandPool,
    primary_command_buffers: Vec<vk::CommandBuffer>,
    secondary_command_buffers: Vec<vk::CommandBuffer>,
    render_device: Arc<RenderDevice>,
}

impl CommandPool {
    /// Create a new Vulkan command pool.
    ///
    /// # Safety
    ///
    /// Unsafe because:
    ///   - command pools must be destroyed before the Vulkan device is dropped.
    pub unsafe fn new(
        render_device: Arc<RenderDevice>,
        create_info: &vk::CommandPoolCreateInfo,
    ) -> Result<Self, GraphicsError> {
        let command_pool = unsafe {
            render_device
                .device()
                .create_command_pool(create_info, None)?
        };
        Ok(Self {
            command_pool,
            primary_command_buffers: vec![],
            secondary_command_buffers: vec![],
            render_device,
        })
    }

    /// Set the name which shows up in Vulkan debug logs for this resource.
    pub fn set_debug_name(&self, name: impl Into<String>) {
        self.render_device.set_debug_name(
            self.command_pool,
            vk::ObjectType::COMMAND_POOL,
            name,
        );
    }

    /// Get the n'th primary command buffer allocated by this pool.
    ///
    /// Note: The command pool destroys all allocated buffers when it is
    /// dropped. The caller must ensure that no command buffers are kept around
    /// after the pool is dropped.
    pub fn primary_command_buffer(&self, index: usize) -> vk::CommandBuffer {
        self.primary_command_buffers[index]
    }

    /// Get the n'th secondary command buffer allocated by this pool.
    ///
    /// Note: The command pool destroys all allocated buffers when it is
    /// dropped. The caller must ensure that no command buffers are kept around
    /// after the pool is dropped.
    pub fn secondary_command_buffer(&self, index: usize) -> vk::CommandBuffer {
        self.secondary_command_buffers[index]
    }

    /// Allocate primary command buffers from this pool.
    ///
    /// # Returns
    ///
    /// Returns the index of the first newly allocated command buffer.
    pub fn allocate_primary_command_buffers(
        &mut self,
        count: u32,
    ) -> Result<usize, GraphicsError> {
        let create_info = vk::CommandBufferAllocateInfo {
            command_pool: self.command_pool,
            level: vk::CommandBufferLevel::PRIMARY,
            command_buffer_count: count,
            ..Default::default()
        };
        let new_buffers = unsafe {
            self.render_device
                .device()
                .allocate_command_buffers(&create_info)?
        };
        let last = self.primary_command_buffers.len();
        self.primary_command_buffers.extend(new_buffers.into_iter());
        Ok(last)
    }

    /// Allocate primary command buffers from this pool.
    ///
    /// # Returns
    ///
    /// Returns the index of the first newly allocated command buffer.
    pub fn allocate_secondary_command_buffers(
        &mut self,
        count: u32,
    ) -> Result<usize, GraphicsError> {
        let create_info = vk::CommandBufferAllocateInfo {
            command_pool: self.command_pool,
            level: vk::CommandBufferLevel::SECONDARY,
            command_buffer_count: count,
            ..Default::default()
        };
        let new_buffers = unsafe {
            self.render_device
                .device()
                .allocate_command_buffers(&create_info)?
        };
        let last = self.secondary_command_buffers.len();
        self.secondary_command_buffers
            .extend(new_buffers.into_iter());
        Ok(last)
    }

    /// Get the raw Vulkan command pool handle.
    pub fn raw(&self) -> vk::CommandPool {
        self.command_pool
    }
}

impl Drop for CommandPool {
    fn drop(&mut self) {
        unsafe {
            self.render_device
                .device()
                .destroy_command_pool(self.command_pool, None);
        }
    }
}

impl std::fmt::Debug for CommandPool {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("CommandPool")
            .field("command_pool", &self.command_pool)
            .field("primary_command_buffers", &self.primary_command_buffers)
            .field("secondary_command_buffers", &self.secondary_command_buffers)
            .finish()
    }
}
