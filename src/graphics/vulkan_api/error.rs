use std::str::Utf8Error;

use ash::vk;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum VulkanError {
    #[error(transparent)]
    InvalidDebugLayerName(#[from] Utf8Error),

    #[error("The following extensions are required but unavailable {:?}", .0)]
    RequiredExtensionsNotFound(Vec<String>),

    #[error("Unable to get the available Vulkan extensions {:?}", .0)]
    UnableToListAvailableExtensions(#[source] vk::Result),

    #[error("The following layers are required but unavailable {:?}", .0)]
    RequiredLayersNotFound(Vec<String>),

    #[error("Unable to get the available Vulkan layers {:?}", .0)]
    UnableToListAvailableLayers(#[source] vk::Result),

    #[error("Unable to create a Vulkan instance {:?}", .0)]
    UnableToCreateInstance(#[source] vk::Result),

    #[error("Unable to create the Vulkan debug messenger {:?}", .0)]
    UnableToCreateDebugMessenger(#[source] vk::Result),

    #[error("Unable to enumerate physical devices {:?}", .0)]
    UnableToEnumeratePhysicalDevices(#[source] vk::Result),

    #[error("No suitable physical device found")]
    NoSuitableDeviceFound,

    #[error("Unable to the queue family for physical device support {:?}", .0)]
    UnableToCheckPhysicalDeviceSupport(#[source] vk::Result),

    #[error("Unable to find a queue family for submitting graphics commands")]
    UnableToFindGraphicsQueue,

    #[error(
        "Unable to find a queue family for submitting presentation commands"
    )]
    UnableToFindPresentQueue,

    #[error("Unable to create the Vulkan logical device {:?}", .0)]
    UnableToCreateLogicalDevice(#[source] vk::Result),

    #[error("Unable to get surface capabilities for the physical device {:?}", .0)]
    UnableToGetPhysicalDeviceSurfaceCapabilities(#[source] vk::Result),

    #[error("Unable to create swapchain {:?}", .0)]
    UnableToCreateSwapchain(#[source] vk::Result),

    #[error("Unable to get swapchain images {:?}", .0)]
    UnableToGetSwapchainImages(#[source] vk::Result),

    #[error("Unable to create a Vulkan ImageView {:?}", .0)]
    UnableToCreateImageView(#[source] vk::Result),
}
