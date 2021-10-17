use super::{Image, ImageError, ImageView};

use crate::vulkan::{errors::VulkanDebugError, VulkanDebug};

use ::{
    ash::{version::DeviceV1_0, vk},
    std::sync::Arc,
};

impl ImageView {
    /// Create a new image view for a given image.
    pub fn new(
        image: Arc<Image>,
        create_info: &vk::ImageViewCreateInfo,
    ) -> Result<Self, ImageError> {
        let raw = unsafe {
            image
                .vk_dev
                .logical_device
                .create_image_view(create_info, None)
                .map_err(ImageError::UnableToCreateView)?
        };
        Ok(Self {
            raw,
            vk_dev: image.vk_dev.clone(),
            image,
        })
    }

    /// Create a new 2d image view which targets only the base mipmap.
    pub fn new_2d(
        image: Arc<Image>,
        format: vk::Format,
        aspect_mask: vk::ImageAspectFlags,
    ) -> Result<Self, ImageError> {
        let create_info = vk::ImageViewCreateInfo {
            flags: vk::ImageViewCreateFlags::empty(),
            image: image.raw,
            view_type: vk::ImageViewType::TYPE_2D,
            format,
            subresource_range: vk::ImageSubresourceRange {
                aspect_mask,
                base_mip_level: 0,
                level_count: 1,
                base_array_layer: 0,
                layer_count: 1,
            },
            ..Default::default()
        };
        Self::new(image, &create_info)
    }
}

impl Drop for ImageView {
    /// # DANGER
    ///
    /// There is no internal synchronization for this type. Unexpected behavior
    /// can occur if this instance is still in-use by the GPU when it is
    /// dropped.
    fn drop(&mut self) {
        unsafe {
            self.vk_dev
                .logical_device
                .destroy_image_view(self.raw, None);
        }
    }
}

impl VulkanDebug for ImageView {
    fn set_debug_name(
        &self,
        debug_name: impl Into<String>,
    ) -> Result<(), VulkanDebugError> {
        self.vk_dev.name_vulkan_object(
            debug_name,
            vk::ObjectType::IMAGE_VIEW,
            self.raw,
        )?;
        Ok(())
    }
}
