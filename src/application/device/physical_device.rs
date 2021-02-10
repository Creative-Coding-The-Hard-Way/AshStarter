//! Functions for picking a physical device with the features required by this
//! application.

use crate::application::{
    device::queue_family_indices::QueueFamilyIndices, instance::Instance,
};
use anyhow::{Context, Result};
use ash::{version::InstanceV1_0, vk};

/// Pick a physical device based on suitability criteria.
pub fn pick_physical_device(instance: &Instance) -> Result<vk::PhysicalDevice> {
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
pub fn required_device_features() -> vk::PhysicalDeviceFeatures {
    vk::PhysicalDeviceFeatures::builder()
        .geometry_shader(true)
        .build()
}
