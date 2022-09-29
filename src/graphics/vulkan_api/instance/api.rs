use std::os::raw::c_void;

use ash::vk;

use super::{Instance, PhysicalDeviceFeatures};
use crate::graphics::vulkan_api::VulkanError;

impl Instance {
    /// Get the properties of queues associated with the given physical device.
    pub fn get_physical_device_queue_family_properties(
        &self,
        physical_device: &vk::PhysicalDevice,
    ) -> Vec<vk::QueueFamilyProperties> {
        unsafe {
            self.ash
                .get_physical_device_queue_family_properties(*physical_device)
        }
    }

    /// Get all device extensions for the given physical device.
    pub fn enumerate_device_extension_properties(
        &self,
        physical_device: &vk::PhysicalDevice,
    ) -> Vec<vk::ExtensionProperties> {
        unsafe {
            self.ash
                .enumerate_device_extension_properties(*physical_device)
                .unwrap_or_else(|_| vec![])
        }
    }

    /// Get the set of all physical devices available to the Vulkan instance.
    pub fn enumerate_physical_devices(
        &self,
    ) -> Result<Vec<vk::PhysicalDevice>, VulkanError> {
        unsafe {
            self.ash
                .enumerate_physical_devices()
                .map_err(VulkanError::UnableToEnumeratePhysicalDevices)
        }
    }

    /// Get the physical device's memory properties.
    pub fn get_physical_device_memory_properties(
        &self,
        physical_device: &vk::PhysicalDevice,
    ) -> vk::PhysicalDeviceMemoryProperties {
        unsafe {
            self.ash
                .get_physical_device_memory_properties(*physical_device)
        }
    }

    /// Get the physical device's features along with specific features enabled
    /// by the get_physical_device_features2 api.
    pub fn get_physical_device_features2(
        &self,
        physical_device: vk::PhysicalDevice,
    ) -> PhysicalDeviceFeatures {
        let mut physical_device_descriptor_indexing_features =
            vk::PhysicalDeviceDescriptorIndexingFeatures::default();
        let mut physical_device_features_v2 = vk::PhysicalDeviceFeatures2 {
            p_next: &mut physical_device_descriptor_indexing_features
                as *mut vk::PhysicalDeviceDescriptorIndexingFeatures
                as *mut c_void,
            ..Default::default()
        };
        unsafe {
            self.ash.get_physical_device_features2(
                physical_device,
                &mut physical_device_features_v2,
            );
        }
        PhysicalDeviceFeatures {
            features: physical_device_features_v2.features,
            descriptor_indexing_features:
                physical_device_descriptor_indexing_features,
        }
    }
}
