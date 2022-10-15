use {
    crate::graphics::GraphicsError,
    ash::vk,
    ccthw_ash_instance::{
        LogicalDevice, PhysicalDevice, PhysicalDeviceFeatures, VulkanInstance,
    },
};

mod queue;
mod queue_finder;

use {self::queue_finder::QueueFinder, ccthw_ash_instance::VulkanHandle};

pub use self::queue::Queue;

/// A combination of the VulkanInstance, LogicalDevice, and queues required by
/// this application.
#[derive(Debug)]
pub struct RenderDevice {
    graphics_queue: Queue,
    logical_device: LogicalDevice,
    instance: VulkanInstance,
}

// Public Api
// ----------

impl RenderDevice {
    /// Create a new render device.
    ///
    /// # Params
    ///
    /// * `instance` - the VulkanInstance used to create all application
    ///   resources. The RenderDevice takes ownership of the vulkan instance so
    ///   it can be destroyed in the correct order.
    /// * `features` - the physical device features required by this
    ///   application.
    ///
    /// # Safety
    ///
    /// Unsafe because the application must destroy the render device before
    /// exit. The application must also destroy all resources created by the
    /// logical device before destroying the render device.
    pub unsafe fn new(
        instance: VulkanInstance,
        features: PhysicalDeviceFeatures,
    ) -> Result<Self, GraphicsError> {
        let physical_device = Self::pick_physical_device(&instance, features)?;
        let queue_finder = QueueFinder::new(&physical_device);

        let logical_device = unsafe {
            // SAFE because the RenderDevice takes ownership of the instance
            // along with the LogicalDevice.
            LogicalDevice::new(
                &instance,
                physical_device,
                &[],
                &queue_finder.queue_family_infos(),
            )?
        };

        let graphics_queue =
            queue_finder.get_queues_from_device(&logical_device);

        Ok(Self {
            graphics_queue,
            logical_device,
            instance,
        })
    }

    /// The queue this application uses for graphics operations.
    pub fn graphics_queue(&self) -> &Queue {
        &self.graphics_queue
    }

    /// Destroy the logical device and ash instance.
    ///
    /// # Safety
    ///
    /// The application must call this prior to exit. All resources created
    /// using the logical device must be destroyed prior to calling this method.
    /// The application is responsible for synchronizing access to GPU
    /// resources.
    pub unsafe fn destroy(&mut self) {
        self.logical_device.raw().destroy_device(None);
        self.instance.ash().destroy_instance(None);
    }

    /// The Ash entry used by this RenderDevice.
    pub fn entry(&self) -> &ash::Entry {
        self.instance.entry()
    }

    /// The Ash instance used by this RenderDevice.
    pub fn ash(&self) -> &ash::Instance {
        self.instance.ash()
    }

    /// The Ash logical device used to interface with the underlying Vulkan
    /// hardware device.
    ///
    /// # Safety
    ///
    /// The caller must not keep copies of the device handle after any calls
    /// to `destroy`.
    pub unsafe fn device(&self) -> &ash::Device {
        self.logical_device.raw()
    }
}

impl std::fmt::Display for RenderDevice {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str("RenderDevice\n")?;
        formatter.write_fmt(format_args!(" -> {}\n", &self.logical_device))?;
        formatter.write_fmt(format_args!(
            " -> Graphics {}\n",
            &self.graphics_queue
        ))?;
        Ok(())
    }
}

// Private API
// -----------

impl RenderDevice {
    /// Pick a physical device which is suitable for this application.
    ///
    /// # Params
    ///
    /// * `instance` - the Vulkan instance used to access devices on this
    ///   platform.
    /// * `features` - all features required by this application.
    fn pick_physical_device(
        instance: &VulkanInstance,
        features: PhysicalDeviceFeatures,
    ) -> Result<PhysicalDevice, GraphicsError> {
        let devices: Vec<PhysicalDevice> =
            PhysicalDevice::enumerate_supported_devices(instance, &features)?
                .into_iter()
                .filter(QueueFinder::device_has_required_queues)
                .collect();
        let find_device_type =
            |device_type: vk::PhysicalDeviceType| -> Option<&PhysicalDevice> {
                devices.iter().find(|device| {
                    device.properties().properties().device_type == device_type
                })
            };

        if let Some(device) =
            find_device_type(vk::PhysicalDeviceType::DISCRETE_GPU)
        {
            return Ok(device.clone());
        }

        if let Some(device) =
            find_device_type(vk::PhysicalDeviceType::INTEGRATED_GPU)
        {
            return Ok(device.clone());
        }

        let device = devices
            .first()
            .ok_or(GraphicsError::NoSuitablePhysicalDevice)?;
        Ok(device.clone())
    }
}
