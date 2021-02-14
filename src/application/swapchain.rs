mod images;
mod render_pass;
mod selection;

use crate::application::{Device, WindowSurface};

use anyhow::{Context, Result};
use ash::{extensions::khr, version::DeviceV1_0, vk};
use std::sync::Arc;

/// Bundle up the raw swapchain and the extension functions which are used
/// to operate it.
pub struct Swapchain {
    pub swapchain_loader: khr::Swapchain,
    pub swapchain: vk::SwapchainKHR,

    #[allow(dead_code)]
    swapchain_images: Vec<vk::Image>,

    #[allow(dead_code)]
    swapchain_image_views: Vec<vk::ImageView>,

    #[allow(dead_code)]
    pub framebuffers: Vec<vk::Framebuffer>,

    pub render_pass: vk::RenderPass,
    pub extent: vk::Extent2D,
    pub format: vk::Format,
    pub color_space: vk::ColorSpaceKHR,

    #[allow(dead_code)]
    device: Arc<Device>,
}

impl Swapchain {
    /// Create a new swapchain based on the surface, physical device, and the
    /// current size of the framebuffer.
    pub fn new(
        device: &Arc<Device>,
        window_surface: &WindowSurface,
        framebuffer_size: (u32, u32),
    ) -> Result<Arc<Self>> {
        let image_format = selection::choose_surface_format(
            window_surface,
            &device.physical_device,
        );
        let present_mode = selection::choose_present_mode(
            window_surface,
            &device.physical_device,
        );
        let extent = selection::choose_swap_extent(
            window_surface,
            &device.physical_device,
            framebuffer_size,
        )?;
        let image_count = selection::choose_image_count(
            window_surface,
            &device.physical_device,
        )?;

        let create_info = vk::SwapchainCreateInfoKHR::builder()
            // set the surface
            .surface(window_surface.surface)
            // image settings
            .image_format(image_format.format)
            .image_color_space(image_format.color_space)
            .image_extent(extent)
            .min_image_count(image_count)
            .image_array_layers(1)
            .image_usage(vk::ImageUsageFlags::COLOR_ATTACHMENT)
            // window system presentation settings
            .present_mode(present_mode)
            .composite_alpha(vk::CompositeAlphaFlagsKHR::OPAQUE)
            .pre_transform(vk::SurfaceTransformFlagsKHR::IDENTITY)
            .clipped(true);

        let indices = vec![
            device.graphics_queue.family_id,
            device.present_queue.family_id,
        ];

        let with_sharing_mode =
            if device.present_queue.is_same(&device.graphics_queue) {
                create_info.image_sharing_mode(vk::SharingMode::EXCLUSIVE)
            } else {
                create_info
                    .image_sharing_mode(vk::SharingMode::CONCURRENT)
                    .queue_family_indices(&indices)
            };

        let swapchain_loader =
            khr::Swapchain::new(&device.instance.ash, &device.logical_device);
        let swapchain = unsafe {
            swapchain_loader.create_swapchain(&with_sharing_mode, None)?
        };

        let swapchain_images = unsafe {
            swapchain_loader
                .get_swapchain_images(swapchain)
                .context("unable to get swapchain images")?
        };

        let render_pass =
            render_pass::create_render_pass(device, image_format.format)?;

        let swapchain_image_views = images::create_image_views(
            device,
            image_format.format,
            &swapchain_images,
        )?;

        let framebuffers = images::create_framebuffers(
            device,
            &swapchain_image_views,
            render_pass,
            extent,
        )?;

        Ok(Arc::new(Self {
            swapchain_loader,
            swapchain,
            render_pass,
            swapchain_images,
            swapchain_image_views,
            framebuffers,
            extent,
            format: image_format.format,
            color_space: image_format.color_space,
            device: device.clone(),
        }))
    }
}

impl Drop for Swapchain {
    fn drop(&mut self) {
        let graphics_queue = self.device.graphics_queue.acquire();
        let present_queue = self.device.present_queue.acquire();
        unsafe {
            self.device
                .logical_device
                .queue_wait_idle(*graphics_queue)
                .expect("wait for graphics queue to drain");
            self.device
                .logical_device
                .queue_wait_idle(*present_queue)
                .expect("wait for presentation queue to drain");
            self.device
                .logical_device
                .device_wait_idle()
                .expect("wait for device to idle");

            let logical_device = &self.device.logical_device;
            self.framebuffers.drain(..).for_each(|framebuffer| {
                logical_device.destroy_framebuffer(framebuffer, None);
            });
            self.swapchain_image_views.drain(..).for_each(|view| {
                logical_device.destroy_image_view(view, None);
            });
            self.device
                .logical_device
                .destroy_render_pass(self.render_pass, None);
            self.swapchain_loader
                .destroy_swapchain(self.swapchain, None);
        }
    }
}
