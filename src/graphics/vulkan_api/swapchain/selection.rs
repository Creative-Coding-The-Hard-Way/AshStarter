//! Private Swapchain Selection API

use {
    super::Swapchain, crate::graphics::GraphicsError, anyhow::Context, ash::vk,
};

impl Swapchain {
    /// Chose the swapchain image format given a slice of available formats.
    ///
    /// # Params
    ///
    /// * `available_formats` - the formats available for presentation on the
    ///   device and surface
    pub(super) fn choose_surface_format(
        available_formats: &[vk::SurfaceFormatKHR],
    ) -> Result<vk::SurfaceFormatKHR, GraphicsError> {
        log::trace!("Available surface formats: {:#?}", available_formats);

        let preferred_format = available_formats.iter().find(|format| {
            format.color_space == vk::ColorSpaceKHR::SRGB_NONLINEAR
                && format.format == vk::Format::B8G8R8A8_SRGB
        });

        if let Some(&format) = preferred_format {
            log::trace!("Using preferred swapchain format {:#?}", format);
            return Ok(format);
        }

        let backup_format = available_formats
            .first()
            .context("No swapchain formats available!")?;

        log::trace!("Fall back to swapchain format {:#?}", backup_format);

        Ok(*backup_format)
    }

    /// Chose the swapchain presentation mode given the set of available modes.
    ///
    /// # Params
    ///
    /// * `available_modes` - the presentation modes supported by the device and
    ///   surface.
    pub(super) fn choose_presentation_mode(
        available_modes: &[vk::PresentModeKHR],
    ) -> vk::PresentModeKHR {
        let preferred_mode = vk::PresentModeKHR::MAILBOX;
        if available_modes.contains(&preferred_mode) {
            log::trace!(
                "Using preferred swapchain present mode {:?}",
                preferred_mode
            );
            return preferred_mode;
        }

        // guaranteed to be available by the Vulkan spec
        let fallback_mode = vk::PresentModeKHR::FIFO;
        log::trace!("Fall back to swapchain present mode {:?}", fallback_mode);

        fallback_mode
    }

    /// Chose the swapchain size given the swapchain limits on framebuffer size.
    ///
    /// # Params
    ///
    /// * `capabilities` - the available surface capabilities for the device
    /// * `framebuffer_size` - the raw framebuffer size in pixels
    pub(super) fn choose_swapchain_extent(
        capabilities: vk::SurfaceCapabilitiesKHR,
        framebuffer_size: (u32, u32),
    ) -> vk::Extent2D {
        let (width, height) = framebuffer_size;

        if capabilities.current_extent.width != u32::MAX {
            // u32::MAX indicates that the surface size will be controlled
            // entirely by the size of the swapchain targeting the
            // surface.
            vk::Extent2D { width, height }
        } else {
            // Otherwise we have to make sure the swapchain size is within the
            // allowed min/max values.
            vk::Extent2D {
                width: width.clamp(
                    capabilities.min_image_extent.width,
                    capabilities.max_image_extent.width,
                ),
                height: height.clamp(
                    capabilities.min_image_extent.height,
                    capabilities.max_image_extent.height,
                ),
            }
        }
    }

    /// Chose the number of swapchain images to use.
    ///
    /// # Params
    ///
    /// * `capabilities` - the available surface capabilities for the device
    pub(super) fn choose_image_count(
        capabilities: vk::SurfaceCapabilitiesKHR,
    ) -> u32 {
        let proposed_image_count = 3;
        if capabilities.max_image_count > 0 {
            proposed_image_count.clamp(
                capabilities.min_image_count,
                capabilities.max_image_count,
            )
        } else {
            proposed_image_count.max(capabilities.min_image_count)
        }
    }
}
