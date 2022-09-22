use ash::vk;

use super::RenderDevice;
use crate::graphics::vulkan_api::VulkanError;

impl RenderDevice {
    /// Create a raw Vulkan ImageView instance.
    ///
    /// # Safety
    ///
    /// Unsafe because the caller must ensure the ImageView is destroyed before
    /// the RenderDevice is dropped.
    pub unsafe fn create_image_view(
        &self,
        create_info: &vk::ImageViewCreateInfo,
    ) -> Result<vk::ImageView, VulkanError> {
        self.logical_device
            .create_image_view(create_info, None)
            .map_err(VulkanError::UnableToCreateImageView)
    }

    /// Destroy a raw Vulkan ImageView.
    ///
    /// # Safety
    ///
    /// Unsafe because the caller must ensure the ImageView is no longer being
    /// used by any GPU operations at the time of destruction.
    pub unsafe fn destroy_image_view(&self, image_view: vk::ImageView) {
        self.logical_device.destroy_image_view(image_view, None)
    }
}
