use {
    crate::graphics::GraphicsError,
    anyhow::Context,
    ash::{extensions, vk},
    ccthw_ash_instance::{PhysicalDevice, VulkanHandle, VulkanInstance},
};

/// The surface targeted by this application and the Ash extension loader which
/// provides access to KHR surface functions.
pub struct WindowSurface {
    surface: vk::SurfaceKHR,
    surface_loader: extensions::khr::Surface,
}

// Public API
// ----------

impl WindowSurface {
    /// Load Vulkan extension functions for interacting with a presentable
    /// surface.
    ///
    /// # Params
    ///
    /// * `instance` - the Vulkan entrypoint for this application
    /// * `surface` - the surface which will be used for presentation. Typically
    ///   provided by the windowing system.
    ///
    /// # Safety
    ///
    /// The application must destroy the surface before the instance is
    /// destroyed.
    pub unsafe fn new(
        instance: &VulkanInstance,
        surface: vk::SurfaceKHR,
    ) -> Self {
        let surface_loader =
            extensions::khr::Surface::new(instance.entry(), instance.ash());
        Self {
            surface,
            surface_loader,
        }
    }

    /// Destroy the surface.
    ///
    /// # Safety
    ///
    /// Unsafe because:
    ///   - It is undefined behavior to use this type after calling destroy.
    ///   - The application must synchronize GPU resources to ensure no pending
    ///     GPU operations still depend on the surface when it's destroyed.
    ///   - The application must destroy the surface before destroying the
    ///     Vulkan instance.
    pub unsafe fn destroy(&mut self) {
        self.surface_loader.destroy_surface(self.surface, None);
    }

    /// Check that a physical device can present swapchain images to the window
    /// surface.
    ///
    /// # Params
    ///
    /// * `physical_device` - the physical device to check for support
    /// * `queue_family_index` - the queue family which will be used for
    ///   presentation. It is possible that the device supports presentation on
    ///   only a subset of all available queue families.
    ///
    /// # Safety
    ///
    /// Unsafe because the queue family index is assumed to be valid and the
    /// physical_device is assumed to still exist.
    pub unsafe fn get_physical_device_surface_support(
        &self,
        physical_device: &PhysicalDevice,
        queue_family_index: usize,
    ) -> Result<bool, GraphicsError> {
        let is_supported = self
            .surface_loader
            .get_physical_device_surface_support(
                *physical_device.raw(),
                queue_family_index as u32,
                self.surface,
            )
            .context("Error checking for physical device surface support!")?;
        Ok(is_supported)
    }
}

impl std::fmt::Debug for WindowSurface {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter
            .debug_struct("WindowSurface")
            .field("surface", &self.surface)
            .finish()
    }
}
