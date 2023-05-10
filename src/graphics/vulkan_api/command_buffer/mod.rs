use {
    crate::graphics::{
        vulkan_api::{Queue, RenderDevice},
        GraphicsError,
    },
    ash::vk,
    ccthw_ash_instance::VulkanHandle,
};

/// A utility for managing a small command pool which runs synchronous commands.
pub struct OneTimeSubmitCommandBuffer {
    command_pool: vk::CommandPool,
    command_buffer: vk::CommandBuffer,
    fence: vk::Fence,
    queue: Queue,
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
        render_device: &RenderDevice,
        queue: Queue,
    ) -> Result<Self, GraphicsError> {
        let pool_create_info = vk::CommandPoolCreateInfo {
            flags: vk::CommandPoolCreateFlags::TRANSIENT,
            queue_family_index: queue.family_index(),
            ..Default::default()
        };
        let command_pool = render_device
            .device()
            .create_command_pool(&pool_create_info, None)?;

        let buffer_create_info = vk::CommandBufferAllocateInfo {
            command_pool,
            level: vk::CommandBufferLevel::PRIMARY,
            command_buffer_count: 1,
            ..Default::default()
        };
        let command_buffer = render_device
            .device()
            .allocate_command_buffers(&buffer_create_info)?[0];

        let begin_info = vk::CommandBufferBeginInfo {
            flags: vk::CommandBufferUsageFlags::ONE_TIME_SUBMIT,
            ..Default::default()
        };
        render_device
            .device()
            .begin_command_buffer(command_buffer, &begin_info)?;

        let fence_create_info = vk::FenceCreateInfo::default();
        let fence = render_device
            .device()
            .create_fence(&fence_create_info, None)?;

        Ok(Self {
            command_buffer,
            command_pool,
            fence,
            queue,
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
        self.command_buffer
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
        render_device: &RenderDevice,
    ) -> Result<(), GraphicsError> {
        render_device
            .device()
            .end_command_buffer(self.command_buffer)?;

        let command_buffer_info = vk::CommandBufferSubmitInfo {
            command_buffer: self.command_buffer,
            ..Default::default()
        };
        render_device.device().queue_submit2(
            *self.queue.raw(),
            &[vk::SubmitInfo2 {
                wait_semaphore_info_count: 0,
                signal_semaphore_info_count: 0,
                command_buffer_info_count: 1,
                p_command_buffer_infos: &command_buffer_info,
                ..Default::default()
            }],
            self.fence,
        )?;
        render_device.device().wait_for_fences(
            &[self.fence],
            true,
            u64::MAX,
        )?;

        render_device.device().reset_command_pool(
            self.command_pool,
            vk::CommandPoolResetFlags::empty(),
        )?;
        let begin_info = vk::CommandBufferBeginInfo {
            flags: vk::CommandBufferUsageFlags::ONE_TIME_SUBMIT,
            ..Default::default()
        };
        render_device
            .device()
            .begin_command_buffer(self.command_buffer, &begin_info)?;
        Ok(())
    }

    /// Destroy all of the underlying Vulkan resources.
    ///
    /// # Param
    ///
    /// - render_device: the render device is used to destroy the Vulkan
    ///   resources
    ///
    /// # Safety
    ///
    /// Unsafe because:
    /// - the OneTimeSubmitCommandBuffer must not be used after calling destroy
    /// - destroy must be called before the render device is dropped
    pub unsafe fn destroy(&mut self, render_device: &RenderDevice) {
        render_device
            .device()
            .destroy_command_pool(self.command_pool, None);
        render_device.device().destroy_fence(self.fence, None);
    }
}
