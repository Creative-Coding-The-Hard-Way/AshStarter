use super::PerFrame;

use crate::vulkan::{
    errors::{VulkanDebugError, VulkanError},
    sync::{Fence, Semaphore},
    CommandBuffer, CommandPool, RenderDevice, VulkanDebug,
};

use ::{anyhow::Result, std::sync::Arc};

impl PerFrame {
    /// Create new per-frame resources.
    pub fn new(vk_dev: Arc<RenderDevice>) -> Result<Self, VulkanError> {
        let acquire_semaphore = None;
        let release_semaphore = Semaphore::new(vk_dev.clone())?;
        let queue_submit_fence = Fence::new(vk_dev.clone())?;

        let command_pool =
            Arc::new(CommandPool::new_transient_graphics_pool(vk_dev.clone())?);
        let command_buffer = CommandBuffer::new_primary(command_pool.clone())?;

        Ok(Self {
            acquire_semaphore,
            release_semaphore,
            queue_submit_fence,
            command_pool,
            command_buffer,
        })
    }
}

impl VulkanDebug for PerFrame {
    fn set_debug_name(
        &self,
        debug_name: impl Into<String>,
    ) -> Result<(), VulkanDebugError> {
        let name = debug_name.into();
        self.release_semaphore
            .set_debug_name(format!("{} - Release Semaphore", name))?;
        self.queue_submit_fence
            .set_debug_name(format!("{} - Queue Submission Fence", name))?;
        self.command_pool
            .set_debug_name(format!("{} - Command Pool", name))?;
        self.command_buffer
            .set_debug_name(format!("{} - Command Buffer", name))
    }
}
