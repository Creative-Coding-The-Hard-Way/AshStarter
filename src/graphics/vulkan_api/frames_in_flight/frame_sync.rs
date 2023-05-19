use {
    crate::graphics::{
        vulkan_api::{raii, RenderDevice},
        GraphicsError,
    },
    anyhow::Context,
    ash::vk,
    std::sync::Arc,
};

/// All of the per-frame synchronization resources.
#[derive(Debug)]
pub(super) struct FrameSync {
    pub(super) index: usize,
    pub(super) command_pool: raii::CommandPool,
    pub(super) swapchain_image_acquired_semaphore: raii::Semaphore,
    pub(super) graphics_commands_completed_semaphore: raii::Semaphore,
    pub(super) graphics_commands_completed_fence: raii::Fence,
    render_device: Arc<RenderDevice>,
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
        render_device: Arc<RenderDevice>,
        index: usize,
    ) -> Result<Self, GraphicsError> {
        let swapchain_image_acquired_semaphore = unsafe {
            raii::Semaphore::new(
                render_device.clone(),
                &vk::SemaphoreCreateInfo::default(),
            )?
        };
        swapchain_image_acquired_semaphore
            .set_debug_name(format!("Frame {index} Swapchain Image Acquired"));

        let graphics_commands_completed_semaphore = unsafe {
            raii::Semaphore::new(
                render_device.clone(),
                &vk::SemaphoreCreateInfo::default(),
            )?
        };
        graphics_commands_completed_semaphore.set_debug_name(format!(
            "Frame {index} Graphics Commands Completed"
        ));

        let graphics_commands_completed_fence = unsafe {
            let create_info = vk::FenceCreateInfo {
                flags: vk::FenceCreateFlags::SIGNALED,
                ..Default::default()
            };
            raii::Fence::new(render_device.clone(), &create_info)?
        };
        graphics_commands_completed_fence.set_debug_name(format!(
            "Frame {index} Graphics Commands Completed"
        ));

        let mut command_pool = unsafe {
            let create_info = vk::CommandPoolCreateInfo {
                flags: vk::CommandPoolCreateFlags::TRANSIENT,
                ..Default::default()
            };
            raii::CommandPool::new(render_device.clone(), &create_info)?
        };
        command_pool.set_debug_name(format!("Frame {index} Command Pool"));
        let _ = command_pool.allocate_primary_command_buffers(1);

        Ok(Self {
            index,
            command_pool,
            swapchain_image_acquired_semaphore,
            graphics_commands_completed_semaphore,
            graphics_commands_completed_fence,
            render_device,
        })
    }

    /// Wait for this frame's last graphics command submission to complete.
    ///
    /// # Params
    ///
    /// * `render_device` - the device used to create the frame sync resources
    pub fn wait_for_graphics_commands_to_complete(
        &self,
    ) -> Result<(), GraphicsError> {
        unsafe {
            self.render_device
                .device()
                .wait_for_fences(
                    &[self.graphics_commands_completed_fence.raw()],
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
    pub fn wait_and_restart_command_buffer(&self) -> Result<(), GraphicsError> {
        self.wait_for_graphics_commands_to_complete()?;
        unsafe {
            // SAFE because we wait for the previous submission's commands to
            // complete before resetting and restarting resources.
            self.render_device
                .device()
                .reset_fences(&[self.graphics_commands_completed_fence.raw()])
                .with_context(|| {
                    format!(
                        "Could not reset graphics completed fence for frame {}",
                        self.index
                    )
                })?;
            self.render_device
                .device()
                .reset_command_pool(
                    self.command_pool.raw(),
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
            self.render_device
                .device()
                .begin_command_buffer(
                    self.command_pool.primary_command_buffer(0),
                    &begin_info,
                )
                .with_context(|| {
                    format!(
                        "Could not begin command buffer for frame {}",
                        self.index
                    )
                })?;
        }
        Ok(())
    }
}
