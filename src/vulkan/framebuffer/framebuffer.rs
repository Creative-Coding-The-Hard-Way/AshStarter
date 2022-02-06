use ::{
    ash::{version::DeviceV1_0, vk},
    std::sync::Arc,
};

use super::{Framebuffer, FramebufferError};
use crate::vulkan::{errors::VulkanDebugError, RenderDevice, VulkanDebug};

impl Framebuffer {
    /// Construct new framebuffers with color attachments for each of the
    /// swapchain's image views.
    pub fn with_swapchain_color_attachments(
        vk_dev: Arc<RenderDevice>,
        render_pass: vk::RenderPass,
        debug_name: impl Into<String>,
    ) -> Result<Vec<Self>, FramebufferError> {
        let name = debug_name.into();
        vk_dev.with_swapchain(
            |swapchain| -> Result<Vec<Self>, FramebufferError> {
                let mut framebuffers = vec![];
                for i in 0..swapchain.image_views.len() {
                    let framebuffer = Self::with_color_attachments(
                        vk_dev.clone(),
                        render_pass,
                        &[swapchain.image_views[i]],
                        swapchain.extent,
                    )?;
                    framebuffer.set_debug_name(format!("{} - {}", name, i))?;
                    framebuffers.push(framebuffer);
                }
                Ok(framebuffers)
            },
        )
    }

    /// Create a single framebuffer with a color attachment.
    pub fn with_color_attachments(
        vk_dev: Arc<RenderDevice>,
        render_pass: vk::RenderPass,
        images: &[vk::ImageView],
        extent: vk::Extent2D,
    ) -> Result<Self, FramebufferError> {
        let create_info = vk::FramebufferCreateInfo {
            flags: vk::FramebufferCreateFlags::empty(),
            render_pass,
            attachment_count: images.len() as u32,
            p_attachments: images.as_ptr(),
            width: extent.width,
            height: extent.height,
            layers: 1,
            ..Default::default()
        };
        let raw = unsafe {
            vk_dev
                .logical_device
                .create_framebuffer(&create_info, None)
                .map_err(FramebufferError::UnableToCreateFramebuffer)?
        };
        Ok(Self { raw, vk_dev })
    }
}

impl VulkanDebug for Framebuffer {
    fn set_debug_name(
        &self,
        debug_name: impl Into<String>,
    ) -> Result<(), VulkanDebugError> {
        self.vk_dev.name_vulkan_object(
            debug_name,
            vk::ObjectType::FRAMEBUFFER,
            self.raw,
        )?;
        Ok(())
    }
}

impl Drop for Framebuffer {
    /// # DANGER
    ///
    /// There is no internal synchronization for this type. Unexpected behavior
    /// can occur if this instance is still in-use by the GPU when it is
    /// dropped.
    fn drop(&mut self) {
        unsafe {
            self.vk_dev
                .logical_device
                .destroy_framebuffer(self.raw, None);
        }
    }
}
