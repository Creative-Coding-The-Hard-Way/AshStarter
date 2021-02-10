//! This module provides functions for picking a physical device and creating
//! the logical device.

mod queue_family_indices;

use crate::{application::instance::Instance, ffi::to_os_ptrs};
use anyhow::{Context, Result};
use ash::{
    version::{DeviceV1_0, InstanceV1_0},
    vk,
};
use queue_family_indices::QueueFamilyIndices;
use std::{ffi::CString, sync::Arc};

/// This struct holds all device-specific resources, the physical device and
/// logical device for interacting with it, and the associated queues.
pub struct Device {
    pub physical_device: vk::PhysicalDevice,
    pub logical_device: ash::Device,

    /// the instance must live longer than the devices which depend on it
    instance: Arc<Instance>,
}

impl Device {
    /// Create a new device based on this application's required features and
    /// properties.
    pub fn new(instance: &Arc<Instance>) -> Result<Arc<Device>> {
        let physical_device = pick_physical_device(instance)?;
        let logical_device = create_logical_device(instance, &physical_device)?;

        let device = Arc::new(Self {
            physical_device,
            logical_device,
            instance: instance.clone(),
        });

        device.name_vulkan_object(
            "Application Logical Device",
            &device.logical_device.handle(),
        )?;

        Ok(device)
    }

    /// Give a debug name for a vulkan object owned by this device.
    ///
    /// Whatever name is provided here will show up in the debug logs if there
    /// are any issues detected by the validation layers.
    ///
    /// # Example
    ///
    /// ```
    /// device.name_vulkan_object(
    ///     "Application Logical Device",
    ///     &device.logical_device.handle()
    /// )?;
    /// ```
    ///
    pub fn name_vulkan_object<Name, Handle>(
        &self,
        name: Name,
        handle: &Handle,
    ) -> Result<()>
    where
        Handle: vk::Handle + Copy,
        Name: Into<String>,
    {
        let cname = CString::new(name.into()).unwrap();

        let name_info = vk::DebugUtilsObjectNameInfoEXT::builder()
            .object_name(&cname)
            .object_type(vk::ObjectType::DEVICE)
            .object_handle(handle.as_raw());

        unsafe {
            self.instance.debug.debug_utils_set_object_name(
                self.logical_device.handle(),
                &name_info,
            )?;
        }

        Ok(())
    }
}

impl Drop for Device {
    /// Destroy the logical device.
    ///
    /// Device owns an Arc<Instance> so it's guaranteed that the instance will
    /// not be destroyed until the logical device has been dropped.
    fn drop(&mut self) {
        unsafe {
            self.logical_device.destroy_device(None);
        }
    }
}

/// Create a new logical device for use by this application. The caller is
/// responsible for destroying the device when done.
fn create_logical_device(
    instance: &Instance,
    physical_device: &vk::PhysicalDevice,
) -> Result<ash::Device> {
    let queue_family_indices =
        QueueFamilyIndices::find(physical_device, &instance.ash)?;

    let queue_create_infos = queue_family_indices.as_queue_create_infos();
    let features = required_device_features();
    let (_c_names, layer_name_ptrs) =
        unsafe { to_os_ptrs(&instance.enabled_layer_names) };

    let create_info = vk::DeviceCreateInfo::builder()
        .queue_create_infos(&queue_create_infos)
        .enabled_features(&features)
        .enabled_layer_names(&layer_name_ptrs);

    let logical_device = unsafe {
        instance
            .ash
            .create_device(*physical_device, &create_info, None)
            .context("unable to create the logical device")?
    };

    Ok(logical_device)
}

/// Pick a physical device based on suitability criteria.
fn pick_physical_device(instance: &Instance) -> Result<vk::PhysicalDevice> {
    let physical_devices =
        unsafe { instance.ash.enumerate_physical_devices()? };
    let physical_device = physical_devices
        .iter()
        .find(|device| is_device_suitable(&instance, device))
        .context("unable to pick a suitable device")?;
    Ok(*physical_device)
}

/// Return true when the device is suitable for this application.
fn is_device_suitable(
    instance: &Instance,
    physical_device: &vk::PhysicalDevice,
) -> bool {
    let features =
        unsafe { instance.ash.get_physical_device_features(*physical_device) };
    let properties = unsafe {
        instance
            .ash
            .get_physical_device_properties(*physical_device)
    };

    QueueFamilyIndices::find(physical_device, &instance.ash).is_ok()
        && features.geometry_shader == vk::TRUE
        && properties.device_type == vk::PhysicalDeviceType::DISCRETE_GPU
}

/// Return the set of required device features for this application.
///
/// `is_device_suitable` should verify that all required features are supported
/// by the chosen physical device.
fn required_device_features() -> vk::PhysicalDeviceFeatures {
    vk::PhysicalDeviceFeatures::builder()
        .geometry_shader(true)
        .build()
}
