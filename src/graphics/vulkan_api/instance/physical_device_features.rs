use ash::vk;

/// This type combines raw Vulkan physical device features with extended feature
/// structs which may be needed by the application.
#[derive(Copy, Clone, Debug, Default)]
pub struct PhysicalDeviceFeatures {
    pub features: vk::PhysicalDeviceFeatures,
    pub descriptor_indexing_features:
        vk::PhysicalDeviceDescriptorIndexingFeatures,
}

/// A Function which inspects a PhysicalDeviceFeatures struct and returns
/// TRUE when the provided features are suitable for this application.
pub type ArePhysicalDeviceFeaturesSuitableFn =
    fn(&PhysicalDeviceFeatures) -> bool;
