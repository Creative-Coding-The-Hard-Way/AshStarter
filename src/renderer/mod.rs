mod clear_frame;
mod finish_frame;
mod render_pass;
mod triangle_canvas;

use crate::vulkan::{Buffer, RenderDevice};

use anyhow::Result;
use ash::vk;

pub trait Renderer {
    /// Fill the frame's command buffer.
    ///
    /// The `current_image` parameter is the index of the swapchain image
    /// currently being targeted.
    ///
    fn fill_command_buffer(
        &self,
        vk_dev: &RenderDevice,
        command_buffer: vk::CommandBuffer,
        current_image: usize,
    ) -> Result<()>;
}

/// A renderer which transitions the image for rendering and clears to a known
/// value.
pub struct ClearFrame {
    clear_color: [f32; 4],
    render_pass: RenderPass,
}

/// A renderer which transitions the image for presentation, effectively
/// finishing the frame.
pub struct FinishFrame {
    render_pass: RenderPass,
}

/// A renderer which just draws triangles on the screen.
pub struct TriangleCanvas {
    vertex_data: Vec<Buffer>,
    descriptor_sets: Vec<vk::DescriptorSet>,

    render_pass: RenderPass,
    pipeline: vk::Pipeline,
    pipeline_layout: vk::PipelineLayout,
    descriptor_layout: vk::DescriptorSetLayout,
    descriptor_pool: vk::DescriptorPool,
}

/// Configuration values for a new render pass instance.
#[derive(Clone)]
pub struct RenderPassArgs {
    /// Indicates that the render pass is the first in the frame. Renderpasses
    /// configured this way will expect the image format to be `UNKNOWN`.
    /// When false, the render pass will expect a previous pass in the frame to
    /// have already transitioned the frame to `COLOR_ATTACHMENT_OPTIMAL`.
    first: bool,

    /// Indicates that the render pass is the last in the frame. RenderPasses
    /// configured this way will transition the image format to
    /// `PRESENT_SRC_KHR`. When false (the default), the render pass will
    /// transition the image format to `COLOR_ATTACHMENT_OPTIMAL`.
    last: bool,

    /// Indicates that the render pass should use the provided values to clear
    /// the framebuffer.
    clear_colors: Option<Vec<vk::ClearValue>>,
}

/// A Renderpass is a combination of a Vulkan RenderPass object and a set of
/// framebuffers.
///
/// This combination is a highly common need for all of the Renderers defined
/// in this module.
pub struct RenderPass {
    name: String,
    args: RenderPassArgs,
    framebuffers: Vec<vk::Framebuffer>,
    render_pass: vk::RenderPass,
}
