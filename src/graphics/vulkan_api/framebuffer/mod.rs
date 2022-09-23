use std::sync::Arc;

use ash::vk;

use crate::graphics::vulkan_api::{
    ImageView, RenderDevice, RenderPass, VulkanDebug, VulkanError,
};

pub struct Framebuffer {
    _image_views: Vec<Arc<ImageView>>,
    framebuffer: vk::Framebuffer,
    render_device: Arc<RenderDevice>,
}

impl Framebuffer {
    pub fn new(
        render_device: Arc<RenderDevice>,
        render_pass: &RenderPass,
        image_views: &[Arc<ImageView>],
        extent: vk::Extent2D,
    ) -> Result<Self, VulkanError> {
        // it's safe to take the render pass's handle because the reference is
        // only held until the framebuffer is created.
        let raw_render_pass = unsafe { render_pass.raw() };

        let images_handles: Vec<vk::ImageView> = image_views
            .iter()
            .map(|image| unsafe { image.raw() })
            .collect();

        let create_info = vk::FramebufferCreateInfo {
            flags: vk::FramebufferCreateFlags::empty(),
            render_pass: raw_render_pass,
            attachment_count: images_handles.len() as u32,
            p_attachments: images_handles.as_ptr(),
            width: extent.width,
            height: extent.height,
            layers: 1,
            ..Default::default()
        };
        let framebuffer =
            unsafe { render_device.create_framebuffer(&create_info)? };
        Ok(Self {
            _image_views: image_views.to_owned(),
            framebuffer,
            render_device,
        })
    }

    /// Get the underlying Vulkan framebuffer handle.
    ///
    /// # Safety
    ///
    /// Unsafe because ownership is not transferred. The caller is responsible
    /// for ensuring no references to the framebuffer occur after the owning
    /// object is dropped.
    pub unsafe fn raw(&self) -> vk::Framebuffer {
        self.framebuffer
    }
}

impl VulkanDebug for Framebuffer {
    fn set_debug_name(&self, debug_name: impl Into<String>) {
        self.render_device.name_vulkan_object(
            debug_name,
            vk::ObjectType::FRAMEBUFFER,
            self.framebuffer,
        );
    }
}

impl Drop for Framebuffer {
    /// # Safety
    ///
    /// The application is responsible for ensuring that no GPU operation
    /// depend on this frame buffer when it's dropped.
    fn drop(&mut self) {
        unsafe {
            self.render_device.destroy_framebuffer(self.framebuffer);
        }
    }
}
