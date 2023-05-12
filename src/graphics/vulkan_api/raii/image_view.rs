use {
    crate::graphics::{vulkan_api::RenderDevice, GraphicsError},
    ash::vk,
    std::sync::Arc,
};

/// A RAII Vulkan Image View.
pub struct ImageView {
    image_view: vk::ImageView,
    render_device: Arc<RenderDevice>,
}

impl ImageView {
    /// Create a new owned Image View which is destroyed when dropped.
    ///
    /// # Safety
    ///
    /// Unsafe because:
    ///   - The application must track and manage lifetimes for any resources
    ///     which this image view depends on. Namely, if the Image View is for a
    ///     given Vulkan Image, that Image must outlive the Image View.
    pub unsafe fn new(
        render_device: Arc<RenderDevice>,
        create_info: &vk::ImageViewCreateInfo,
    ) -> Result<Self, GraphicsError> {
        let image_view = unsafe {
            render_device
                .device()
                .create_image_view(create_info, None)?
        };
        Ok(Self {
            image_view,
            render_device,
        })
    }

    /// Set the debug name for how this resource appears in Vulkan logs.
    pub fn set_debug_name(&self, name: impl Into<String>) {
        self.render_device.set_debug_name(
            self.raw(),
            vk::ObjectType::IMAGE_VIEW,
            name,
        )
    }

    /// Get the raw Vulkan ImageView handle.
    pub fn raw(&self) -> vk::ImageView {
        self.image_view
    }
}

impl Drop for ImageView {
    fn drop(&mut self) {
        unsafe {
            self.render_device
                .device()
                .destroy_image_view(self.image_view, None);
        }
    }
}

impl std::fmt::Debug for ImageView {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ImageView")
            .field("image_view", &self.image_view)
            .finish()
    }
}
