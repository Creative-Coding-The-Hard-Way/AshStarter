use super::{ClearFrame, RenderPass, RenderPassArgs, Renderer};

use crate::vulkan::{errors::VulkanError, RenderDevice};

use ::{anyhow::Result, ash::vk};

const NAME: &'static str = "ClearFrame Renderer";

impl ClearFrame {
    /// Create a new render pass which clears the framebuffer to a fixed color
    /// and prepares the frame for subsequent render passes.
    pub fn new(
        vk_dev: &RenderDevice,
        clear_color: [f32; 4],
    ) -> Result<Self, VulkanError> {
        Ok(Self {
            clear_color,
            render_pass: RenderPass::new(
                vk_dev,
                NAME,
                ClearFrame::args(clear_color),
            )?,
        })
    }

    /// Destroy the renderer's Vulkan resources.
    pub unsafe fn destroy(&mut self, vk_dev: &RenderDevice) {
        self.render_pass.destroy(vk_dev);
    }

    fn args(clear_color: [f32; 4]) -> RenderPassArgs {
        RenderPassArgs {
            first: true,
            clear_colors: Some(vec![vk::ClearValue {
                color: vk::ClearColorValue {
                    float32: clear_color,
                },
            }]),
            ..Default::default()
        }
    }
}

impl Renderer for ClearFrame {
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
        self.render_pass =
            RenderPass::new(vk_dev, NAME, ClearFrame::args(self.clear_color))?;
        Ok(())
    }
}
