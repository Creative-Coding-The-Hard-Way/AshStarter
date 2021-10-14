use super::{CommandBufferError, CommandPool};

use crate::vulkan::{
    errors::VulkanDebugError, GpuQueue, RenderDevice, VulkanDebug,
};

use ::{
    ash::{version::DeviceV1_0, vk},
    std::sync::Arc,
};

impl CommandPool {
    /// Create a new command pool.
    pub fn new(
        vk_dev: Arc<RenderDevice>,
        queue: &GpuQueue,
        flags: vk::CommandPoolCreateFlags,
    ) -> Result<Self, CommandBufferError> {
        let raw = {
            let create_info = vk::CommandPoolCreateInfo {
                queue_family_index: queue.family_id,
                flags,
                ..Default::default()
            };
            unsafe {
                vk_dev
                    .logical_device
                    .create_command_pool(&create_info, None)
                    .map_err(CommandBufferError::UnableToCreateCommandPool)?
            }
        };
        Ok(Self { raw, vk_dev })
    }

    /// Create a new transient command pool for submitting graphics commands.
    pub fn new_transient_graphics_pool(
        vk_dev: Arc<RenderDevice>,
    ) -> Result<Self, CommandBufferError> {
        Self::new(
            vk_dev.clone(),
            &vk_dev.graphics_queue,
            vk::CommandPoolCreateFlags::TRANSIENT,
        )
    }

    /// Allocate raw vulkan command buffers.
    ///
    /// # Unsafe
    ///
    /// Because the caller is responsible for freeing the buffer when it's
    /// no-longer in use. Consider creating an owned [CommandBuffer] instance
    /// instead.
    pub unsafe fn allocate_command_buffers(
        &self,
        level: vk::CommandBufferLevel,
        command_buffer_count: u32,
    ) -> Result<Vec<vk::CommandBuffer>, CommandBufferError> {
        let create_info = vk::CommandBufferAllocateInfo {
            command_pool: self.raw,
            level,
            command_buffer_count,
            ..Default::default()
        };
        let buffer = self
            .vk_dev
            .logical_device
            .allocate_command_buffers(&create_info)
            .map_err(CommandBufferError::UnableToAllocateBuffer)?;
        Ok(buffer)
    }

    /// Allocate raw vulkan command buffers.
    ///
    /// # Unsafe
    ///
    /// Because the caller is responsible for freeing the buffer when it's
    /// no-longer in use. Consider creating an owned [CommandBuffer] instance
    /// instead.
    pub unsafe fn allocate_command_buffer(
        &self,
        level: vk::CommandBufferLevel,
    ) -> Result<vk::CommandBuffer, CommandBufferError> {
        let buffers = self.allocate_command_buffers(level, 1)?;
        Ok(buffers[0])
    }

    /// Free a Vulkan command buffer which was allocated from this pool.
    ///
    /// # Unsafe
    ///
    /// Because the caller must ensure that the provided buffer was actually
    /// allocated from this pool. Additionally, there is no internal
    /// synchronization, so it is invalid to use this method from multiple
    /// threads or while the command buffer is in use by the GPU.
    pub unsafe fn free_command_buffers(
        &self,
        command_buffers: &[vk::CommandBuffer],
    ) {
        self.vk_dev
            .logical_device
            .free_command_buffers(self.raw, command_buffers);
    }

    /// Free a Vulkan command buffer which was allocated from this pool.
    ///
    /// # Unsafe
    ///
    /// Because the caller must ensure that the provided buffer was actually
    /// allocated from this pool. Additionally, there is no internal
    /// synchronization, so it is invalid to use this method from multiple
    /// threads or while the command buffer is in use by the GPU.
    pub unsafe fn free_command_buffer(
        &self,
        command_buffer: vk::CommandBuffer,
    ) {
        self.free_command_buffers(&[command_buffer]);
    }

    /// Reset the entire command pool.
    pub fn reset(&self) -> Result<(), CommandBufferError> {
        unsafe {
            self.vk_dev
                .logical_device
                .reset_command_pool(
                    self.raw,
                    vk::CommandPoolResetFlags::empty(),
                )
                .map_err(CommandBufferError::UnableToResetPool)?;
        }
        Ok(())
    }
}

impl VulkanDebug for CommandPool {
    fn set_debug_name(
        &self,
        debug_name: impl Into<String>,
    ) -> Result<(), VulkanDebugError> {
        self.vk_dev.name_vulkan_object(
            debug_name,
            vk::ObjectType::COMMAND_POOL,
            self.raw,
        )?;
        Ok(())
    }
}

impl Drop for CommandPool {
    /// # DANGER
    ///
    /// There is no internal synchronization for this type. Unexpected behavior
    /// can occur if this instance is still in-use by the GPU when it is
    /// dropped.
    fn drop(&mut self) {
        unsafe {
            self.vk_dev
                .logical_device
                .destroy_command_pool(self.raw, None)
        }
    }
}
