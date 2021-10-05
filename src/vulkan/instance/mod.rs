mod debug_callback;
mod extensions;
mod instance;
mod layers;

use ash::{extensions::ext::DebugUtils, vk, Entry};
use thiserror::Error;

/// This enum represents errors which can occur when building and handling the
/// Vulkan instance.
#[derive(Debug, Error)]
pub enum InstanceError {
    #[error("Unable to setup the Vulkan debug callback")]
    DebugMessengerCreateFailed(#[source] vk::Result),

    #[error("Unable to list the available Vulkan extensions on this platform")]
    UnableToListAvailableExtensions(#[source] vk::Result),

    #[error("Required extensions are not available on this platform: {:?}", .0)]
    RequiredExtensionsNotFound(Vec<String>),

    #[error("Unable to list the available Vulkan layers on this platform")]
    UnableToListAvailableLayers(#[source] vk::Result),

    #[error("Required layers are not available on this platform: {:?}", .0)]
    RequiredLayersNotFound(Vec<String>),

    #[error("Error while creating the Vulkan function loader")]
    VulkanLoadingError(#[source] ash::LoadingError),

    #[error("Error while creating the Vulkan function loader")]
    InvalidDebugLayerName(#[source] std::str::Utf8Error),

    #[error("Unable to create the Vulkan instance")]
    UnableToCreateInstance(#[source] ash::InstanceError),

    #[error("Unable to create the logical device")]
    UnableToCreateLogicalDevice(#[source] vk::Result),
}

/// The Instance struct holds the ash entry and ash library handle along with
/// the debug callback.
///
/// # Example
///
///     use ccthw::vulkan::Instance;
///
///     // Typically the required extensions come from the window system.
///     let required_extensions = vec![
///         "some_required_extension",
///     ];
///
///     let instance = Instance::new(&required_extensions);
///
pub struct Instance {
    /// The Ash Vulkan library entrypoint.
    pub ash: ash::Instance,

    /// The Debug entrypoint, used to set debug names for vulkan objects.
    pub debug: DebugUtils,

    /// The layers applied to this vulkan instance
    #[allow(unused)]
    layers: Vec<String>,

    /// The instance's debug messenger
    debug_messenger: vk::DebugUtilsMessengerEXT,

    /// The vulkan function loader
    #[allow(unused)]
    pub entry: Entry,
}
