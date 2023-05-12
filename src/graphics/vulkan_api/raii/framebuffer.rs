use {
    crate::graphics::{vulkan_api::RenderDevice, GraphicsError},
    ash::vk,
    std::sync::Arc,
};

pub struct Framebuffer {
    framebuffer: vk::Framebuffer,
    render_device: Arc<RenderDevice>,
}

impl Framebuffer {
    /// Create a new Vulkan Framebuffer which is automatically destroyed when
    /// dropped.
    ///
    /// # Safety
    ///
    /// Unsafe because:
    ///   - The application must not drop the Framebuffer while it is in use by
    ///     the GPU.
    pub fn new(
        render_device: Arc<RenderDevice>,
        create_info: &vk::FramebufferCreateInfo,
    ) -> Result<Self, GraphicsError> {
        let framebuffer = unsafe {
            render_device
                .device()
                .create_framebuffer(create_info, None)?
        };
        Ok(Self {
            framebuffer,
            render_device,
        })
    }

    /// Set the debug name for how this resource appears in Vulkan logs.
    pub fn set_debug_name(&self, name: impl Into<String>) {
        self.render_device.set_debug_name(
            self.raw(),
            vk::ObjectType::FRAMEBUFFER,
            name,
        )
    }

    /// Get the raw Vulkan ImageView handle.
    pub fn raw(&self) -> vk::Framebuffer {
        self.framebuffer
    }
}

impl Drop for Framebuffer {
    fn drop(&mut self) {
        unsafe {
            self.render_device
                .device()
                .destroy_framebuffer(self.framebuffer, None);
        }
    }
}

impl std::fmt::Debug for Framebuffer {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Framebuffer")
            .field("framebuffer", &self.framebuffer)
            .finish()
    }
}
