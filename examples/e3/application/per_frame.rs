use ::{
    anyhow::Result,
    ccthw::vulkan::{
        errors::VulkanDebugError,
        sync::{Fence, Semaphore},
        CommandBuffer, CommandPool, RenderDevice, VulkanDebug,
    },
    std::sync::Arc,
};

pub struct PerFrame {
    /// Signalled when the frame is ready to be used for rendering.
    pub acquire_semaphore: Option<Semaphore>,

    /// Signalled when all graphics operations are complete and the frame is
    /// ready for presentation.
    pub release_semaphore: Semaphore,

    /// Signalled when all submitted graphics commands have completed.
    pub queue_submit_fence: Fence,

    /// The command buffer for operations in this frame.
    pub command_buffer: CommandBuffer,

    /// The command pool for operations in this frame.
    pub command_pool: Arc<CommandPool>,

    /// The owning vulkan device for the resources in this structure.
    pub vk_dev: Arc<RenderDevice>,
}

impl PerFrame {
    /// Create new per-frame resources.
    pub fn new(vk_dev: Arc<RenderDevice>) -> Result<Self> {
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
            vk_dev,
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
