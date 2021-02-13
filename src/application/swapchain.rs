use crate::application::{Device, WindowSurface};

use anyhow::Result;
use ash::{extensions::khr, vk, vk::SwapchainCreateInfoKHR};
use std::sync::Arc;

/// Bundle up the raw swapchain and the extension functions which are used
/// to operate it.
pub struct Swapchain {
    swapchain_loader: khr::Swapchain,
    swapchain: vk::SwapchainKHR,

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
        let image_format =
            choose_surface_format(window_surface, &device.physical_device);
        let present_mode =
            choose_present_mode(window_surface, &device.physical_device);
        let extent = choose_swap_extent(
            window_surface,
            &device.physical_device,
            framebuffer_size,
        )?;
        let image_count =
            choose_image_count(window_surface, &device.physical_device)?;

        let create_info = SwapchainCreateInfoKHR::builder()
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

        Ok(Arc::new(Self {
            swapchain_loader,
            swapchain,
            device: device.clone(),
        }))
    }
}

impl Drop for Swapchain {
    fn drop(&mut self) {
        unsafe {
            self.swapchain_loader
                .destroy_swapchain(self.swapchain, None);
        }
    }
}

/// Choose the number of images to use in the swapchain based on the min and
/// max numbers of images supported by the device.
fn choose_image_count(
    window_surface: &WindowSurface,
    physical_device: &vk::PhysicalDevice,
) -> Result<u32> {
    //! querying surface capabilities is safe in this context because the
    //! physical device will not be selected unless it supports the swapchain
    //! extension
    let capabilities =
        unsafe { window_surface.surface_capabilities(physical_device)? };
    let proposed_image_count = capabilities.min_image_count + 1;
    if capabilities.max_image_count > 0 {
        Ok(std::cmp::min(
            proposed_image_count,
            capabilities.max_image_count,
        ))
    } else {
        Ok(proposed_image_count)
    }
}

/// Choose a surface format for the swapchain based on the window and chosen
/// physical device.
///
fn choose_surface_format(
    window_surface: &WindowSurface,
    physical_device: &vk::PhysicalDevice,
) -> vk::SurfaceFormatKHR {
    //! checking formats is safe because support for the swapchain extension is
    //! verified when picking a physical device
    let formats = unsafe { window_surface.supported_formats(physical_device) };

    log::info!("available formats {:?}", formats);

    let format = formats
        .iter()
        .cloned()
        .find(|format| {
            format.color_space == vk::ColorSpaceKHR::SRGB_NONLINEAR
                && format.format == vk::Format::B8G8R8A8_SRGB
        })
        .unwrap_or_else(|| formats[0]);

    log::info!("chosen format {:?}", format);

    format
}

/// Choose a presentation mode for the swapchain based on the window and chosen
/// physical device.
///
pub fn choose_present_mode(
    window_surface: &WindowSurface,
    physical_device: &vk::PhysicalDevice,
) -> vk::PresentModeKHR {
    //! checking presentation modes is safe because support for the swapchain
    //! extension is verified when picking a physical device
    let modes =
        unsafe { window_surface.supported_presentation_modes(physical_device) };

    log::info!("available presentation modes {:?}", modes);

    let mode = if modes.contains(&vk::PresentModeKHR::MAILBOX) {
        vk::PresentModeKHR::MAILBOX
    } else {
        vk::PresentModeKHR::IMMEDIATE
    };

    log::info!("chosen presentation mode {:?}", mode);

    mode
}

/// Choose the swap extent for the swapchain based on the window's framebuffer
/// size.
fn choose_swap_extent(
    window_surface: &WindowSurface,
    physical_device: &vk::PhysicalDevice,
    framebuffer_size: (u32, u32),
) -> Result<vk::Extent2D> {
    //! Getting surface capabilities is safe because suppport for the swapchain
    //! extenstion is verified when picking a physical device
    let capabilities =
        unsafe { window_surface.surface_capabilities(physical_device)? };

    if capabilities.current_extent.width != u32::MAX {
        log::debug!("use current extent {:?}", capabilities.current_extent);
        Ok(capabilities.current_extent)
    } else {
        let extent = vk::Extent2D {
            width: clamp(
                framebuffer_size.0,
                capabilities.min_image_extent.width,
                capabilities.max_image_extent.width,
            ),
            height: clamp(
                framebuffer_size.1,
                capabilities.min_image_extent.height,
                capabilities.max_image_extent.height,
            ),
        };
        log::debug!("use computed extent {:?}", extent);
        Ok(extent)
    }
}

/// Clamp a value between a minimum and maximum bound.
fn clamp(x: u32, min: u32, max: u32) -> u32 {
    std::cmp::max(min, std::cmp::min(x, max))
}
