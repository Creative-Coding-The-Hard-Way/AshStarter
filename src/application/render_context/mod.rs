mod frame_sync;

use self::frame_sync::FrameSync;
use crate::{
    application::GraphicsPipeline,
    rendering::{Device, Swapchain},
};

use anyhow::{Context, Result};
use ash::{version::DeviceV1_0, vk};
use std::sync::Arc;

#[derive(Debug, Copy, Clone, PartialEq)]
pub enum SwapchainState {
    Ok,
    NeedsRebuild,
}

pub struct RenderContext {
    command_pools: Vec<vk::CommandPool>,
    command_buffers: Vec<vk::CommandBuffer>,

    images_in_flight: Vec<FrameSync>,
    previous_frame: usize,

    swapchain_state: SwapchainState,

    graphics_pipeline: Arc<GraphicsPipeline>,
    swapchain: Arc<Swapchain>,
    device: Arc<Device>,
}

impl RenderContext {
    pub fn new(
        device: &Arc<Device>,
        swapchain: &Arc<Swapchain>,
        graphics_pipeline: &Arc<GraphicsPipeline>,
    ) -> Result<Self> {
        let command_pools = create_command_pools(device, swapchain)?;
        let command_buffers = create_command_buffers(device, &command_pools)?;

        let images_in_flight =
            FrameSync::for_n_frames(device, swapchain.framebuffers.len())?;

        let frame = Self {
            command_pools,
            command_buffers,

            images_in_flight,
            swapchain_state: SwapchainState::Ok,

            previous_frame: 0, // always 'start' on frame 0

            graphics_pipeline: graphics_pipeline.clone(),
            swapchain: swapchain.clone(),
            device: device.clone(),
        };

        frame.record_buffer_commands()?;

        Ok(frame)
    }

    /// Signal that the swapchain needs to be rebuilt before the next frame
    /// is rendered.
    pub fn needs_rebuild(&mut self) {
        self.swapchain_state = SwapchainState::NeedsRebuild;
    }

    /// Render a single application frame.
    pub fn draw_frame(&mut self) -> Result<()> {
        if self.swapchain_state == SwapchainState::NeedsRebuild {
            return self.rebuild_swapchain();
        }

        let acquired_semaphore = self.images_in_flight[self.previous_frame]
            .image_available_semaphore;

        let result = unsafe {
            self.swapchain.swapchain_loader.acquire_next_image(
                self.swapchain.swapchain,
                u64::MAX,
                acquired_semaphore,
                vk::Fence::null(),
            )
        };
        if let Err(vk::Result::ERROR_OUT_OF_DATE_KHR) = result {
            return self.rebuild_swapchain();
        }
        if let Ok((_, true)) = result {
            return self.rebuild_swapchain();
        }

        let (index, _) = result?;
        let current_frame_sync = &self.images_in_flight[index as usize];

        unsafe {
            self.device.logical_device.wait_for_fences(
                &[current_frame_sync.graphics_finished_fence],
                true,
                u64::MAX,
            )?;
        }

        let wait_semaphores = [acquired_semaphore];
        let wait_stages = [vk::PipelineStageFlags::COLOR_ATTACHMENT_OUTPUT];
        let command_buffers = [self.command_buffers[index as usize]];
        let render_finished_signal_semaphores =
            [current_frame_sync.render_finished_semaphore];
        let submit_info = [vk::SubmitInfo::builder()
            .wait_semaphores(&wait_semaphores)
            .wait_dst_stage_mask(&wait_stages)
            .command_buffers(&command_buffers)
            .signal_semaphores(&render_finished_signal_semaphores)
            .build()];

        unsafe {
            let graphics_queue = self.device.graphics_queue.acquire();
            self.device
                .logical_device
                .reset_fences(&[current_frame_sync.graphics_finished_fence])?;
            self.device.logical_device.queue_submit(
                *graphics_queue,
                &submit_info,
                current_frame_sync.graphics_finished_fence,
            )?;
        }

        let swapchains = [self.swapchain.swapchain];
        let indices = [index];
        let present_info = vk::PresentInfoKHR::builder()
            .wait_semaphores(&render_finished_signal_semaphores)
            .swapchains(&swapchains)
            .image_indices(&indices);

        let result = unsafe {
            let present_queue = self.device.present_queue.acquire();
            self.swapchain
                .swapchain_loader
                .queue_present(*present_queue, &present_info)
        };
        if Err(vk::Result::ERROR_OUT_OF_DATE_KHR) == result {
            return self.rebuild_swapchain();
        }

        self.previous_frame = index as usize;
        Ok(())
    }

