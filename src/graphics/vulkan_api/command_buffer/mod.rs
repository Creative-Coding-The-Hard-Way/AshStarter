use {
    crate::graphics::{
        vulkan_api::{raii, Queue, RenderDevice},
        GraphicsError,
    },
    ash::vk,
    ccthw_ash_instance::VulkanHandle,
    std::sync::Arc,
};

/// A utility for managing a small command pool which runs synchronous commands.
pub struct OneTimeSubmitCommandBuffer {
    command_pool: raii::CommandPool,
    fence: raii::Fence,
    queue: Queue,
    render_device: Arc<RenderDevice>,
}

impl OneTimeSubmitCommandBuffer {
    /// Create a new one time submit command buffer.
    ///
    /// # Params
    ///
    /// - render_device: used to create the underlying Vulkan resources
    /// - queue: The queue which will be used for command buffer submission
    ///
    /// # Safety
    ///
    /// Unsafe because:
    /// - The application must destroy this resource before exiting.
    pub unsafe fn new(
        render_device: Arc<RenderDevice>,
        queue: Queue,
    ) -> Result<Self, GraphicsError> {
        let pool_create_info = vk::CommandPoolCreateInfo {
            flags: vk::CommandPoolCreateFlags::TRANSIENT,
            queue_family_index: queue.family_index(),
            ..Default::default()
        };
        let mut command_pool =
            raii::CommandPool::new(render_device.clone(), &pool_create_info)?;
        let _ = command_pool.allocate_primary_command_buffers(1)?;

        let begin_info = vk::CommandBufferBeginInfo {
            flags: vk::CommandBufferUsageFlags::ONE_TIME_SUBMIT,
            ..Default::default()
        };
        render_device.device().begin_command_buffer(
            command_pool.primary_command_buffer(0),
            &begin_info,
        )?;

        let fence_create_info = vk::FenceCreateInfo::default();
        let fence =
            raii::Fence::new(render_device.clone(), &fence_create_info)?;

        Ok(Self {
            command_pool,
            fence,
            queue,
            render_device,
        })
    }

    /// Get the current command buffer.
    ///
    /// # Safety
    ///
    /// Unsafe because:
    /// - the caller must not hold the command buffer after submission
    /// - the command buffer is always ready for commands, there is no need to
    ///   call vkBeginCommandBuffer
    /// - the command buffer never needs to be explicitly ended by
    ///   vkEndCommandBuffer
    pub unsafe fn command_buffer(&self) -> vk::CommandBuffer {
        self.command_pool.primary_command_buffer(0)
    }

    /// Submit the current command buffer and block the CPU until all commands
    /// complete on the GPU.
    ///
    /// # Safety
    ///
    /// Unsafe because:
    /// - the application is responsible for synchronizing access to any
    ///   resources referenced by the commands as they execute.
    pub unsafe fn sync_submit_and_reset(
        &mut self,
    ) -> Result<(), GraphicsError> {
        self.render_device
            .device()
            .end_command_buffer(self.command_pool.primary_command_buffer(0))?;

        let command_buffer_info = vk::CommandBufferSubmitInfo {
            command_buffer: self.command_pool.primary_command_buffer(0),
            ..Default::default()
        };
        self.render_device.device().queue_submit2(
            *self.queue.raw(),
            &[vk::SubmitInfo2 {
                wait_semaphore_info_count: 0,
                signal_semaphore_info_count: 0,
                command_buffer_info_count: 1,
                p_command_buffer_infos: &command_buffer_info,
                ..Default::default()
            }],
            self.fence.raw(),
        )?;
        self.render_device.device().wait_for_fences(
            &[self.fence.raw()],
            true,
            u64::MAX,
        )?;

        self.render_device.device().reset_command_pool(
            self.command_pool.raw(),
            vk::CommandPoolResetFlags::empty(),
        )?;
        let begin_info = vk::CommandBufferBeginInfo {
            flags: vk::CommandBufferUsageFlags::ONE_TIME_SUBMIT,
            ..Default::default()
        };
        self.render_device.device().begin_command_buffer(
            self.command_pool.primary_command_buffer(0),
            &begin_info,
        )?;
        Ok(())
    }
}
