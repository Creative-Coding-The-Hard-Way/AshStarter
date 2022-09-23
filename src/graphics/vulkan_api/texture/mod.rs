use std::sync::Arc;

use ash::vk;

use crate::graphics::vulkan_api::{
    RenderDevice, Swapchain, VulkanDebug, VulkanError,
};

enum ImageViewResource {
    /// Used when the ImageView is for a swapchain image.
    Swapchain(Arc<Swapchain>),
}

/// An owned Vulkan image view which keeps the viewed resource alive for its
/// entire lifetime.
pub struct ImageView {
    image_view: vk::ImageView,
    _viewed_resource: ImageViewResource,
    render_device: Arc<RenderDevice>,
}

impl ImageView {
    /// Create an ImageView for every image owned by the Swapchain.
    pub fn for_swapchain_image(
        render_device: Arc<RenderDevice>,
        swapchain: Arc<Swapchain>,
        image_index: usize,
    ) -> Result<Self, VulkanError> {
        // It is safe to use the swapchain images because each view will keep
        // a reference to the swapchain to keep it alive until the images are
        // done being used.
        let images = unsafe { swapchain.images() };

        let create_info = vk::ImageViewCreateInfo {
            image: images[image_index],
            format: swapchain.format(),
            view_type: vk::ImageViewType::TYPE_2D,
            subresource_range: vk::ImageSubresourceRange {
                aspect_mask: vk::ImageAspectFlags::COLOR,
                base_mip_level: 0,
                level_count: 1,
                base_array_layer: 0,
                layer_count: 1,
            },
            components: vk::ComponentMapping {
                r: vk::ComponentSwizzle::IDENTITY,
                g: vk::ComponentSwizzle::IDENTITY,
                b: vk::ComponentSwizzle::IDENTITY,
                a: vk::ComponentSwizzle::IDENTITY,
            },
            ..Default::default()
        };
        let raw_image_view =
            unsafe { render_device.create_image_view(&create_info)? };
        Ok(Self {
            image_view: raw_image_view,
            _viewed_resource: ImageViewResource::Swapchain(swapchain.clone()),
            render_device,
        })
    }

    /// Get the underlying Vulkan ImageView resource handle.
    ///
    /// # Safety
    ///
    /// Unsafe because ownership is not transferred. The caller is responsible
    /// for ensuring no references to the image view remain after this object
    /// is dropped.
    pub unsafe fn raw(&self) -> vk::ImageView {
        self.image_view
    }
}

impl VulkanDebug for ImageView {
    fn set_debug_name(&self, debug_name: impl Into<String>) {
        self.render_device.name_vulkan_object(
            debug_name,
            vk::ObjectType::IMAGE_VIEW,
            self.image_view,
        );
    }
}

impl Drop for ImageView {
    fn drop(&mut self) {
        unsafe {
            self.render_device.destroy_image_view(self.image_view);
        }
    }
}
