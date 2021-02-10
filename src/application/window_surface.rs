use crate::application::instance::Instance;

use anyhow::{bail, Result};
use ash::{extensions::khr::Surface, version::InstanceV1_0, vk, vk::Handle};
use std::{ptr::null, sync::Arc};

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
