use crate::application::instance::Instance;

use anyhow::{bail, Context, Result};
use ash::{extensions::khr::Surface, version::InstanceV1_0, vk, vk::Handle};
use std::{ptr::null, sync::Arc};
use vk::SurfaceCapabilitiesKHR;

/// Presentation related resources.
pub struct WindowSurface {
    pub surface: vk::SurfaceKHR,
    pub surface_loader: Surface,

    /// The instance must not be destroyed before the WindowSurface
    _instance: Arc<Instance>,
}

impl WindowSurface {
    /// Create a new application window and vulkan surface.
    pub fn new(
        window: &glfw::Window,
        instance: Arc<Instance>,
    ) -> Result<Arc<WindowSurface>> {
        let surface = create_surface(&instance, window)?;
        let surface_loader = Surface::new(&instance.entry, &instance.ash);

        Ok(Arc::new(Self {
            surface,
            surface_loader,
            _instance: instance,
        }))
    }

    /// Returns the set of all supported formats for this device.
    ///
    /// Unsafe because the device's supported extensions must be checked prior
    /// to querying the surface formats.
    pub unsafe fn supported_formats(
        &self,
        physical_device: &vk::PhysicalDevice,
    ) -> Vec<vk::SurfaceFormatKHR> {
        self.surface_loader
            .get_physical_device_surface_formats(*physical_device, self.surface)
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
        self.surface_loader
            .get_physical_device_surface_present_modes(
                *physical_device,
                self.surface,
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
    ) -> Result<SurfaceCapabilitiesKHR> {
        self.surface_loader
            .get_physical_device_surface_capabilities(
                *physical_device,
                self.surface,
            )
            .context("unable to get surface capabiliities for this device")
    }

    /// This application's required surface format.
    pub fn required_format(&self) -> vk::SurfaceFormatKHR {
        vk::SurfaceFormatKHR::builder()
            .format(vk::Format::B8G8R8A8_SRGB)
            .build()
    }
}

impl Drop for WindowSurface {
    fn drop(&mut self) {
        unsafe {
            self.surface_loader.destroy_surface(self.surface, None);
        }
    }
}

/// Create a vulkan surface using the glfw to hide the platform-specific setup.
fn create_surface(
    instance: &Instance,
    window: &glfw::Window,
) -> Result<vk::SurfaceKHR> {
    let mut surface_handle: u64 = 0;
    let result = window.create_window_surface(
        instance.ash.handle().as_raw() as usize,
        null(),
        &mut surface_handle,
    );
    if result != vk::Result::SUCCESS.as_raw() as u32 {
        bail!("unable to create the vulkan surface");
    }
    Ok(vk::SurfaceKHR::from_raw(surface_handle))
}
