use ::{ash::vk, std::sync::Mutex};

use crate::vulkan::{
    render_device::{
        physical_device, GpuQueue, QueueFamilyIndices, RenderDeviceError,
        Swapchain,
    },
    Instance, WindowSurface,
};

/// The render device holds the core Vulkan state and devices which are used
/// by all parts of the application.
pub struct RenderDevice {
    /// The physical device used by this application.
    #[allow(unused)]
    pub physical_device: vk::PhysicalDevice,

    /// The Vulkan logical device used to issue commands to the physical device.
    pub logical_device: ash::Device,

    /// The GPU queue used to submit graphics commands.
    pub graphics_queue: GpuQueue,

    /// The GPU queue used to submit presentation commands.
    pub present_queue: GpuQueue,

    /// The window's swapchain and related resources.
    pub swapchain: Mutex<Option<Swapchain>>,

    /// The Vulkan presentation surface for the current window.
    pub window_surface: WindowSurface,

    /// The Vulkan library instance.
    pub instance: Instance,
}

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
            swapchain: Mutex::new(None),
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

    /// Query the device for MSAA support.
    ///
    /// # Returns
    ///
    /// The minimum between the `desired` sample count and the sample count
    /// supported by the device.
    ///
    /// e.g. if the device supports 4xMSAA and 8xMSAA is desired, this method
    /// will return 4xMSAA. Similarly, if the device supports 4xMSAA and 2xMSAA
    /// is desired, then this method will return 2xMSAA.
    pub fn get_supported_msaa(
        &self,
        desired: vk::SampleCountFlags,
    ) -> vk::SampleCountFlags {
        let props = unsafe {
            self.instance
                .ash
                .get_physical_device_properties(self.physical_device)
        };
        let color_samples = props.limits.framebuffer_color_sample_counts;
        let depth_samples = props.limits.framebuffer_depth_sample_counts;
        let supported_samples = color_samples.min(depth_samples);

        if supported_samples.contains(vk::SampleCountFlags::TYPE_64) {
            desired.min(vk::SampleCountFlags::TYPE_64)
        } else if supported_samples.contains(vk::SampleCountFlags::TYPE_32) {
            desired.min(vk::SampleCountFlags::TYPE_32)
        } else if supported_samples.contains(vk::SampleCountFlags::TYPE_16) {
            desired.min(vk::SampleCountFlags::TYPE_16)
        } else if supported_samples.contains(vk::SampleCountFlags::TYPE_8) {
            desired.min(vk::SampleCountFlags::TYPE_8)
        } else if supported_samples.contains(vk::SampleCountFlags::TYPE_4) {
            desired.min(vk::SampleCountFlags::TYPE_4)
        } else if supported_samples.contains(vk::SampleCountFlags::TYPE_2) {
            desired.min(vk::SampleCountFlags::TYPE_2)
        } else {
            vk::SampleCountFlags::TYPE_1
        }
    }
}

impl Drop for RenderDevice {
    /// The owner must ensure that the render device is only dropped after other
    /// resources which depend on it! There is no internal synchronization.
    fn drop(&mut self) {
        unsafe {
            let mut swapchain = self
                .swapchain
                .lock()
                .expect("Unable to acquire the swapchain mutex");
            if let Some(swapchain) = swapchain.take() {
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
