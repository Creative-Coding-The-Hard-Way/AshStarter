mod graphics_pipeline;
mod vertex;

use self::graphics_pipeline::GraphicsPipeline;
pub use self::vertex::Vertex;
use crate::{
    application::render_context::{Frame, RenderTarget},
    rendering::{Device, Swapchain},
};

use anyhow::Result;
use ash::{version::DeviceV1_0, vk};
use std::sync::Arc;

/// Resources used to render a single triangle to a frame.
pub struct Triangle {
    pub vertices: Vec<Vertex>,
    graphics_pipeline: Arc<GraphicsPipeline>,
    swapchain: Arc<Swapchain>,
    device: Arc<Device>,
}

impl RenderTarget for Triangle {
    /// Render the triangle to a single frame.
    fn render_to_frame(
        &mut self,
        image_available: vk::Semaphore,
        frame: &mut Frame,
    ) -> Result<vk::Semaphore> {
        // It's safe to write to the buffer because this method will not be
        // called until the previous rendering commands for this frame have
        // finished
        let vertex_buffer_handle = unsafe {
            let buffer = frame.request_buffer();
            buffer.write_data(&self.vertices)?;
            buffer.raw_buffer()
        };

        let command_buffer = frame.request_command_buffer()?;

        self.record_buffer_commands(
            &frame.framebuffer,
            &command_buffer,
            vertex_buffer_handle,
        )?;

        frame.submit_command_buffers(image_available, &[command_buffer])
    }
}

impl Triangle {
    /// Create a new Triangle subsystem which knows how to render itself to a
    /// single frame.
    pub fn new(device: Arc<Device>, swapchain: Arc<Swapchain>) -> Result<Self> {
        let graphics_pipeline = GraphicsPipeline::new(&device, &swapchain)?;

        Ok(Self {
            vertices: vec![],
            graphics_pipeline,
            swapchain,
            device,
        })
    }

    /// Replace the swapchain and all dependent resources in the Triangle
    /// subsystem.
    pub fn replace_swapchain(
        &mut self,
        swapchain: Arc<Swapchain>,
    ) -> Result<()> {
        self.swapchain = swapchain;
        self.graphics_pipeline =
            GraphicsPipeline::new(&self.device, &self.swapchain)?;
        Ok(())
    }

    fn record_buffer_commands(
        &self,
        framebuffer: &vk::Framebuffer,
        command_buffer: &vk::CommandBuffer,
        vertex_buffer: vk::Buffer,
    ) -> Result<()> {
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

            let buffers = [vertex_buffer];
            let offsets = [0];
            self.device.logical_device.cmd_bind_vertex_buffers(
                *command_buffer,
                0,
                &buffers,
                &offsets,
            );

            // draw
            self.device.logical_device.cmd_draw(
                *command_buffer,
                self.vertices.len() as u32, // vertex count
                1,                          // instance count
                0,                          // first vertex
                0,                          // first instance
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

        Ok(())
    }
}

impl Drop for Triangle {
    fn drop(&mut self) {
        unsafe {
            self.device.logical_device.device_wait_idle().expect(
                "error while waiting for the device to complete all work",
            );
        }
    }
}
