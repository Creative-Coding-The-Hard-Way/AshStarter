use std::sync::Arc;

use ash::vk;

use crate::graphics::vulkan_api::{RenderDevice, VulkanDebug, VulkanError};

pub struct CommandPool {
    command_pool: vk::CommandPool,
    render_device: Arc<RenderDevice>,
}

impl CommandPool {
    pub fn new(
        render_device: Arc<RenderDevice>,
        queue_family_index: u32,
        flags: vk::CommandPoolCreateFlags,
    ) -> Result<Self, VulkanError> {
        let create_info = vk::CommandPoolCreateInfo {
            queue_family_index,
            flags,
            ..Default::default()
        };
        let command_pool =
            unsafe { render_device.create_command_pool(&create_info)? };
        Ok(Self {
            command_pool,
            render_device,
        })
    }

    /// # Safety
    ///
    /// Unsafe because the caller must free the command buffer before this pool
    /// is dropped.
    pub unsafe fn allocate_command_buffer(
        &self,
        level: vk::CommandBufferLevel,
    ) -> Result<vk::CommandBuffer, VulkanError> {
        let allocate_info = vk::CommandBufferAllocateInfo {
            command_pool: self.command_pool,
            level,
            command_buffer_count: 1,
            ..Default::default()
        };
        let buffers = self
            .render_device
            .allocate_command_buffers(&allocate_info)?;
        Ok(buffers[0])
    }

    /// # Safety
    ///
    /// Unsafe because the caller must ensure the command buffer is not in use
    /// by the GPU when it is freed.
    pub unsafe fn free_command_buffer(
        &self,
        command_buffer: vk::CommandBuffer,
    ) {
        self.render_device
            .free_command_buffers(&self.command_pool, &[command_buffer])
    }

    /// # Safety
    ///
    /// Unsafe because the application must ensure none of the command
    /// buffers allocated from this pool are in-use when the pool is
    /// reset.
    pub unsafe fn reset(&self) -> Result<(), VulkanError> {
        self.render_device.reset_command_pool(
            &self.command_pool,
            vk::CommandPoolResetFlags::empty(),
        )
    }
}

impl VulkanDebug for CommandPool {
    fn set_debug_name(&self, debug_name: impl Into<String>) {
        self.render_device.name_vulkan_object(
            debug_name,
            vk::ObjectType::COMMAND_POOL,
            self.command_pool,
        )
    }
}

impl Drop for CommandPool {
    /// # Safety
    ///
    /// The application must ensure that no GPU operations still refer to this
    /// command pool when it's destroyed.
    fn drop(&mut self) {
        unsafe { self.render_device.destroy_command_pool(self.command_pool) }
    }
}
