use super::{RenderDevice, RenderDeviceError};

use ash::{version::DeviceV1_0, vk};

impl RenderDevice {
    /// Create framebuffers for each of the swapchain image views.
    pub fn create_framebuffers<Name>(
        &self,
        render_pass: &vk::RenderPass,
        debug_name: Name,
    ) -> Result<Vec<vk::Framebuffer>, RenderDeviceError>
    where
        Name: Into<String>,
    {
        let name = debug_name.into();
        let count = self.swapchain().image_views.len();
        let mut framebuffers = vec![];
        for i in 0..count {
            let attachments = [self.swapchain().image_views[i]];
            let create_info = vk::FramebufferCreateInfo {
                flags: vk::FramebufferCreateFlags::empty(),
                render_pass: *render_pass,
                attachment_count: 1,
                p_attachments: attachments.as_ptr(),
                width: self.swapchain().extent.width,
                height: self.swapchain().extent.height,
                layers: 1,
                ..Default::default()
            };
            let framebuffer = unsafe {
                self.logical_device
                    .create_framebuffer(&create_info, None)
                    .map_err(|err| {
                        RenderDeviceError::UnableToCreateFramebuffer(i, err)
                    })?
            };
            self.name_vulkan_object(
                format!("{} - {}", name, i),
                (vk::ObjectType::FRAMEBUFFER, framebuffer),
            )?;
            framebuffers.push(framebuffer);
        }
        Ok(framebuffers)
    }
}
