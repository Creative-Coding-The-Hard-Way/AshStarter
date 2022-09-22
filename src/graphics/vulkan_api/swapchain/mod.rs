mod selection;

use std::sync::Arc;

use ash::vk;

use crate::graphics::vulkan_api::{RenderDevice, VulkanError};

/// The swapchain and all related resources.
pub struct Swapchain {
    _images: Vec<vk::Image>,
    image_views: Vec<vk::ImageView>,
    loader: ash::extensions::khr::Swapchain,
    swapchain_khr: vk::SwapchainKHR,
    extent: vk::Extent2D,
    render_device: Arc<RenderDevice>,
}

impl Swapchain {
    /// Create a new Swapchain, accounting for the previous swapchain if one
    /// existed.
    pub fn new(
        render_device: Arc<RenderDevice>,
        framebuffer_size: (u32, u32),
        previous: Option<Self>,
    ) -> Result<Self, VulkanError> {
        let format = selection::choose_surface_format(&render_device);
        let mode = selection::choose_present_mode(&render_device);
        let extent =
            selection::choose_swap_extent(&render_device, framebuffer_size)?;
        let image_count = selection::choose_image_count(&render_device)?;

        let mut create_info = vk::SwapchainCreateInfoKHR {
            // it is safe to use the surface KHR reference here because the
            // swapchain keeps a reference to the RenderDevice until dropped.
            surface: unsafe { render_device.surface_khr() },

            // image settings
            image_format: format.format,
            image_color_space: format.color_space,
            image_extent: extent,
            min_image_count: image_count,
            image_array_layers: 1,
            image_usage: vk::ImageUsageFlags::COLOR_ATTACHMENT,

            // window system presentation settings
            present_mode: mode,
            composite_alpha: vk::CompositeAlphaFlagsKHR::OPAQUE,
            pre_transform: vk::SurfaceTransformFlagsKHR::IDENTITY,
            old_swapchain: if previous.is_some() {
                previous.as_ref().unwrap().swapchain_khr
            } else {
                vk::SwapchainKHR::null()
            },
            clipped: 1,

            ..Default::default()
        };

        let indices = render_device.swapchain_queue_family_indices();

        if indices.len() == 1 {
            create_info.image_sharing_mode = vk::SharingMode::EXCLUSIVE;
        } else {
            create_info.image_sharing_mode = vk::SharingMode::CONCURRENT;
            create_info.p_queue_family_indices = indices.as_ptr();
            create_info.queue_family_index_count = indices.len() as u32;
        }

        let loader = render_device.create_swapchain_loader();
        let swapchain_khr = unsafe {
            loader
                .create_swapchain(&create_info, None)
                .map_err(VulkanError::UnableToCreateSwapchain)?
        };
        let images = unsafe {
            loader
                .get_swapchain_images(swapchain_khr)
                .map_err(VulkanError::UnableToGetSwapchainImages)?
        };
        let image_views = selection::create_image_views(
            &render_device,
            &images,
            format.format,
        )?;

        Ok(Self {
            _images: images,
            image_views,
            loader,
            swapchain_khr,
            extent,
            render_device,
        })
    }

    /// Get the 2D extent used to create the swapchain images and views.
    pub fn extent(&self) -> vk::Extent2D {
        self.extent
    }
}

impl Drop for Swapchain {
    /// # Safety
    ///
    /// The application must ensure that all usage of the Swapchain is complete
    /// before dropping.
    fn drop(&mut self) {
        unsafe {
            for &image_view in &self.image_views {
                self.render_device.destroy_image_view(image_view);
            }
            self.loader.destroy_swapchain(self.swapchain_khr, None);
        }
    }
}
