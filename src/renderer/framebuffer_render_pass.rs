use super::{FramebufferRenderPass, RenderPassArgs};

use crate::vulkan::{
    errors::{VulkanDebugError, VulkanError},
    CommandBuffer, Framebuffer, RenderDevice, VulkanDebug,
};

use ::{
    ash::{version::DeviceV1_0, vk},
    std::sync::Arc,
};

impl FramebufferRenderPass {
    /// Create a new render pass wrapper.
    pub fn new(
        vk_dev: Arc<RenderDevice>,
        args: RenderPassArgs,
    ) -> Result<Self, VulkanError> {
        let render_pass = args.create_render_pass(vk_dev.clone())?;
        let framebuffers = Framebuffer::with_swapchain_color_attachments(
            vk_dev.clone(),
            render_pass.raw,
            "Framebuffer",
        )?;
        Ok(Self {
            args,
            render_pass,
            framebuffers,
            framebuffer_extent: vk_dev
                .with_swapchain(|swapchain| swapchain.extent),
            vk_dev,
        })
    }

    /// Called when the swapchain has been rebuilt.
    pub unsafe fn rebuild_swapchain_resources(
        &mut self,
    ) -> Result<(), VulkanError> {
        self.render_pass = self
            .args
            .create_render_pass(self.render_pass.vk_dev.clone())?;
        self.framebuffers = Framebuffer::with_swapchain_color_attachments(
            self.vk_dev.clone(),
            self.render_pass.raw,
            "Framebuffer",
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
