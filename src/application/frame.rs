use crate::application::{Device, GraphicsPipeline, Swapchain};

use anyhow::{Context, Result};
use ash::{version::DeviceV1_0, vk};
use std::{iter::Cycle, slice::Iter, sync::Arc};

/// Synchronization primitives used to coordinate rendering each frame without
/// accidentally sharing resources.
struct FrameSync {
    pub image_available_semaphore: vk::Semaphore,
    pub render_finished_semaphore: vk::Semaphore,
}

impl FrameSync {
    /// Create a vector of named frame sync objects.
    pub fn for_n_frames(
        device: &Device,
        max_frames_in_flight: usize,
    ) -> Result<Vec<Self>> {
        let mut frames_in_flight = vec![];
        for i in 0..max_frames_in_flight {
            frames_in_flight
                .push(FrameSync::new(device, format!("FrameSync {}", i))?);
        }
        Ok(frames_in_flight)
    }

    /// Create the synchronization primitives used for each frame.
    ///
    pub fn new<Name>(device: &Device, name: Name) -> Result<Self>
    where
        Name: Into<String>,
    {
        let owned_name = name.into();
        let image_available_semaphore = unsafe {
            device
                .logical_device
                .create_semaphore(&vk::SemaphoreCreateInfo::builder(), None)?
        };
        device.name_vulkan_object(
            format!("{} Swapchain Image Available", &owned_name),
            vk::ObjectType::SEMAPHORE,
            &image_available_semaphore,
        )?;

        let render_finished_semaphore = unsafe {
            device
                .logical_device
                .create_semaphore(&vk::SemaphoreCreateInfo::builder(), None)?
        };
        device.name_vulkan_object(
            format!("{} Render Finished", &owned_name),
            vk::ObjectType::SEMAPHORE,
            &render_finished_semaphore,
        )?;

        Ok(Self {
            image_available_semaphore,
            render_finished_semaphore,
        })
    }

    /// Called by the owner when all sync resources should be destroyed.
    pub unsafe fn destroy(self, device: &Device) {
        //! This function does no checking that the semaphores are done being used,
        //! that is up to the owner. (for example, wait for the device to idle)
        device
            .logical_device
            .destroy_semaphore(self.image_available_semaphore, None);
        device
            .logical_device
            .destroy_semaphore(self.render_finished_semaphore, None);
    }
}

pub struct Frame {
    command_pool: vk::CommandPool,
    command_buffers: Vec<vk::CommandBuffer>,

    frames_in_flight: Vec<FrameSync>,
    current_frame: usize,

    graphics_pipeline: Arc<GraphicsPipeline>,
    swapchain: Arc<Swapchain>,
    device: Arc<Device>,
}

impl Frame {
    pub fn new(
        device: &Arc<Device>,
        swapchain: &Arc<Swapchain>,
        graphics_pipeline: &Arc<GraphicsPipeline>,
    ) -> Result<Self> {
        let command_pool = create_command_pool(device)?;
        let command_buffers =
            create_command_buffers(device, swapchain, &command_pool)?;

        let frames_in_flight = FrameSync::for_n_frames(device, 3)?;
        let current_frame = 0;

        let frame = Self {
            command_pool,
            command_buffers,
            frames_in_flight,
            current_frame,

            graphics_pipeline: graphics_pipeline.clone(),
            swapchain: swapchain.clone(),
            device: device.clone(),
        };

        frame.record_buffer_commands()?;

        Ok(frame)
    }

    /// Render a single application frame.
    pub fn draw_frame(&mut self) -> Result<()> {
        self.current_frame =
            (self.current_frame + 1) % self.frames_in_flight.len();
        let frame_sync = &self.frames_in_flight[self.current_frame];

        let (index, _needs_rebuild) = unsafe {
            self.swapchain.swapchain_loader.acquire_next_image(
                self.swapchain.swapchain,
                u64::MAX,
                frame_sync.image_available_semaphore,
                vk::Fence::null(),
            )?
        };

        let wait_semaphores = [frame_sync.image_available_semaphore];
        let wait_stages = [vk::PipelineStageFlags::COLOR_ATTACHMENT_OUTPUT];
        let command_buffers = [self.command_buffers[index as usize]];
        let render_finished_signal_semaphores =
            [frame_sync.render_finished_semaphore];
        let submit_info = [vk::SubmitInfo::builder()
            .wait_semaphores(&wait_semaphores)
            .wait_dst_stage_mask(&wait_stages)
            .command_buffers(&command_buffers)
            .signal_semaphores(&render_finished_signal_semaphores)
            .build()];

        let graphics_queue = self.device.graphics_queue.acquire();

        unsafe {
            self.device.logical_device.queue_submit(
                *graphics_queue,
                &submit_info,
                vk::Fence::null(),
            )?;
        }

        let swapchains = [self.swapchain.swapchain];
        let indices = [index];
        let present_info = vk::PresentInfoKHR::builder()
            .wait_semaphores(&render_finished_signal_semaphores)
            .swapchains(&swapchains)
            .image_indices(&indices);

        let present_queue = self.device.present_queue.acquire();

        let _ = unsafe {
            self.swapchain
                .swapchain_loader
                .queue_present(*present_queue, &present_info)?
        };

        // unsafe {
        //     self.device
        //         .logical_device
        //         .queue_wait_idle(*graphics_queue)?;
        //     self.device.logical_device.queue_wait_idle(*present_queue)?;
        // }

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

impl Drop for Frame {
    fn drop(&mut self) {
        unsafe {
            // don't delete anything until the GPU has stoped using our
            // resources
            self.device
                .logical_device
                .device_wait_idle()
                .expect("wait for device to idle");

            let device = &self.device;
            self.frames_in_flight
                .drain(..)
                .for_each(|frame| frame.destroy(device));

            // safe to delete now
            self.device
                .logical_device
                .free_command_buffers(self.command_pool, &self.command_buffers);
            self.device
                .logical_device
                .destroy_command_pool(self.command_pool, None);
        }
    }
}

/// Create the command buffer pool.
///
/// The caller is responsible for destroying the pool before the device.
fn create_command_pool(device: &Device) -> Result<vk::CommandPool> {
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
        "Graphics Command Pool",
        vk::ObjectType::COMMAND_POOL,
        &command_pool,
    )?;
    Ok(command_pool)
}

/// Create one command buffer for each frame.
///
/// The caller is responsible for deallocating the command buffers when done
/// using them.
fn create_command_buffers(
    device: &Device,
    swapchain: &Swapchain,
    command_pool: &vk::CommandPool,
) -> Result<Vec<vk::CommandBuffer>> {
    let create_info = vk::CommandBufferAllocateInfo::builder()
        .command_pool(*command_pool)
        .level(vk::CommandBufferLevel::PRIMARY)
        .command_buffer_count(swapchain.framebuffers.len() as u32);
    let command_buffers = unsafe {
        device
            .logical_device
            .allocate_command_buffers(&create_info)?
    };

    Ok(command_buffers)
}
