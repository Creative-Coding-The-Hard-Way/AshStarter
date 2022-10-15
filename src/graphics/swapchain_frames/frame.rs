use {
    crate::graphics::vulkan_api::{
        CommandBuffer, CommandPool, Fence, ImageView, RenderDevice, Semaphore,
        SemaphorePool, Swapchain, VulkanDebug, VulkanError,
    },
    ash::vk,
    std::sync::Arc,
};

/// All of the per-frame resources required to synchronize graphics command
/// and presentation.
pub struct Frame {
    // The swapchain image index this frame corresponds to.
    swapchain_image_index: usize,

    // Each frame has its own command buffer.
    command_buffer: CommandBuffer,

    // The command pool used to allocate the frame's command buffer.
    command_pool: Arc<CommandPool>,

    // Signalled when the swapchain image is ready for rendering
    acquire_semaphore: Option<Semaphore>,

    // Signalled when all graphics operations are complete and the frame is
    // ready to present.
    release_semaphore: Semaphore,

    // Signalled when all submitted graphics commands have completed for this
    // frame.
    queue_submit_fence: Fence,

    // The ImageView for the corresponding Swapchain image.
    swapchain_image_view: Arc<ImageView>,
}

impl Frame {
    /// Get the index for the swapchain image that this frame corresponds to.
    /// This is always in the range `0..swapchain_image_count`.
    pub fn swapchain_image_index(&self) -> usize {
        self.swapchain_image_index
    }

    /// Get the swapchain image view associated with this frame.
    pub fn swapchain_image_view(&self) -> &Arc<ImageView> {
        &self.swapchain_image_view
    }

    /// Get the frame's command buffer.
    ///
    /// The command buffer is ready for commands when the Frame is returned
    /// by acquire_swapchain_frame. Commands are submitted when present_frame
    /// is called.
    pub fn command_buffer(&mut self) -> &mut CommandBuffer {
        &mut self.command_buffer
    }
}

impl Frame {
    pub(super) fn new(
        render_device: &Arc<RenderDevice>,
        semaphore_pool: &mut SemaphorePool,
        swapchain_image_index: usize,
        swapchain: Arc<Swapchain>,
    ) -> Result<Self, VulkanError> {
        let acquire_semaphore = None;
        let release_semaphore = semaphore_pool.get_semaphore()?;
        let queue_submit_fence = Fence::new(render_device.clone())?;
        let command_pool = Arc::new(CommandPool::new(
            render_device.clone(),
            render_device.graphics_queue_family_index(),
            vk::CommandPoolCreateFlags::empty(),
        )?);
        let command_buffer = CommandBuffer::new(
            render_device.clone(),
            command_pool.clone(),
            vk::CommandBufferLevel::PRIMARY,
        )?;
        let swapchain_image_view = Arc::new(ImageView::for_swapchain_image(
            render_device.clone(),
            swapchain,
            swapchain_image_index,
        )?);
        Ok(Self {
            swapchain_image_index,
            command_buffer,
            command_pool,
            acquire_semaphore,
            release_semaphore,
            queue_submit_fence,
            swapchain_image_view,
        })
    }

    /// Wait for all pending graphics commands for this frame to complete.
    pub(super) fn wait_for_graphics_commands_to_complete(
        &mut self,
    ) -> Result<(), VulkanError> {
        self.queue_submit_fence.wait()
    }

    /// Replace the Frame's acquire semaphore with the given semaphore. The
    /// old value is returned.
    pub(super) fn replace_acquire_semaphore(
        &mut self,
        acquire_semaphore: Semaphore,
    ) -> Option<Semaphore> {
        acquire_semaphore.set_debug_name(format!(
            "Frame {} acquire semaphore",
            self.swapchain_image_index
        ));
        self.acquire_semaphore.replace(acquire_semaphore)
    }

    /// Reset the frame command buffer and prepare for the current frame's
    /// commands.
    pub(super) fn reset_frame_commands(&mut self) -> Result<(), VulkanError> {
        self.wait_for_graphics_commands_to_complete()?;
        self.queue_submit_fence.reset()?;

        // Safe because the queue submission fence is waited before resetting.
        unsafe {
            self.command_pool.reset()?;
        }
        self.command_buffer.begin_one_time_submit()
    }

    /// End the command buffer and submit to the graphics queue.
    pub(super) fn submit_frame_commands(
        &mut self,
        graphics_complete_signal_semaphores: &[&Semaphore],
    ) -> Result<(), VulkanError> {
        let sempahores = [
            graphics_complete_signal_semaphores,
            &[&self.release_semaphore],
        ]
        .concat();
        self.command_buffer.end_command_buffer()?;
        unsafe {
            self.command_buffer.submit_graphics_commands(
                &[self.acquire_semaphore.as_ref().unwrap()],
                &[vk::PipelineStageFlags::COLOR_ATTACHMENT_OUTPUT],
                &sempahores,
                Some(&self.queue_submit_fence),
            )
        }
    }

    /// Borrow the release semaphore which is signalled by the completion of
    /// this frame's graphics commands.
    pub(super) fn release_semaphore(&self) -> &Semaphore {
        &self.release_semaphore
    }
}

impl VulkanDebug for Frame {
    fn set_debug_name(&self, debug_name: impl Into<String>) {
        let name = debug_name.into();
        self.command_buffer
            .set_debug_name(format!("{} command buffer", name));
        self.command_pool
            .set_debug_name(format!("{} command pool", name));
        self.release_semaphore
            .set_debug_name(format!("{} release semaphore", name,));
        self.queue_submit_fence
            .set_debug_name(format!("{} queue submit fence", name,));
        self.swapchain_image_view
            .set_debug_name(format!("{} swapchain image view", name,));
    }
}

impl Drop for Frame {
    fn drop(&mut self) {
        self.wait_for_graphics_commands_to_complete()
            .unwrap_or_else(|_| {
                panic!(
                    "Error while waiting for commands to complete for frame {}",
                    self.swapchain_image_index()
                )
            });
    }
}
