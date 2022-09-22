use crate::graphics::vulkan_api::{Instance, VulkanError};
use ash::{extensions::khr, vk};

/// The KHR Surface and Loader used by this application. These resources must
/// be dropped before the instance.
pub struct WindowSurface {
    surface_khr: vk::SurfaceKHR,
    loader: khr::Surface,
}

impl WindowSurface {
    /// The vk SurfaceKHR instance is typically provided by the window system,
    /// and the extension loader is provided by the vulkan instance.
    pub fn new(instance: &Instance, surface_khr: ash::vk::SurfaceKHR) -> Self {
        Self {
            surface_khr,
            loader: instance.create_surface_loader(),
        }
    }

    /// Check that the window surface can be presented using the provided
    /// physical device and queue family index.
    ///
    /// # Safety
    ///
    /// Unsafe because the physical device's extensions must be checked prior to
    /// querying for queue presentation support.
    pub unsafe fn get_physical_device_surface_support(
        &self,
        physical_device: &vk::PhysicalDevice,
        queue_family_index: u32,
    ) -> Result<bool, VulkanError> {
        self.loader
            .get_physical_device_surface_support(
                *physical_device,
                queue_family_index,
                self.surface_khr,
            )
            .map_err(VulkanError::UnableToCheckPhysicalDeviceSupport)
    }

    /// Get the set of all supported formats for this device.
    ///
    /// # Safety
    ///
    /// Unsafe because the devices supported extensions must be checeked prior
    /// to calling this function.
    pub unsafe fn supported_formats(
        &self,
        physical_device: &vk::PhysicalDevice,
    ) -> Vec<vk::SurfaceFormatKHR> {
        self.loader
            .get_physical_device_surface_formats(
                *physical_device,
                self.surface_khr,
            )
            .unwrap_or_else(|_| vec![])
    }

    /// Get the set of all supported presentation modes for this device.
    ///
    /// # Safety
    ///
    /// Unsafe because the devices supported extensinos must be checked prior
    /// to calling this function.
    pub unsafe fn supported_presentation_modes(
        &self,
        physical_device: &vk::PhysicalDevice,
    ) -> Vec<vk::PresentModeKHR> {
        self.loader
            .get_physical_device_surface_present_modes(
                *physical_device,
                self.surface_khr,
            )
            .unwrap_or_else(|_| vec![])
    }
}

impl Drop for WindowSurface {
    /// Destroy the KHR Surface.
    ///
    /// # Safety
    ///
    /// Unsafe because there is no check that the surface is done being used
    /// by the application. It is up to the application to ensure all GPU
    /// operations which use these resources are finished prior to dropping.
    fn drop(&mut self) {
        unsafe {
            self.loader.destroy_surface(self.surface_khr, None);
        }
    }
}
