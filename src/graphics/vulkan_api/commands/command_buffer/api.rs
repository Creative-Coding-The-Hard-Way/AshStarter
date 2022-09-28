use ash::vk;

use super::CommandBuffer;
use crate::graphics::vulkan_api::{
    DescriptorSet, Framebuffer, GraphicsPipeline, HostCoherentBuffer,
    PipelineLayout, RenderPass, VulkanError,
};

impl CommandBuffer {
    pub fn begin_one_time_submit(&self) -> Result<(), VulkanError> {
        let begin_info = vk::CommandBufferBeginInfo {
            flags: vk::CommandBufferUsageFlags::ONE_TIME_SUBMIT,
            ..Default::default()
        };
        self.render_device
            .begin_command_buffer(&self.command_buffer, &begin_info)
    }

    pub fn end_command_buffer(&self) -> Result<(), VulkanError> {
        self.render_device.end_command_buffer(&self.command_buffer)
    }

    pub fn end_render_pass(&self) -> &Self {
        self.render_device.cmd_end_render_pass(&self.command_buffer);
        self
    }

    /// # Safety
    ///
    /// Unsafe because the caller must ensure that the render pass and
    /// framebuffer live until the commands have completed executing on
    /// the GPU.
    pub unsafe fn begin_render_pass_inline(
        &self,
        render_pass: &RenderPass,
        framebuffer: &Framebuffer,
        extent: vk::Extent2D,
        clear_color: [f32; 4],
    ) -> &Self {
        let clear_values = [vk::ClearValue {
            color: vk::ClearColorValue {
                float32: clear_color,
            },
        }];
        let begin_info = vk::RenderPassBeginInfo {
            render_pass: render_pass.raw(),
            framebuffer: framebuffer.raw(),
            render_area: vk::Rect2D {
                offset: vk::Offset2D { x: 0, y: 0 },
                extent,
            },
            clear_value_count: 1,
            p_clear_values: clear_values.as_ptr(),
            ..Default::default()
        };
        self.render_device.cmd_begin_render_pass(
            &self.command_buffer,
            &begin_info,
            vk::SubpassContents::INLINE,
        );
        self
    }

    /// Bind a pipeline for rendering.
    ///
    /// # Safety
    ///
    /// Unsafe because:
    ///   - The application must not drop the bound pipeline until this command
    ///     buffer is destroyed or finishes executing.
    pub unsafe fn bind_graphics_pipeline(
        &self,
        graphics_pipeline: &GraphicsPipeline,
    ) -> &Self {
        self.render_device.cmd_bind_pipeline(
            &self.command_buffer,
            vk::PipelineBindPoint::GRAPHICS,
            graphics_pipeline.raw(),
        );
        self
    }

    /// Set a viewport for rendering commands.
    pub fn set_viewport(&self, extent: vk::Extent2D) -> &Self {
        let viewport = vk::Viewport {
            x: 0.0,
            y: 0.0,
            width: extent.width as f32,
            height: extent.height as f32,
            min_depth: 0.0,
            max_depth: 1.0,
        };
        // Safe because only one viewport is set, so there's no need to check
        // for multi-viewport support.
        unsafe {
            self.render_device.cmd_set_viewport(
                &self.command_buffer,
                0,
                &[viewport],
            )
        }
        self
    }

    /// Set a scissor region for rendering commands.
    pub fn set_scissor(&self, x: i32, y: i32, extent: vk::Extent2D) -> &Self {
        let scissor = vk::Rect2D {
            offset: vk::Offset2D { x, y },
            extent,
        };
        // Safe because only one scissor is set, so theres no need to check
        // for multi-viewport support.
        unsafe {
            self.render_device.cmd_set_scissor(
                &self.command_buffer,
                0,
                &[scissor],
            );
        }
        self
    }

    /// Add an un-instanced draw command to the command buffer.
    ///
    /// # Safety
    ///
    /// Unsafe because:
    ///   - The application must ensure that the required vertex buffers and
    ///     pipelines and descriptors are all set prior to issuing this command.
    pub unsafe fn draw(&self, vertex_count: u32, first_vertex: u32) -> &Self {
        self.render_device.cmd_draw(
            &self.command_buffer,
            vertex_count, // vertex count
            1,            // instance count
            first_vertex, // first vertex
            0,            // first instance
        );
        self
    }

    /// Bind a vertex buffer for drawing operations.
    ///
    /// # Safety
    ///
    /// Unsafe because:
    ///   - The application must ensure that the vertex buffer lives until the
    ///     commands have finished executing.
    pub unsafe fn bind_vertex_buffer<T>(
        &self,
        buffer: &HostCoherentBuffer<T>,
        offset: u64,
    ) -> &Self {
        self.render_device.cmd_bind_vertex_buffers(
            &self.command_buffer,
            0,
            &[*buffer.raw()],
            &[offset],
        );
        self
    }

    /// Bind descriptor sets for a pipeline.
    ///
    /// # Safety
    ///
    /// Unsafe because:
    ///   - Descriptor sets cannot typically be written while bound.
    ///   - The application must keep all bound resources alive until the
    ///     commands in this buffer finishexecuting.
    pub unsafe fn bind_graphics_descriptor_sets(
        &self,
        pipeline_layout: &PipelineLayout,
        descriptor_sets: &[&DescriptorSet],
    ) -> &Self {
        let raw_descriptor_sets: Vec<vk::DescriptorSet> = descriptor_sets
            .iter()
            .map(|descriptor_set| *descriptor_set.raw())
            .collect();
        self.render_device.cmd_bind_descriptor_sets(
            &self.command_buffer,
            vk::PipelineBindPoint::GRAPHICS,
            pipeline_layout.raw(),
            0,
            &raw_descriptor_sets,
            &[],
        );
        self
    }
}
