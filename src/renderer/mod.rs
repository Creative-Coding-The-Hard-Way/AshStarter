mod clear_frame;
mod finish_frame;
mod framebuffer_render_pass;
mod render_pass_args;
mod triangle_canvas;

use ::anyhow::Result;

pub use self::{
    clear_frame::ClearFrame,
    finish_frame::FinishFrame,
    framebuffer_render_pass::FramebufferRenderPass,
    render_pass_args::RenderPassArgs,
    triangle_canvas::{TriangleCanvas, Vertex2D},
};
use crate::vulkan::CommandBuffer;

pub trait Renderer {
    /// Fill the frame's command buffer.
    ///
    /// The `current_image` parameter is the index of the swapchain image
    /// currently being targeted.
    ///
    fn fill_command_buffer(
        &self,
        command_buffer: &CommandBuffer,
        current_image: usize,
    ) -> Result<()>;
}
