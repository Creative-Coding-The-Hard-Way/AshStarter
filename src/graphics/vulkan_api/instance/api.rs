use ash::vk;

use super::Instance;
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
}
