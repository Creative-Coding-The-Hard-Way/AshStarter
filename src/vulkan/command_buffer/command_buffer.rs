use ::{ash::vk, std::sync::Arc};

use super::{CommandBuffer, CommandBufferError, CommandPool};
use crate::vulkan::{errors::VulkanDebugError, VulkanDebug};

impl CommandBuffer {
    /// Allocate a new command buffer from the given pool.
    pub fn new(
        pool: Arc<CommandPool>,
        command_level: vk::CommandBufferLevel,
    ) -> Result<Self, CommandBufferError> {
        let raw = unsafe { pool.allocate_command_buffer(command_level)? };
        Ok(Self {
            raw,
            vk_dev: pool.vk_dev.clone(),
            pool,
        })
    }

    /// Allocate a new primary command buffer from the given pool.
    pub fn new_primary(
        pool: Arc<CommandPool>,
    ) -> Result<Self, CommandBufferError> {
        Self::new(pool, vk::CommandBufferLevel::PRIMARY)
    }
}

impl VulkanDebug for CommandBuffer {
    fn set_debug_name(
        &self,
        debug_name: impl Into<String>,
    ) -> Result<(), VulkanDebugError> {
        self.vk_dev.name_vulkan_object(
            debug_name,
            vk::ObjectType::COMMAND_BUFFER,
            self.raw,
        )?;
        Ok(())
    }
}

impl Drop for CommandBuffer {
    /// # DANGER
    ///
    /// There is no internal synchronization for this type. Unexpected behavior
    /// can occur if this instance is still in-use by the GPU when it is
    /// dropped.
    fn drop(&mut self) {
        unsafe {
            self.pool.free_command_buffer(self.raw);
        }
    }
}
