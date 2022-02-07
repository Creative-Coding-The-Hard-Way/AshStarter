use ::{anyhow::Context, ash::vk, std::sync::Arc};

use crate::{
    frame_pipeline::{FrameError, PerFrame},
    vulkan::{
        errors::{SwapchainError, VulkanError},
        sync::SemaphorePool,
        CommandBuffer, RenderDevice, VulkanDebug,
    },
    vulkan_ext::CommandBufferExt,
};

/// A frame pipeline aids with the swapchain acquire->render->present workflow.
pub struct FramePipeline {
    frames: Vec<PerFrame>,
    semaphore_pool: SemaphorePool,

    /// The device used to create this frame pipeline.
    pub vk_dev: Arc<RenderDevice>,
}

impl FramePipeline {
    pub fn new(vk_dev: Arc<RenderDevice>) -> Result<Self, FrameError> {
        let mut frame_pipeline = Self {
            frames: vec![],
            semaphore_pool: SemaphorePool::new(vk_dev.clone()),
            vk_dev,
        };
        frame_pipeline.rebuild_swapchain_resources()?;
        Ok(frame_pipeline)
    }

    /// Begin rendering a single frame.
    ///
    /// # Returns
    ///
    /// A tuple containing two things:
    ///
    /// 1. The index of the swapchain image being targeted. This is useful for
    ///    renderers which have per-swapchain-image resources, it is also used
    ///    when ending the frame.
    ///
    /// 2. The graphics command buffer for the current frame. Renderers can
    ///    add commands to the buffer for execution prior to the call to
    ///    `end_frame`.
    ///
    pub fn begin_frame(
        &mut self,
    ) -> Result<(usize, &CommandBuffer), FrameError> {
        let current_image = self.acquire_next_image()?;
        let cmd = self.prepare_frame_command_buffer(current_image)?;
        Ok((current_image, cmd))
    }

    /// End rendering a single frame. This submits all commands on the graphics
    /// command buffer and schedules the swapchain image for presentation.
    pub fn end_frame(
        &mut self,
        current_image: usize,
    ) -> Result<(), FrameError> {
        self.submit_and_present(current_image)?;
        Ok(())
    }

    /// Rebuild all swapchain-dependent resources.
    pub fn rebuild_swapchain_resources(&mut self) -> Result<(), FrameError> {
        for frame in self.frames.drain(..) {
            frame
                .queue_submit_fence
                .wait_and_reset()
                .map_err(VulkanError::FenceError)?;
        }
        for i in 0..self.vk_dev.swapchain_image_count() {
            let frame = PerFrame::new(self.vk_dev.clone())?;
            frame
                .set_debug_name(format!("Frame {}", i))
                .map_err(VulkanError::VulkanDebugError)?;
            self.frames.push(frame);
        }
        Ok(())
    }
}

impl FramePipeline {
    fn acquire_next_image(&mut self) -> Result<usize, FrameError> {
        let acquire_semaphore = self.semaphore_pool.get_semaphore().context(
            "unable to get a semaphore for the next swapchain image",
        )?;
        let index = {
            let result = self.vk_dev.acquire_next_swapchain_image(
                acquire_semaphore.raw,
                vk::Fence::null(),
            );
            if let Err(SwapchainError::NeedsRebuild) = result {
                return Err(FrameError::SwapchainNeedsRebuild);
            }
            result.context("unable to acquire the next swapchain image")?
        };

        // Replace the old acquire_semaphore with the new one which will be
        // signaled when this frame is ready.
        let old_semaphore = self.frames[index]
            .acquire_semaphore
            .replace(acquire_semaphore);
        if let Some(semaphore) = old_semaphore {
            self.semaphore_pool.return_semaphore(semaphore);
        }

        // This typically is a no-op because multiple other frames have been
        // rendered between this time and the last time the frame was rendered.
        self.frames[index]
            .queue_submit_fence
            .wait_and_reset()
            .map_err(VulkanError::FenceError)?;

        self.frames[index]
            .command_pool
            .reset()
            .map_err(VulkanError::CommandBufferError)?;

        Ok(index)
    }

    fn prepare_frame_command_buffer(
        &mut self,
        current_image: usize,
    ) -> Result<&CommandBuffer, FrameError> {
        let current_frame = &self.frames[current_image];
        unsafe {
            current_frame
                .command_buffer
                .begin_one_time_submit()
                .with_context(|| {
                    format!(
                        "Unable to begin the command buffer for frame {}",
                        current_image
                    )
                })?;
        }
        Ok(&current_frame.command_buffer)
    }

    fn submit_and_present(&mut self, index: usize) -> Result<(), FrameError> {
        let current_frame = &self.frames[index];
        unsafe {
            current_frame
                .command_buffer
                .end_commands()
                .with_context(|| {
                    format!("Unable to end command buffer for frame {}", index)
                })?;
        }

        // submit the command buffer
        let wait_stage = vk::PipelineStageFlags::COLOR_ATTACHMENT_OUTPUT;
        let submit_info = vk::SubmitInfo {
            command_buffer_count: 1,
            p_command_buffers: &current_frame.command_buffer.raw,
            wait_semaphore_count: 1,
            p_wait_semaphores: &current_frame
                .acquire_semaphore
                .as_ref()
                .unwrap()
                .raw,
            p_wait_dst_stage_mask: &wait_stage,
            signal_semaphore_count: 1,
            p_signal_semaphores: &current_frame.release_semaphore.raw,
            ..Default::default()
        };
        unsafe {
            self.vk_dev
                .logical_device
                .queue_submit(
                    self.vk_dev.graphics_queue.queue,
                    &[submit_info],
                    current_frame.queue_submit_fence.raw,
                )
                .with_context(|| {
                    format!(
                        "Unable to submit graphics commands on frame {}",
                        index
                    )
                })?;
        }

        let index_u32 = index as u32;
        let current_frame = &self.frames[index];

        self.vk_dev.with_swapchain(|swapchain| {
            let present_info = vk::PresentInfoKHR {
                swapchain_count: 1,
                p_swapchains: &swapchain.khr,
                p_image_indices: &index_u32,
                wait_semaphore_count: 1,
                p_wait_semaphores: &current_frame.release_semaphore.raw,
                ..Default::default()
            };
            unsafe {
                swapchain
                    .loader
                    .queue_present(
                        self.vk_dev.present_queue.queue,
                        &present_info,
                    )
                    .with_context(|| "Unable to present the swapchain image")
            }
        })?;
        Ok(())
    }
}
