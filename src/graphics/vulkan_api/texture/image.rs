use {
    crate::graphics::vulkan_api::{
        Allocation, RenderDevice, VulkanDebug, VulkanError,
    },
    ash::vk,
    std::sync::Arc,
};

/// An owned Vulkan image.
pub struct Image {
    allocation: Allocation,
    image: vk::Image,
    render_device: Arc<RenderDevice>,
}

impl Image {
    pub fn new(
        render_device: Arc<RenderDevice>,
        create_info: &vk::ImageCreateInfo,
    ) -> Result<Self, VulkanError> {
        let (image, allocation) = unsafe {
            // safe because the image is destroyed when this instance is dropped
            // and this instance keeps an arc of the render device
            let image = render_device.create_image(create_info)?;
            let allocation = render_device.allocate_memory(
                render_device.get_image_memory_requirements(image),
                vk::MemoryPropertyFlags::DEVICE_LOCAL,
            )?;
            render_device.bind_image_memory(&image, &allocation)?;
            (image, allocation)
        };
        Ok(Self {
            allocation,
            image,
            render_device,
        })
    }

    /// The raw Vulkan image handle.
    ///
    /// # Safety
    ///
    /// Unsafe because:
    ///   - Ownership is not transferred.
    pub unsafe fn raw(&self) -> &vk::Image {
        &self.image
    }
}

impl VulkanDebug for Image {
    fn set_debug_name(&self, debug_name: impl Into<String>) {
        self.render_device.name_vulkan_object(
            debug_name,
            vk::ObjectType::IMAGE,
            self.image,
        )
    }
}

impl Drop for Image {
    fn drop(&mut self) {
        unsafe {
            self.render_device.destroy_image(self.image);
            self.render_device
                .free_memory(&self.allocation)
                .expect("Unable to free image memory");
        }
    }
}
