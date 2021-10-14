use super::{FinishFrame, FramebufferRenderPass, RenderPassArgs, Renderer};

use crate::{
    vulkan::{errors::VulkanError, CommandBuffer, RenderDevice, VulkanDebug},
    vulkan_ext::CommandBufferExt,
};

use ::{anyhow::Result, ash::vk, std::sync::Arc};

const NAME: &'static str = "FinishFrame";

impl FinishFrame {
    /// Create a new render pass which clears the framebuffer to a fixed color
    /// and prepares the frame for subsequent render passes.
    pub fn new(vk_dev: Arc<RenderDevice>) -> Result<Self, VulkanError> {
        let args = RenderPassArgs {
            last: true,
            ..Default::default()
        };
        let fbrp = FramebufferRenderPass::new(vk_dev, args)?;
        fbrp.set_debug_name(NAME)?;
        Ok(Self { fbrp })
    }

    pub unsafe fn rebuild_swapchain_resources(
        &mut self,
    ) -> Result<(), VulkanError> {
        self.fbrp.rebuild_swapchain_resources()?;
        self.fbrp.set_debug_name(NAME)?;
        Ok(())
    }
}

impl Renderer for FinishFrame {
    /// Fill a command buffer with render commands.
    fn fill_command_buffer(
        &self,
        cmd: &CommandBuffer,
        current_image: usize,
    ) -> Result<()> {
        unsafe {
            self.fbrp.begin_framebuffer_renderpass(
                cmd,
                current_image,
                vk::SubpassContents::INLINE,
            );
            cmd.end_renderpass();
        }
        Ok(())
    }
}