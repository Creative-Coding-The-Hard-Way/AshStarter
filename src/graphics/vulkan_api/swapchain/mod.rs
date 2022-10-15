use {
    crate::graphics::{vulkan_api::RenderDevice, GraphicsError},
    anyhow::Context,
    ash::{extensions, vk},
    ccthw_ash_instance::VulkanHandle,
};

/// The Vulkan swapchain, loader, images, image views, and related data.
///
/// It's often useful to keep the raw Vulkan swapchain together with all of
/// it's related information. It's also helpful to have a newtype which can
/// define some helper functions for working with swapchain resources.
pub struct Swapchain {
    image_count: u32,
    extent: vk::Extent2D,
    format: vk::SurfaceFormatKHR,
    present_mode: vk::PresentModeKHR,
    swapchain: vk::SwapchainKHR,
    swapchain_loader: extensions::khr::Swapchain,
}

// Public API
// ----------

impl Swapchain {
    /// Create a new swapchain and acompanying resources.
    ///
    /// # Params
    ///
    /// * `render_device` - the device used to create vulkan resources
    /// * `framebuffer_size` - the size of the window's framebuffer in device
    ///   pixels.
    ///
    /// # Safety
    ///
    /// Unsafe because:
    ///   - the application must destroy the swapchain before the render device
    ///   - the application must synchronize access to GPU resources
    pub unsafe fn new(
        render_device: &RenderDevice,
        framebuffer_size: (u32, u32),
        previous_swapchain: Option<Self>,
    ) -> Result<Self, GraphicsError> {
        let format =
            Self::choose_surface_format(&render_device.get_surface_formats()?)?;
        let present_mode =
            Self::choose_presentation_mode(&render_device.get_present_modes()?);
        let capabilities = render_device.get_surface_capabilities()?;
        let extent =
            Self::choose_swapchain_extent(capabilities, framebuffer_size);
        let min_image_count = Self::choose_image_count(capabilities);

        let mut create_info = vk::SwapchainCreateInfoKHR {
            surface: *render_device.surface(),

            // image settings
            min_image_count,
            image_format: format.format,
            image_color_space: format.color_space,
            image_extent: extent,
            image_array_layers: 1,
            image_usage: vk::ImageUsageFlags::COLOR_ATTACHMENT,

            // window system settings
            present_mode,
            composite_alpha: vk::CompositeAlphaFlagsKHR::OPAQUE,
            pre_transform: vk::SurfaceTransformFlagsKHR::IDENTITY,
            old_swapchain: if previous_swapchain.is_some() {
                *previous_swapchain.as_ref().unwrap().raw()
            } else {
                vk::SwapchainKHR::null()
            },
            clipped: vk::TRUE,

            ..Default::default()
        };

        let indices = vec![
            render_device.graphics_queue().family_index(),
            render_device.presentation_queue().family_index(),
        ];
        if indices[0] == indices[1] {
            create_info.image_sharing_mode = vk::SharingMode::EXCLUSIVE;
        } else {
            create_info.image_sharing_mode = vk::SharingMode::CONCURRENT;
            create_info.p_queue_family_indices = indices.as_ptr();
            create_info.queue_family_index_count = indices.len() as u32;
        }
        log::trace!(
            "Using image sharing mode {:?}",
            create_info.image_sharing_mode
        );

        let swapchain_loader = extensions::khr::Swapchain::new(
            render_device.ash(),
            render_device.device(),
        );
        let swapchain =
            unsafe { swapchain_loader.create_swapchain(&create_info, None)? };

        Ok(Self {
            image_count: min_image_count,
            extent,
            format,
            present_mode,
            swapchain,
            swapchain_loader,
        })
    }

    /// The format used by images in the swapchain.
    pub fn image_format(&self) -> vk::Format {
        self.format.format
    }

    /// The extent for all swapchain images.
    pub fn extent(&self) -> vk::Extent2D {
        self.extent
    }

    /// The presentation mode used by this swapchain.
    pub fn present_mode(&self) -> vk::PresentModeKHR {
        self.present_mode
    }

    /// Destroy all swapchain resources.
    ///
    /// # Safety
    ///
    /// Unsafe because:
    ///   - the application must destroy the swapchain before the logical device
    ///     and vulkan instance
    ///   - the application must synchronize access to GPU resources and ensure
    ///     no pending operations still depend on the swapchain
    ///   - it is invalid to use this instance after calling `destroy`
    pub unsafe fn destroy(&mut self) {
        self.swapchain_loader
            .destroy_swapchain(self.swapchain, None);
    }
}

impl std::fmt::Debug for Swapchain {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Swapchain")
            .field("image_count", &self.image_count)
            .field("extent", &self.extent)
            .field("format", &self.format)
            .field("present_mode", &self.present_mode)
            .finish()
    }
}

impl std::fmt::Display for Swapchain {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_fmt(format_args!("{:#?}", self))
    }
}

impl VulkanHandle for Swapchain {
    type Handle = vk::SwapchainKHR;

    unsafe fn raw(&self) -> &Self::Handle {
        &self.swapchain
    }
}

// Private API
// -----------

impl Swapchain {
    /// Chose the swapchain image format given a slice of available formats.
    ///
    /// # Params
    ///
    /// * `available_formats` - the formats available for presentation on the
    ///   device and surface
    fn choose_surface_format(
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
    fn choose_presentation_mode(
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
    fn choose_swapchain_extent(
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
    fn choose_image_count(capabilities: vk::SurfaceCapabilitiesKHR) -> u32 {
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
