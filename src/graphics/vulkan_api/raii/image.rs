use {
    crate::graphics::{vulkan_api::RenderDevice, GraphicsError},
    ash::vk,
    ccthw_ash_allocator::Allocation,
    std::sync::Arc,
};

/// RAII Vulkan Image.
pub struct Image {
    image: vk::Image,
    allocation: Allocation,
    render_device: Arc<RenderDevice>,
}

impl Image {
    /// Create a new Vulkan descriptor pool.
    ///
    /// # Safety
    ///
    /// Unsafe because:
    ///   - command pools must be destroyed before the Vulkan device is dropped.
    pub unsafe fn new(
        render_device: Arc<RenderDevice>,
        create_info: &vk::ImageCreateInfo,
        memory_property_flags: vk::MemoryPropertyFlags,
    ) -> Result<Self, GraphicsError> {
        let (image, allocation) = unsafe {
            render_device
                .memory()
                .allocate_image(create_info, memory_property_flags)?
        };
        Ok(Self {
            image,
            allocation,
            render_device,
        })
    }

    /// Set the name which shows up in Vulkan debug logs for this resource.
    pub fn set_debug_name(&self, name: impl Into<String>) {
        self.render_device.set_debug_name(
            self.image,
            vk::ObjectType::IMAGE,
            name,
        );
    }

    /// Get the backing memory allocation for the Image.
    pub fn allocation(&self) -> &Allocation {
        &self.allocation
    }

    /// Get the raw Vulkan command pool handle.
    pub fn raw(&self) -> vk::Image {
        self.image
    }
}

impl Drop for Image {
    fn drop(&mut self) {
        unsafe {
            self.render_device
                .memory()
                .free_image(self.image, self.allocation.clone());
        }
    }
}

impl std::fmt::Debug for Image {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Image")
            .field("image", &self.image)
            .field("allocation", &self.allocation)
            .finish()
    }
}
