use super::{FrameError, FramePipeline, PerFrame};

use crate::vulkan::{errors::SwapchainError, RenderDevice, SemaphorePool};

use ::{
    anyhow::{Context, Result},
    ash::{version::DeviceV1_0, vk},
};

impl FramePipeline {
    pub fn new(vk_dev: &RenderDevice) -> Result<Self> {
        let mut semaphore_pool = SemaphorePool::new();

        // build per-frame resources
        let mut frames = vec![];
        for i in 0..vk_dev.swapchain.as_ref().unwrap().image_views.len() {
            frames.push(PerFrame::new(&vk_dev, &mut semaphore_pool, i)?);
        }

        Ok(Self {
            semaphore_pool,
            frames,
        })
    }

    pub fn draw_and_present<F>(
        &mut self,
        vk_dev: &RenderDevice,
        build_frame_commands: F,
    ) -> Result<(), FrameError>
    where
        F: FnOnce(&RenderDevice, usize, vk::CommandBuffer) -> Result<()>,
    {
        let index = self.acquire_next_image(vk_dev)?;
        self.submit_frame_commands(vk_dev, index, build_frame_commands)?;
        self.present_image(vk_dev, index)
    }

    pub unsafe fn destroy(&mut self, vk_dev: &RenderDevice) {
        for frame in self.frames.drain(..) {
            frame.destroy(vk_dev);
        }
        self.semaphore_pool.destroy(vk_dev);
    }

    pub unsafe fn rebuild_swapchain_resources(
        &mut self,
        vk_dev: &RenderDevice,
    ) -> Result<()> {
        for frame in self.frames.drain(..) {
            frame.destroy(vk_dev);
        }
        for i in 0..vk_dev.swapchain.as_ref().unwrap().image_views.len() {
            self.frames.push(PerFrame::new(
                &vk_dev,
                &mut self.semaphore_pool,
                i,
            )?);
        }
        Ok(())
    }
}

impl FramePipeline {
    fn acquire_next_image(
        &mut self,
        vk_dev: &RenderDevice,
    ) -> Result<usize, FrameError> {
        let acquire_semaphore =
            self.semaphore_pool.get_semaphore(vk_dev).context(
                "unable to get a semaphore for the next swapchain image",
            )?;
        let index = {
            let result = vk_dev.acquire_next_swapchain_image(
                acquire_semaphore,
                vk::Fence::null(),
            );
            if result.is_err() {
                self.semaphore_pool.return_semaphore(acquire_semaphore);
            }
            if let Err(SwapchainError::NeedsRebuild) = result {
                return Err(FrameError::SwapchainNeedsRebuild);
            }
            result.context("unable to acquire the next swapchain image")?
        };

        // Replace the old acquire_semaphore with the new one which will be
        // signaled when this frame is ready.
        self.semaphore_pool
            .return_semaphore(self.frames[index].acquire_semaphore);
        self.frames[index].acquire_semaphore = acquire_semaphore;

        // This typically is a no-op because multiple other frames have been
        // rendered between this time and the last time the frame was rendered.
        if self.frames[index].queue_submit_fence != vk::Fence::null() {
            unsafe {
                vk_dev
                    .logical_device
                    .wait_for_fences(
                        &[self.frames[index].queue_submit_fence],
                        true,
                        u64::MAX,
                    )
                    .context("error waiting for queue submission fence")?;
                vk_dev
                    .logical_device
                    .reset_fences(&[self.frames[index].queue_submit_fence])
                    .context("unable to reset queue submission fence")?;
            }
        }

        unsafe {
            vk_dev
                .logical_device
                .reset_command_pool(
                    self.frames[index].command_pool,
                    vk::CommandPoolResetFlags::empty(),
                )
                .context("unable to reset the frame command pool")?;
        }
        Ok(index)
    }

    fn submit_frame_commands<F>(
        &mut self,
        vk_dev: &RenderDevice,
        index: usize,
        fill_command_buffer: F,
    ) -> Result<(), FrameError>
    where
        F: FnOnce(&RenderDevice, usize, vk::CommandBuffer) -> Result<()>,
    {
        let current_frame = &self.frames[index];

        // build the command buffer
        unsafe {
            let begin_info = vk::CommandBufferBeginInfo {
                flags: vk::CommandBufferUsageFlags::ONE_TIME_SUBMIT,
                ..Default::default()
            };
            vk_dev
                .logical_device
                .begin_command_buffer(current_frame.command_buffer, &begin_info)
                .with_context(|| {
                    format!(
                        "Unable to begin the command buffer for frame {}",
                        index
                    )
                })?;

            fill_command_buffer(vk_dev, index, current_frame.command_buffer)?;

            vk_dev
                .logical_device
                .end_command_buffer(current_frame.command_buffer)
                .with_context(|| {
                    format!("Unable to end command buffer for frame {}", index)
                })?;
        }

        // submit the command buffer
        let wait_stage = vk::PipelineStageFlags::COLOR_ATTACHMENT_OUTPUT;
        let submit_info = vk::SubmitInfo {
            command_buffer_count: 1,
            p_command_buffers: &current_frame.command_buffer,
            wait_semaphore_count: 1,
            p_wait_semaphores: &current_frame.acquire_semaphore,
            p_wait_dst_stage_mask: &wait_stage,
            signal_semaphore_count: 1,
            p_signal_semaphores: &current_frame.release_semaphore,
            ..Default::default()
        };
        unsafe {
            vk_dev
                .logical_device
                .queue_submit(
                    vk_dev.graphics_queue.queue,
                    &[submit_info],
                    current_frame.queue_submit_fence,
                )
                .with_context(|| {
                    format!(
                        "Unable to submit graphics commands on frame {}",
                        index
                    )
                })?;
        }

        Ok(())
    }

    fn present_image(
        &mut self,
        vk_dev: &RenderDevice,
        index: usize,
    ) -> Result<(), FrameError> {
        let index_u32 = index as u32;
        let current_frame = &self.frames[index];
        let present_info = vk::PresentInfoKHR {
            swapchain_count: 1,
            p_swapchains: &vk_dev.swapchain().khr,
            p_image_indices: &index_u32,
            wait_semaphore_count: 1,
            p_wait_semaphores: &current_frame.release_semaphore,
            ..Default::default()
        };
        unsafe {
            vk_dev
                .swapchain()
                .loader
                .queue_present(vk_dev.present_queue.queue, &present_info)
                .with_context(|| "Unable to present the swapchain image")?;
        }
        Ok(())
    }
}
