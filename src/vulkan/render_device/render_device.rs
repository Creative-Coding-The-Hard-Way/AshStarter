use super::{
    physical_device, QueueFamilyIndices, RenderDevice, RenderDeviceError,
};
use crate::vulkan::{Instance, WindowSurface};

use ash::{version::DeviceV1_0, vk};

impl RenderDevice {
    /// Create the Vulkan Render Device.
    ///
    /// The device is initialized *without* a swapchain initially. An
    /// additional call to `rebuild_swapchain` is needed prior to any rendering
    /// operations.
    pub fn new(
        instance: Instance,
        window_surface: WindowSurface,
    ) -> Result<Self, RenderDeviceError> {
        let physical_device =
            physical_device::find_optimal(&instance.ash, &window_surface)?;
        let queue_family_indices = QueueFamilyIndices::find(
            &instance.ash,
            &physical_device,
            &window_surface,
        )?;
        let logical_device = instance.create_logical_device(
            &physical_device,
            physical_device::required_features(),
            &physical_device::required_extensions(),
            &queue_family_indices.as_queue_create_infos(),
        )?;
        let (graphics_queue, present_queue) =
            queue_family_indices.get_queues(&logical_device);

        let vk_dev = Self {
            instance,
            physical_device,
            logical_device,
            graphics_queue,
            present_queue,
            window_surface,
            swapchain: None,
        };

        vk_dev.name_vulkan_object(
            "Graphics Queue",
            vk::ObjectType::QUEUE,
            vk_dev.graphics_queue.queue,
        )?;
        if !vk_dev.graphics_queue.is_same(&vk_dev.present_queue) {
            vk_dev.name_vulkan_object(
                "Present Queue",
                vk::ObjectType::QUEUE,
                vk_dev.present_queue.queue,
            )?;
        }

        Ok(vk_dev)
    }

    /// Give a debug name for a vulkan object owned by this device.
    ///
    /// Whatever name is provided here will show up in the debug logs if there
    /// are any issues detected by the validation layers.
    pub fn name_vulkan_object<Name, Handle>(
        &self,
        name: Name,
        object_type: vk::ObjectType,
        handle: Handle,
    ) -> Result<(), RenderDeviceError>
    where
        Name: Into<String>,
        Handle: vk::Handle + Copy,
    {
        let owned_name = name.into();
        let cname = std::ffi::CString::new(owned_name.clone()).unwrap();
        let name_info = vk::DebugUtilsObjectNameInfoEXT {
            object_type,
            p_object_name: cname.as_ptr(),
            object_handle: handle.as_raw(),
            ..Default::default()
        };

        unsafe {
            self.instance
                .debug
                .debug_utils_set_object_name(
                    self.logical_device.handle(),
                    &name_info,
                )
                .map_err(|error| {
                    RenderDeviceError::UnableToSetDebugName(
                        owned_name,
                        object_type,
                        error,
                    )
                })
        }
    }
}

impl Drop for RenderDevice {
    /// The owner must ensure that the render device is only dropped after other
    /// resources which depend on it! There is no internal synchronization.
    fn drop(&mut self) {
        unsafe {
            if let Some(swapchain) = self.swapchain.take() {
                self.destroy_swapchain(swapchain)
                    .expect("Error while destroying the swapchain");
            }
            self.logical_device
                .device_wait_idle()
                .expect("Error while waiting for device work to finish");
            self.logical_device.destroy_device(None);
        }
    }
}
