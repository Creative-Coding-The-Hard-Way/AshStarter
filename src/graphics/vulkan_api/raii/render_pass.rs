use {
    crate::graphics::{vulkan_api::RenderDevice, GraphicsError},
    ash::vk,
    std::sync::Arc,
};

/// A RAII Vulkan RenderPass which is destroyed when dropped.
pub struct RenderPass {
    render_pass: vk::RenderPass,
    render_device: Arc<RenderDevice>,
}

impl RenderPass {
    /// Create a new Vulkan RenderPass which is automatically destroyed when
    /// dropped.
    ///
    /// # Safety
    ///
    /// Unsafe because:
    ///   - The application must not drop the RenderPass while it is in use by
    ///     the GPU.
    pub fn new(
        render_device: Arc<RenderDevice>,
        create_info: &vk::RenderPassCreateInfo,
    ) -> Result<Self, GraphicsError> {
        let render_pass = unsafe {
            render_device
                .device()
                .create_render_pass(create_info, None)?
        };
        Ok(Self {
            render_pass,
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
    pub fn raw(&self) -> vk::RenderPass {
        self.render_pass
    }
}

impl Drop for RenderPass {
    fn drop(&mut self) {
        unsafe {
            self.render_device
                .device()
                .destroy_render_pass(self.render_pass, None);
        }
    }
}

impl std::fmt::Debug for RenderPass {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("RenderPass")
            .field("render_pass", &self.render_pass)
            .finish()
    }
}
