mod device_queue;
mod physical_device;
mod queue_families;
mod window_surface;

use ash::vk;

use self::{
    device_queue::DeviceQueue, queue_families::QueueFamilies,
    window_surface::WindowSurface,
};

use crate::graphics::vulkan_api::{Instance, VulkanError};

/// The Vulkan Logical Device and related resources which are needed for
/// presenting graphics to the screen.
pub struct RenderDevice {
    graphics_queue: DeviceQueue,
    present_queue: DeviceQueue,
    logical_device: ash::Device,
    window_surface: WindowSurface,
    instance: Instance,
}

impl RenderDevice {
    /// Create the logical Vulkan Device for this application.
    pub fn new(
        instance: Instance,
        surface_khr: ash::vk::SurfaceKHR,
    ) -> Result<Self, VulkanError> {
        let window_surface = WindowSurface::new(&instance, surface_khr);
        let physical_device = physical_device::find_optimal_physical_device(
            &instance,
            &window_surface,
        )?;
        let queue_families = QueueFamilies::find_for_physical_device(
            &instance,
            &window_surface,
            &physical_device,
        )?;
        let logical_device = instance.create_logical_device(
            &physical_device,
            &physical_device::required_device_extensions(),
            &queue_families.as_queue_create_infos(),
        )?;
        let (graphics_queue, present_queue) =
            queue_families.get_queues(&logical_device);
        Ok(Self {
            graphics_queue,
            present_queue,
            logical_device,
            window_surface,
            instance,
        })
    }

    /// Give a debug name for the Vulkan object owned by this device. The name
    /// set here will be visible in the Vulkan validation layer logs.
    pub fn name_vulkan_object<Name, Handle>(
        &self,
        name: Name,
        object_type: vk::ObjectType,
        handle: Handle,
    ) where
        Name: Into<String>,
        Handle: vk::Handle + Copy,
    {
        let owned_name = name.into();
        let cname = std::ffi::CString::new(owned_name).unwrap();
        let name_info = vk::DebugUtilsObjectNameInfoEXT {
            object_type,
            p_object_name: cname.as_ptr(),
            object_handle: handle.as_raw(),
            ..Default::default()
        };
        self.instance
            .debug_utils_set_object_name(&self.logical_device, &name_info);
    }
}

impl Drop for RenderDevice {
    fn drop(&mut self) {
        unsafe {
            self.logical_device
                .device_wait_idle()
                .expect("Error while idling the device before destruction!");
            self.logical_device.destroy_device(None);
        }
    }
}
