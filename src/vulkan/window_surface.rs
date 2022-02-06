use ash::{extensions::khr, vk};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum WindowSurfaceError {
    #[error(
        "Unable to determine if the device can present images with this queue"
    )]
    UnableToCheckPhysicalDeviceSurfaceSupport(#[source] vk::Result),

    #[error("Unable to get the surface capabilities for a physical device")]
    UnableToGetPhysicalDeviceSurfaceCapabilities(#[source] vk::Result),
}

/// A wrapper for the Surface and Loader used by this application. It's
/// convenient to keep these pieces of data together because they're so
/// frequently used together.
pub struct WindowSurface {
    /// The surface and surface loader used to present framebuffers to the
    /// screen.
    pub loader: khr::Surface,
    pub khr: vk::SurfaceKHR,
}

impl WindowSurface {
    /// Create an instance with the provided surface and loader.
    pub fn new(
        surface_khr: ash::vk::SurfaceKHR,
        surface_loader: ash::extensions::khr::Surface,
    ) -> Self {
        Self {
            loader: surface_loader,
            khr: surface_khr,
        }
    }

    /// Check that a physical device supports rendering to this surface.
    ///
    /// UNSAFE: because the physical device's supported extensions must be
    /// checked prior to querying for queue presentation support.
    pub unsafe fn get_physical_device_surface_support(
        &self,
        physical_device: &vk::PhysicalDevice,
        queue_family_index: u32,
    ) -> Result<bool, WindowSurfaceError> {
        self.loader
            .get_physical_device_surface_support(
                *physical_device,
                queue_family_index,
                self.khr,
            )
            .map_err(
                WindowSurfaceError::UnableToCheckPhysicalDeviceSurfaceSupport,
            )
    }

    /// Returns the set of all supported formats for this device.
    ///
    /// UNSAFE: because the device's supported extensions must be checked prior
    /// to querying the surface formats.
    pub unsafe fn supported_formats(
        &self,
        physical_device: &vk::PhysicalDevice,
    ) -> Vec<vk::SurfaceFormatKHR> {
        self.loader
            .get_physical_device_surface_formats(*physical_device, self.khr)
            .unwrap_or_else(|_| vec![])
    }

    /// Returns the set of all supported presentation modes for this device.
    ///
    /// Unsafe because the device's supported extensions must be checked prior
    /// to querying the presentation modes.
    pub unsafe fn supported_presentation_modes(
        &self,
        physical_device: &vk::PhysicalDevice,
    ) -> Vec<vk::PresentModeKHR> {
        self.loader
            .get_physical_device_surface_present_modes(
                *physical_device,
                self.khr,
            )
            .unwrap_or_else(|_| vec![])
    }

    /// Returns the set of all supported surface capabilities.
    ///
    /// Unsafe because the device's supported extensions must be checked prior
    /// to querying the surface capabilities.
    pub unsafe fn surface_capabilities(
        &self,
        physical_device: &vk::PhysicalDevice,
    ) -> Result<vk::SurfaceCapabilitiesKHR, WindowSurfaceError> {
        self.loader
            .get_physical_device_surface_capabilities(
                *physical_device,
                self.khr,
            ).map_err(WindowSurfaceError::UnableToGetPhysicalDeviceSurfaceCapabilities)
    }
}

impl Drop for WindowSurface {
    /// UNSAFE: There is no internal synchronization with GPU resources. The
    /// application must ensure this object isn't dropped until all other
    /// resources are done using it.
    fn drop(&mut self) {
        unsafe {
            self.loader.destroy_surface(self.khr, None);
        }
    }
}
