use super::{physical_device, RenderDevice, RenderDeviceError};

use crate::vulkan::{Instance, WindowSurface};

impl RenderDevice {
    /// Create the Vulkan Render Device.
    pub fn new(
        instance: Instance,
        window_surface: WindowSurface,
    ) -> Result<Self, RenderDeviceError> {
        let physical_device =
            physical_device::find_optimal(&instance.ash, &window_surface)?;

        Ok(Self {
            instance,
            physical_device,
        })
    }
}
