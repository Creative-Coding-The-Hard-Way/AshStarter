use ::{ash::vk, std::sync::Arc};

use crate::{
    renderer::RenderPassArgs,
    vulkan::{
        errors::{VulkanDebugError, VulkanError},
        CommandBuffer, Framebuffer, ImageView, RenderDevice, RenderPass,
        VulkanDebug,
    },
};

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

    /// The MSAA color render target
    pub msaa_render_target: Arc<ImageView>,

    /// The device used to create this instance
    pub vk_dev: Arc<RenderDevice>,
}

impl FramebufferRenderPass {
    /// Create a new render pass wrapper.
    pub fn new(
        vk_dev: Arc<RenderDevice>,
        args: RenderPassArgs,
        msaa_render_target: Arc<ImageView>,
    ) -> Result<Self, VulkanError> {
        let render_pass = args.create_render_pass(vk_dev.clone())?;
        let framebuffers = args.create_framebuffers(
            vk_dev.clone(),
            &render_pass,
            &msaa_render_target,
        )?;
        Ok(Self {
            args,
            render_pass,
            framebuffers,
            framebuffer_extent: vk_dev
                .with_swapchain(|swapchain| swapchain.extent),
            msaa_render_target,
            vk_dev,
        })
    }

    /// Called when the swapchain has been rebuilt.
    pub unsafe fn rebuild_swapchain_resources(
        &mut self,
        msaa_render_target: Arc<ImageView>,
    ) -> Result<(), VulkanError> {
        self.msaa_render_target = msaa_render_target;
        self.render_pass = self
            .args
            .create_render_pass(self.render_pass.vk_dev.clone())?;
        self.framebuffers = self.args.create_framebuffers(
            self.vk_dev.clone(),
            &self.render_pass,
            &self.msaa_render_target,
        )?;
        self.framebuffer_extent =
            self.vk_dev.with_swapchain(|swapchain| swapchain.extent);
        Ok(())
    }

    /// Begin the render pass for the current frame
    pub unsafe fn begin_framebuffer_renderpass(
        &self,
        command_buffer: &CommandBuffer,
        current_image: usize,
        subpass_contents: vk::SubpassContents,
    ) -> &Self {
        let render_pass_begin_info = vk::RenderPassBeginInfo {
            render_pass: self.render_pass.raw,
            framebuffer: self.framebuffers[current_image].raw,
            render_area: vk::Rect2D {
                offset: vk::Offset2D { x: 0, y: 0 },
                extent: self.framebuffer_extent,
            },
            clear_value_count: self.args.clear_count(),
            p_clear_values: self.args.clear_colors_ptr(),
            ..Default::default()
        };
        self.vk_dev.logical_device.cmd_begin_render_pass(
            command_buffer.raw,
            &render_pass_begin_info,
            subpass_contents,
        );
        &self
    }
}

impl VulkanDebug for FramebufferRenderPass {
    fn set_debug_name(
        &self,
        debug_name: impl Into<String>,
    ) -> Result<(), VulkanDebugError> {
        let name = debug_name.into();
        self.render_pass
            .set_debug_name(format!("{} RenderPass", name))?;
        for (i, framebuffer) in self.framebuffers.iter().enumerate() {
            framebuffer
                .set_debug_name(format!("{} Framebuffer {}", name, i))?;
        }
        Ok(())
    }
}
