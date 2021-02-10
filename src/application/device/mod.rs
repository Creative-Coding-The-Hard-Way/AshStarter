//! This module provides functions for picking a physical device and creating
//! the logical device.

mod queue_family_indices;

use crate::application::instance::Instance;
use anyhow::{Context, Result};
use ash::{version::InstanceV1_0, vk};
use queue_family_indices::QueueFamilyIndices;
use std::sync::Arc;

/// This struct holds all device-specific resources, the physical device and
/// logical device for interacting with it, and the associated queues.
pub struct Device {
    physical_device: vk::PhysicalDevice,

    /// the instance must live longer than the devices which depend on it
    _instance: Arc<Instance>,
}

impl Device {
    /// Create a new device based on this application's required features and
    /// properties.
    pub fn new(instance: &Arc<Instance>) -> Result<Arc<Device>> {
        Ok(Arc::new(Self {
            physical_device: Self::pick_physical_device(instance)?,
            _instance: instance.clone(),
        }))
    }

    /// Pick a physical device based on suitability criteria.
    fn pick_physical_device(instance: &Instance) -> Result<vk::PhysicalDevice> {
        let physical_devices =
            unsafe { instance.ash.enumerate_physical_devices()? };
        let physical_device = physical_devices
            .iter()
            .find(|device| Self::is_device_suitable(&instance, device))
            .context("unable to pick a suitable device")?;
        Ok(*physical_device)
    }

    /// Return true when the device is suitable for this application.
    fn is_device_suitable(
        instance: &Instance,
        physical_device: &vk::PhysicalDevice,
    ) -> bool {
        let features = unsafe {
            instance.ash.get_physical_device_features(*physical_device)
        };
        let properties = unsafe {
            instance
                .ash
                .get_physical_device_properties(*physical_device)
        };

        QueueFamilyIndices::find(physical_device, &instance.ash).is_ok()
            && features.geometry_shader == vk::TRUE
            && properties.device_type == vk::PhysicalDeviceType::DISCRETE_GPU
    }
}
