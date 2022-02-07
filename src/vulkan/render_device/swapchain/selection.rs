use ash::vk;

use crate::{
    markdown::MdList,
    vulkan::{errors::SwapchainError, RenderDevice},
};

impl RenderDevice {
    /// Choose the number of images for the swapchain to manage.
    pub(super) fn choose_image_count(
        &self,
    ) -> std::result::Result<u32, SwapchainError> {
        //! querying surface capabilities is safe in this context because the
        //! physical device will not be selected unless it supports the swapchain
        //! extension
        let capabilities = unsafe {
            self.window_surface
                .surface_capabilities(&self.physical_device)?
        };

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
    pub(super) fn choose_surface_format(&self) -> vk::SurfaceFormatKHR {
        //! checking formats is safe because support for the swapchain extension is
        //! verified when picking a physical device
        let formats = unsafe {
            self.window_surface.supported_formats(&self.physical_device)
        };

        log::debug!("available formats: {:#?}", MdList(&formats));

        let format = formats
            .iter()
            .cloned()
            .find(|format| {
                format.color_space == vk::ColorSpaceKHR::SRGB_NONLINEAR
                    && format.format == vk::Format::B8G8R8A8_SRGB
            })
            .unwrap_or_else(|| formats[0]);

        log::debug!("chosen format {:#?}", format);

        format
    }

    /// Choose a presentation mode for the swapchain based on the window and chosen
    /// physical device.
    pub(super) fn choose_present_mode(&self) -> vk::PresentModeKHR {
        //! checking presentation modes is safe because support for the swapchain
        //! extension is verified when picking a physical device
        let modes = unsafe {
            self.window_surface
                .supported_presentation_modes(&self.physical_device)
        };

        log::debug!("available presentation modes: {:?}", MdList(&modes));

        let mode = if modes.contains(&vk::PresentModeKHR::MAILBOX) {
            vk::PresentModeKHR::MAILBOX
        } else {
            vk::PresentModeKHR::IMMEDIATE
        };

        log::debug!("chosen presentation mode {:?}", mode);

        mode
    }

    /// Choose the swap extent for the swapchain based on the window's framebuffer
    /// size.
    pub(super) fn choose_swap_extent(
        &self,
        framebuffer_size: (u32, u32),
    ) -> Result<vk::Extent2D, SwapchainError> {
        //! Getting surface capabilities is safe because suppport for the swapchain
        //! extenstion is verified when picking a physical device
        let capabilities = unsafe {
            self.window_surface
                .surface_capabilities(&self.physical_device)?
        };

        if capabilities.current_extent.width != u32::MAX {
            log::debug!("use current extent {:?}", capabilities.current_extent);
            Ok(capabilities.current_extent)
        } else {
            let (width, height) = framebuffer_size;
            let extent = vk::Extent2D {
                width: width.clamp(
                    capabilities.min_image_extent.width,
                    capabilities.max_image_extent.width,
                ),
                height: height.clamp(
                    capabilities.min_image_extent.height,
                    capabilities.max_image_extent.height,
                ),
            };
            log::debug!("use computed extent {:?}", extent);
            Ok(extent)
        }
    }
}
