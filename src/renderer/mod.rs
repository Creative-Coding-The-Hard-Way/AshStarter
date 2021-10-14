mod clear_frame;
mod finish_frame;
mod framebuffer_render_pass;
mod render_pass_args;
//mod triangle_canvas;

use crate::vulkan::{CommandBuffer, Framebuffer, RenderDevice, RenderPass};

use ::{anyhow::Result, ash::vk, std::sync::Arc};

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

/// A renderer which transitions the image for rendering and clears to a known
/// value.
pub struct ClearFrame {
    fbrp: FramebufferRenderPass,
}

/// A renderer which transitions the image for presentation, effectively
/// finishing the frame.
pub struct FinishFrame {
    fbrp: FramebufferRenderPass,
}

///// A renderer which just draws triangles on the screen.
//pub struct TriangleCanvas {
//    vertex_data: Vec<Buffer>,
//    descriptor_sets: Vec<vk::DescriptorSet>,
//
//    render_pass: RenderPass,
//    pipeline: vk::Pipeline,
//    pipeline_layout: vk::PipelineLayout,
//    descriptor_layout: vk::DescriptorSetLayout,
//    descriptor_pool: vk::DescriptorPool,
//}

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

/// A combination of a Vulkan RenderPass object and a set of framebuffers with
/// a 1-1 mapping to swapchain images.
pub struct FramebufferRenderPass {
    /// Renderpass args control the creation of the underlying Vulkan renderpass
    /// instance.
    pub args: RenderPassArgs,

    /// Framebuffers for each respective swapchain image
    pub framebuffers: Vec<Framebuffer>,

    /// The renderpass created based on the renderpass args
    pub render_pass: RenderPass,

    /// The full size of each framebuffer
    pub framebuffer_extent: vk::Extent2D,

    /// The device used to create this instance
    pub vk_dev: Arc<RenderDevice>,
}
