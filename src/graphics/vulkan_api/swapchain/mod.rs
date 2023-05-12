use {
    crate::graphics::{vulkan_api::RenderDevice, GraphicsError},
    anyhow::Context,
    ash::{extensions, vk},
    ccthw_ash_instance::VulkanHandle,
    std::sync::Arc,
};

mod acquire_present;
mod selection;

pub use self::acquire_present::SwapchainStatus;

/// The Vulkan swapchain, loader, images, image views, and related data.
///
/// It's often useful to keep the raw Vulkan swapchain together with all of
/// it's related information. It's also helpful to have a newtype which can
/// define some helper functions for working with swapchain resources.
pub struct Swapchain {
    images: Vec<vk::Image>,
    extent: vk::Extent2D,
    format: vk::SurfaceFormatKHR,
    present_mode: vk::PresentModeKHR,
    swapchain: vk::SwapchainKHR,
    swapchain_loader: extensions::khr::Swapchain,
    render_device: Arc<RenderDevice>,
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
    /// * `previous_swapchain` - the previous swapchain (if any). This is
    ///   provided to the new swapchain and will be destroyed inside this
    ///   method.
    ///
    /// # Safety
    ///
    /// Unsafe because:
    ///   - the application must destroy the swapchain before the render device
    ///   - the application must synchronize access to GPU resources
    ///   - the application is responsible for ensuring no GPU resources still
    ///     reference the previous swapchain when it is provided to this method.
    ///     The previous swapchain will be destroyed when the new swapchain is
    ///     constructed.
    pub unsafe fn new(
        render_device: Arc<RenderDevice>,
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
        let swapchain = unsafe {
            swapchain_loader
                .create_swapchain(&create_info, None)
                .context("Error creating the swapchain!")?
        };

        let images = unsafe {
            swapchain_loader
                .get_swapchain_images(swapchain)
                .context("Error getting swapchain images!")?
        };

        Ok(Self {
            images,
            extent,
            format,
            present_mode,
            swapchain,
            swapchain_loader,
            render_device,
        })
    }

    /// Access the raw Swapchain images.
    ///
    /// # Safety
    ///
    /// Unsafe because:
    ///   - the application must synchronize access to swapchain images
    ///   - the images are destroyed when the swapchain is replaced, the
    ///     application must ensure the image handles are not referenced after
    ///     any calls to destroy.
    pub unsafe fn images(&self) -> &[vk::Image] {
        &self.images
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
}

impl Drop for Swapchain {
    /// Destroy the swapchain.
    ///
    /// # Safety
    ///
    /// Unsafe because:
    ///   - the application must drop the swapchain before the logical device
    ///     and vulkan instance
    ///   - the application must synchronize access to GPU resources and ensure
    ///     no pending operations still depend on the swapchain
    fn drop(&mut self) {
        unsafe {
            self.swapchain_loader
                .destroy_swapchain(self.swapchain, None);
        }
    }
}

impl std::fmt::Debug for Swapchain {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Swapchain")
            .field("images", &self.images)
            .field("extent", &self.extent)
            .field("format", &self.format)
            .field("present_mode", &self.present_mode)
            .finish()
    }
}

impl std::fmt::Display for Swapchain {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_fmt(format_args!("{self:#?}"))
    }
}

impl VulkanHandle for Swapchain {
    type Handle = vk::SwapchainKHR;

    unsafe fn raw(&self) -> &Self::Handle {
        &self.swapchain
    }
}
