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
}
