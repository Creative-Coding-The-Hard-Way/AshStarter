use super::{ClearFrame, FramebufferRenderPass, RenderPassArgs, Renderer};

use crate::{
    vulkan::{
        errors::VulkanError, CommandBuffer, ImageView, MemoryAllocator,
        RenderDevice, VulkanDebug,
    },
    vulkan_ext::CommandBufferExt,
};

use ::{anyhow::Result, ash::vk, std::sync::Arc};

const NAME: &'static str = "ClearFrame";

impl ClearFrame {
    /// Create a new render pass which clears the framebuffer to a fixed color
    /// and prepares the frame for subsequent render passes.
    pub fn new(
        vk_dev: Arc<RenderDevice>,
        vk_alloc: Arc<dyn MemoryAllocator>,
        clear_color: [f32; 4],
    ) -> Result<Self, VulkanError> {
        let args = RenderPassArgs {
            first: true,
            last: false,
            clear_colors: Some(vec![vk::ClearValue {
                color: vk::ClearColorValue {
                    float32: clear_color,
                },
            }]),
            samples: vk_dev.get_supported_msaa(vk::SampleCountFlags::TYPE_4),
        };
        let render_target =
            args.create_msaa_render_target(vk_dev.clone(), vk_alloc.clone())?;

        let fbrp =
            FramebufferRenderPass::new(vk_dev.clone(), args, render_target)?;
        fbrp.set_debug_name(NAME)?;
        Ok(Self {
            fbrp,
            vk_alloc,
            vk_dev,
        })
    }

    pub unsafe fn rebuild_swapchain_resources(
        &mut self,
    ) -> Result<(), VulkanError> {
        let render_target = self.fbrp.args.create_msaa_render_target(
            self.vk_dev.clone(),
            self.vk_alloc.clone(),
        )?;
        self.fbrp.rebuild_swapchain_resources(render_target)?;
        self.fbrp.set_debug_name(NAME)?;
        Ok(())
    }

    pub fn color_render_target(&self) -> &Arc<ImageView> {
        &self.fbrp.msaa_render_target
    }
}

impl Renderer for ClearFrame {
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

impl ClearFrame {}
