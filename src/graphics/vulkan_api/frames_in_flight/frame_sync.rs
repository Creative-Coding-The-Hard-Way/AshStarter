use {
    crate::graphics::{vulkan_api::RenderDevice, GraphicsError},
    anyhow::Context,
    ash::vk,
};

/// All of the per-frame synchronization resources.
#[derive(Copy, Clone, Debug)]
pub(super) struct FrameSync {
    pub(super) index: usize,
    pub(super) command_buffer: vk::CommandBuffer,
    pub(super) command_pool: vk::CommandPool,
    pub(super) swapchain_image_acquired_semaphore: vk::Semaphore,
    pub(super) graphics_commands_completed_semaphore: vk::Semaphore,
    pub(super) graphics_commands_completed_fence: vk::Fence,
}

impl FrameSync {
    /// Create synchronization resources for a single in-flight frame.
    ///
    /// # Params
    ///
    /// * `render_device` - the render device used to create and destroy all
    ///   frame resources
    /// * `index` - the frame's index
    ///
    /// # Safety
    ///
    /// Unsafe because:
    ///   - all resources must be destroyed before application exit
    pub unsafe fn new(
        render_device: &RenderDevice,
        index: usize,
    ) -> Result<Self, GraphicsError> {
        let swapchain_image_acquired_semaphore = unsafe {
            let create_info = vk::SemaphoreCreateInfo::default();
            render_device
                .device()
                .create_semaphore(&create_info, None)?
        };
        render_device.set_debug_name(
            swapchain_image_acquired_semaphore,
            vk::ObjectType::SEMAPHORE,
            format!("Frame {index} Swapchain Image Acquired"),
        );

        let graphics_commands_completed_semaphore = unsafe {
            let create_info = vk::SemaphoreCreateInfo::default();
            render_device
                .device()
                .create_semaphore(&create_info, None)?
        };
        render_device.set_debug_name(
            graphics_commands_completed_semaphore,
            vk::ObjectType::SEMAPHORE,
            format!("Frame {index} Graphics Commands Completed"),
        );

        let graphics_commands_completed_fence = unsafe {
            let create_info = vk::FenceCreateInfo {
                flags: vk::FenceCreateFlags::SIGNALED,
                ..Default::default()
            };
            render_device.device().create_fence(&create_info, None)?
        };
        render_device.set_debug_name(
            graphics_commands_completed_fence,
            vk::ObjectType::FENCE,
            format!("Frame {index} Graphics Commands Completed"),
        );

        let command_pool = unsafe {
            let create_info = vk::CommandPoolCreateInfo {
                flags: vk::CommandPoolCreateFlags::TRANSIENT,
                ..Default::default()
            };
            render_device
                .device()
                .create_command_pool(&create_info, None)?
        };
        render_device.set_debug_name(
            command_pool,
            vk::ObjectType::COMMAND_POOL,
            format!("Frame {index} Command Pool"),
        );

        let command_buffer = unsafe {
            let create_info = vk::CommandBufferAllocateInfo {
                command_pool,
                level: vk::CommandBufferLevel::PRIMARY,
                command_buffer_count: 1,
                ..Default::default()
            };
            render_device
                .device()
                .allocate_command_buffers(&create_info)?
                .pop()
                .unwrap()
        };
        render_device.set_debug_name(
            command_buffer,
            vk::ObjectType::COMMAND_BUFFER,
            format!("Frame {index} Command Buffer"),
        );

        Ok(Self {
            index,
            command_buffer,
            command_pool,
            swapchain_image_acquired_semaphore,
            graphics_commands_completed_semaphore,
            graphics_commands_completed_fence,
        })
    }

    /// Wait for this frame's last graphics command submission to complete.
    ///
    /// # Params
    ///
    /// * `render_device` - the device used to create the frame sync resources
    pub fn wait_for_graphics_commands_to_complete(
        &self,
        render_device: &RenderDevice,
    ) -> Result<(), GraphicsError> {
        unsafe {
            render_device
                .device()
                .wait_for_fences(
                    &[self.graphics_commands_completed_fence],
                    true,
                    u64::MAX,
                )
                .context(
                    "Error while waiting for graphics commands to complete",
                )?
        }
        Ok(())
    }

    /// Reset and restart the command buffer for this frame.
    ///
    /// # Params
    ///
    /// * `render_device` - the device used to create the frame sync resources
    pub fn wait_and_restart_command_buffer(
        &self,
        render_device: &RenderDevice,
    ) -> Result<(), GraphicsError> {
        self.wait_for_graphics_commands_to_complete(render_device)?;
        unsafe {
            // SAFE because we wait for the previous submission's commands to
            // complete before resetting and restarting resources.
            render_device
                .device()
                .reset_fences(&[self.graphics_commands_completed_fence])
                .with_context(|| {
                    format!(
                        "Could not reset graphics completed fence for frame {}",
                        self.index
                    )
                })?;
            render_device
                .device()
                .reset_command_pool(
                    self.command_pool,
                    vk::CommandPoolResetFlags::empty(),
                )
                .with_context(|| {
                    format!(
                        "Could not reset command pool for frame {}",
                        self.index
                    )
                })?;
            let begin_info = vk::CommandBufferBeginInfo {
                flags: vk::CommandBufferUsageFlags::ONE_TIME_SUBMIT,
                ..Default::default()
            };
            render_device
                .device()
                .begin_command_buffer(self.command_buffer, &begin_info)
                .with_context(|| {
                    format!(
                        "Could not begin command buffer for frame {}",
                        self.index
                    )
                })?;
        }
        Ok(())
    }

    /// Destroy all resources used by this frame.
    ///
    /// # Params
    ///
    /// * `render_device` - the device used to create the frame sync resources
    ///
    /// # Safety
    ///
    /// Unsafe because:
    ///  - the caller should wait for all graphics commands which referenc this
    ///    frame to complete before calling this function.
    pub unsafe fn destroy(&mut self, render_device: &RenderDevice) {
        render_device
            .device()
            .destroy_command_pool(self.command_pool, None);
        render_device
            .device()
            .destroy_semaphore(self.swapchain_image_acquired_semaphore, None);
        render_device.device().destroy_semaphore(
            self.graphics_commands_completed_semaphore,
            None,
        );
        render_device
            .device()
            .destroy_fence(self.graphics_commands_completed_fence, None);
    }
}
