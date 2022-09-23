mod api;
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

/// Types which implement this trait can name their Vulkan resources so they
/// have a friendly name in Vulkan debug logs.
pub trait VulkanDebug {
    /// Set the name which appears in Vulkan debug logs for this object.
    fn set_debug_name(&self, debug_name: impl Into<String>);
}

/// The Vulkan Logical Device and related resources which are needed for
/// presenting graphics to the screen.
///
/// All operations whiche require the logical device are performed using this
/// object.
pub struct RenderDevice {
    graphics_queue: DeviceQueue,
    present_queue: DeviceQueue,
    physical_device: vk::PhysicalDevice,
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
        let render_device = Self {
            graphics_queue,
            present_queue,
            physical_device,
            logical_device,
            window_surface,
            instance,
        };

        if graphics_queue.is_same(&present_queue) {
            render_device.name_vulkan_object(
                "graphics+present queue",
                vk::ObjectType::QUEUE,
                graphics_queue.raw_queue(),
            );
        } else {
            render_device.name_vulkan_object(
                "graphics queue",
                vk::ObjectType::QUEUE,
                graphics_queue.raw_queue(),
            );
            render_device.name_vulkan_object(
                "present queue",
                vk::ObjectType::QUEUE,
                present_queue.raw_queue(),
            );
        }

        Ok(render_device)
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

    /// List all queue families which need access to swapchain images.
    pub fn swapchain_queue_family_indices(&self) -> Vec<u32> {
        let graphics_family_index = self.graphics_queue.family_index();
        let present_family_index = self.present_queue.family_index();
        if graphics_family_index == present_family_index {
            vec![graphics_family_index]
        } else {
            vec![graphics_family_index, present_family_index]
        }
    }

    /// The family index for the graphics queue.
    pub fn graphics_queue_family_index(&self) -> u32 {
        self.graphics_queue.family_index()
    }

    /// List all surface formats supported by this render device.
    pub fn supported_surface_formats(&self) -> Vec<vk::SurfaceFormatKHR> {
        unsafe { self.window_surface.supported_formats(&self.physical_device) }
    }

    /// List all presentation modes supported by this render device.
    pub fn supported_presentation_modes(&self) -> Vec<vk::PresentModeKHR> {
        unsafe {
            self.window_surface
                .supported_presentation_modes(&self.physical_device)
        }
    }

    /// Get the surface capabilities for this render device.
    pub fn surface_capabilities(
        &self,
    ) -> Result<vk::SurfaceCapabilitiesKHR, VulkanError> {
        unsafe {
            self.window_surface
                .surface_capabilities(&self.physical_device)
        }
    }

    /// Get the underlying KHR surface handle for this render device.
    ///
    /// # Safety
    ///
    /// Ownership of the surface is retained by the RenderDevice. It is
    /// the responsibility of the caller to ensure any usage of the
    /// underyling resource completes before the RenderDevice is
    /// destroyed.
    pub unsafe fn surface_khr(&self) -> vk::SurfaceKHR {
        self.window_surface.surface_khr
    }

    /// Create an ash extension loader for a KHR Swapchain.
    pub fn create_swapchain_loader(&self) -> ash::extensions::khr::Swapchain {
        self.instance.create_swapchain_loader(&self.logical_device)
    }

    /// Return the Vulkan queue which can be used for presenting
    /// swapchain images.
    pub fn present_queue(&self) -> vk::Queue {
        self.present_queue.raw_queue()
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
