use super::{FinishFrame, RenderPass, RenderPassArgs, Renderer};

use crate::vulkan::{errors::VulkanError, RenderDevice};

use ::{anyhow::Result, ash::vk};

const NAME: &'static str = "FinishFrame Renderer";

impl FinishFrame {
    /// Create a new render pass which transitions the framebuffer for
    /// presentation.
    pub fn new(vk_dev: &RenderDevice) -> Result<Self, VulkanError> {
        Ok(Self {
            render_pass: RenderPass::new(vk_dev, NAME, FinishFrame::args())?,
        })
    }

    /// Destroy the renderer's Vulkan resources.
    pub unsafe fn destroy(&mut self, vk_dev: &RenderDevice) {
        self.render_pass.destroy(vk_dev);
    }

    fn args() -> RenderPassArgs {
        RenderPassArgs {
            last: true,
            clear_colors: None,
            ..Default::default()
        }
    }
}

impl Renderer for FinishFrame {
    /// Fill a command buffer with render commands.
    fn fill_command_buffer(
        &self,
        vk_dev: &RenderDevice,
        cmd: vk::CommandBuffer,
        current_image: u32,
    ) -> Result<()> {
        self.render_pass
            .begin_render_pass(vk_dev, cmd, current_image);
        self.render_pass.end_render_pass(vk_dev, cmd);
        Ok(())
    }

    unsafe fn rebuild_swapchain_resources(
        &mut self,
        vk_dev: &RenderDevice,
    ) -> anyhow::Result<()> {
        self.destroy(vk_dev);
        self.render_pass = RenderPass::new(vk_dev, NAME, FinishFrame::args())?;
        Ok(())
    }
}