    fn rebuild_swapchain(&mut self) -> Result<()> {
        unsafe {
            let device = &self.device;
            self.device.logical_device.device_wait_idle()?;
            for (pool, buffer) in
                self.command_pools.iter().zip(self.command_buffers.iter())
            {
                device
                    .logical_device
                    .free_command_buffers(*pool, &[*buffer]);
                device.logical_device.destroy_command_pool(*pool, None);
            }
            self.command_pools.clear();
            self.command_buffers.clear();
            self.images_in_flight
                .drain(..)
                .for_each(|frame_sync| frame_sync.destroy(device));
        }

        self.swapchain = self.swapchain.rebuild()?;
        self.images_in_flight = FrameSync::for_n_frames(
            &self.device,
            self.swapchain.framebuffers.len(),
        )?;
        self.command_pools =
            create_command_pools(&self.device, &self.swapchain)?;
        self.command_buffers =
            create_command_buffers(&self.device, &self.command_pools)?;
        self.graphics_pipeline =
            GraphicsPipeline::new(&self.device, &self.swapchain)?;
        self.record_buffer_commands()?;
        self.swapchain_state = SwapchainState::Ok;

        Ok(())
    }

    fn record_buffer_commands(&self) -> Result<()> {
        for (framebuffer, command_buffer) in self
            .swapchain
            .framebuffers
            .iter()
            .zip(self.command_buffers.iter())
        {
            // begin the command buffer
            let begin_info = vk::CommandBufferBeginInfo::builder()
                .flags(vk::CommandBufferUsageFlags::empty());

            // begin the render pass
            let clear_values = [vk::ClearValue {
                color: vk::ClearColorValue {
                    float32: [0.0, 0.0, 0.0, 1.0],
                },
            }];
            let render_pass_begin_info = vk::RenderPassBeginInfo::builder()
                .render_pass(self.swapchain.render_pass)
                .framebuffer(*framebuffer)
                .render_area(vk::Rect2D {
                    offset: vk::Offset2D { x: 0, y: 0 },
                    extent: self.swapchain.extent,
                })
                .clear_values(&clear_values);

            unsafe {
                // begin the command buffer
                self.device
                    .logical_device
                    .begin_command_buffer(*command_buffer, &begin_info)?;

                // begin the render pass
                self.device.logical_device.cmd_begin_render_pass(
                    *command_buffer,
                    &render_pass_begin_info,
                    vk::SubpassContents::INLINE,
                );

                // bind the graphics pipeline
                self.device.logical_device.cmd_bind_pipeline(
                    *command_buffer,
                    vk::PipelineBindPoint::GRAPHICS,
                    self.graphics_pipeline.pipeline,
                );

                // draw
                self.device.logical_device.cmd_draw(
                    *command_buffer,
                    3, // vertex count
                    1, // instance count
                    0, // first vertex
                    0, // first instance
                );

                // end the render pass
                self.device
                    .logical_device
                    .cmd_end_render_pass(*command_buffer);

                // end the buffer
                self.device
                    .logical_device
                    .end_command_buffer(*command_buffer)?;
            }
        }

        Ok(())
    }
}

impl Drop for RenderContext {
    fn drop(&mut self) {
        unsafe {
            // don't delete anything until the GPU has stoped using our
            // resources
            self.device
                .logical_device
                .device_wait_idle()
                .expect("wait for device to idle");

            let device = &self.device;
            self.images_in_flight
                .drain(..)
                .for_each(|frame| frame.destroy(device));

            // safe to delete now
            for (pool, buffer) in
                self.command_pools.iter().zip(self.command_buffers.iter())
            {
                device
                    .logical_device
                    .free_command_buffers(*pool, &[*buffer]);
                device.logical_device.destroy_command_pool(*pool, None);
            }
        }
    }
}

/// Create the command buffer pool.
///
/// The caller is responsible for destroying the pool before the device.
fn create_command_pools(
    device: &Device,
    swapchain: &Swapchain,
) -> Result<Vec<vk::CommandPool>> {
    let mut pools = vec![];
    for i in 0..swapchain.framebuffers.len() {
        let create_info = vk::CommandPoolCreateInfo::builder()
            .queue_family_index(device.graphics_queue.family_id)
            .flags(vk::CommandPoolCreateFlags::empty());
        let command_pool = unsafe {
            device
                .logical_device
                .create_command_pool(&create_info, None)
                .context("unable to create the command pool")?
        };
        device.name_vulkan_object(
            format!("Graphics Command Pool {}", i),
            vk::ObjectType::COMMAND_POOL,
            &command_pool,
        )?;
        pools.push(command_pool);
    }
    Ok(pools)
}

/// Create one command buffer for each frame.
///
/// The caller is responsible for deallocating the command buffers when done
/// using them.
fn create_command_buffers(
    device: &Device,
    command_pools: &[vk::CommandPool],
) -> Result<Vec<vk::CommandBuffer>> {
    let mut buffers = vec![];
    for pool in command_pools {
        let create_info = vk::CommandBufferAllocateInfo::builder()
            .command_pool(*pool)
            .level(vk::CommandBufferLevel::PRIMARY)
            .command_buffer_count(1);
        let command_buffers = unsafe {
            device
                .logical_device
                .allocate_command_buffers(&create_info)?
        };
        buffers.push(command_buffers[0]);
    }
    Ok(buffers)
}
