mod graphics_pipeline;
mod vertex;

use self::graphics_pipeline::GraphicsPipeline;
pub use self::vertex::Vertex;
use crate::{
    application::render_context::{Frame, RenderTarget},
    rendering::{Device, Swapchain},
};

use anyhow::{Context, Result};
use ash::{version::DeviceV1_0, version::InstanceV1_0, vk};
use std::sync::Arc;

/// Resources used to render a single triangle to a frame.
pub struct Triangle {
    graphics_pipeline: Arc<GraphicsPipeline>,
    swapchain: Arc<Swapchain>,
    device: Arc<Device>,

    vertex_buffer: vk::Buffer,
    vertex_buffer_memory: vk::DeviceMemory,
}

impl RenderTarget for Triangle {
    /// Render the triangle to a single frame.
    fn render_to_frame(
        &mut self,
        image_available: vk::Semaphore,
        frame: &mut Frame,
    ) -> Result<vk::Semaphore> {
        let command_buffer = frame.request_command_buffer()?;

        self.record_buffer_commands(&frame.framebuffer, &command_buffer)?;

        frame.submit_command_buffers(image_available, &[command_buffer])
    }
}

impl Triangle {
    /// Create a new Triangle subsystem which knows how to render itself to a
    /// single frame.
    pub fn new(device: Arc<Device>, swapchain: Arc<Swapchain>) -> Result<Self> {
        let graphics_pipeline = GraphicsPipeline::new(&device, &swapchain)?;

        let create_info = vk::BufferCreateInfo::builder()
            .size((std::mem::size_of::<Vertex>() * 3) as u64)
            .usage(vk::BufferUsageFlags::VERTEX_BUFFER)
            .sharing_mode(vk::SharingMode::EXCLUSIVE);

        let vertex_buffer =
            unsafe { device.logical_device.create_buffer(&create_info, None)? };

        let requirements = unsafe {
            device
                .logical_device
                .get_buffer_memory_requirements(vertex_buffer)
        };

        let memory_properties = unsafe {
            device
                .instance
                .ash
                .get_physical_device_memory_properties(device.physical_device)
        };

        let memory_type_index = memory_properties
            .memory_types
            .iter()
            .enumerate()
            .find(|(i, memory_type)| {
                let type_supported =
                    requirements.memory_type_bits & (1 << i) != 0;
                let properties_supported = memory_type.property_flags.contains(
                    vk::MemoryPropertyFlags::HOST_VISIBLE
                        | vk::MemoryPropertyFlags::HOST_COHERENT,
                );
                type_supported & properties_supported
            })
            .map(|(i, _memory_type)| i as u32)
            .context(
                "unable to find a suitable memory type for the vertex buffer!",
            )?;
        let allocate_info = vk::MemoryAllocateInfo::builder()
            .allocation_size(requirements.size)
            .memory_type_index(memory_type_index);

        let vertex_buffer_memory = unsafe {
            device
                .logical_device
                .allocate_memory(&allocate_info, None)?
        };

        let vertices = [
            Vertex::new([0.0, -0.99], [1.0, 0.0, 0.0, 1.0]),
            Vertex::new([0.5, 0.5], [0.0, 1.0, 0.0, 1.0]),
            Vertex::new([-0.5, 0.5], [0.0, 0.0, 1.0, 1.0]),
        ];
        unsafe {
            device.logical_device.bind_buffer_memory(
                vertex_buffer,
                vertex_buffer_memory,
                0,
            )?;

            let ptr = device.logical_device.map_memory(
                vertex_buffer_memory,
                0,
                requirements.size,
                vk::MemoryMapFlags::empty(),
            )?;

            std::ptr::copy_nonoverlapping(
                vertices.as_ptr(),
                ptr as *mut Vertex,
                3,
            );

            device.logical_device.unmap_memory(vertex_buffer_memory);
        }

        Ok(Self {
            graphics_pipeline,
            swapchain,
            device,
            vertex_buffer,
            vertex_buffer_memory,
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

            let buffers = [self.vertex_buffer];
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

        Ok(())
    }
}

impl Drop for Triangle {
    fn drop(&mut self) {
        unsafe {
            self.device.logical_device.device_wait_idle().expect(
                "error while waiting for the device to complete all work",
            );
            self.device
                .logical_device
                .destroy_buffer(self.vertex_buffer, None);
            self.device
                .logical_device
                .free_memory(self.vertex_buffer_memory, None);
        }
    }
}
