use ash::vk;

use super::CommandBuffer;
use crate::graphics::vulkan_api::{Framebuffer, RenderPass, VulkanError};

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

    pub fn end_render_pass(&self) {
        self.render_device.cmd_end_render_pass(&self.command_buffer)
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
    ) {
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
        )
    }
}
