mod debug_callback;
mod extensions;
mod layers;

use crate::{markdown::MdList, vulkan::ffi::to_os_ptrs};
use ash::{
    extensions::ext::DebugUtils,
    version::{EntryV1_0, InstanceV1_0},
    vk, Entry,
};
use std::ffi::CString;
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
    ash: ash::Instance,

    /// The Debug entrypoint, used to set debug names for vulkan objects.
    debug: DebugUtils,

    /// The layers applied to this vulkan instance
    #[allow(unused)]
    layers: Vec<String>,

    /// The instance's debug messenger
    debug_messenger: vk::DebugUtilsMessengerEXT,

    /// The vulkan function loader
    #[allow(unused)]
    entry: Entry,
}

impl Instance {
    /// Create a new ash instance with the required extensions.
    ///
    /// Debug and validation layers are automatically setup along with the
    /// debug callback.
    pub fn new(
        required_extensions: &Vec<String>,
    ) -> Result<Self, InstanceError> {
        let (instance, entry) = create_instance(required_extensions)?;
        let (debug, debug_messenger) =
            debug_callback::create_debug_logger(&entry, &instance)?;
        Ok(Self {
            ash: instance,
            entry,
            debug,
            debug_messenger,
            layers: debug_layers(),
        })
    }
}

impl Drop for Instance {
    /// The owner must ensure that the Instance is only dropped after other
    /// resources which depend on it! There is no internal synchronization.
    fn drop(&mut self) {
        unsafe {
            self.debug
                .destroy_debug_utils_messenger(self.debug_messenger, None);
            self.ash.destroy_instance(None);
        }
    }
}

/// The debug layers required by this application
fn debug_layers() -> Vec<String> {
    vec![
        "VK_LAYER_KHRONOS_validation".to_owned(),
        // "VK_LAYER_LUNARG_api_dump".to_owned(),
    ]
}

/// Create a Vulkan instance with the required extensions.
fn create_instance(
    required_extensions: &Vec<String>,
) -> Result<(ash::Instance, Entry), InstanceError> {
    let entry = Entry::new().map_err(InstanceError::VulkanLoadingError)?;

    let mut required_with_debug = required_extensions.clone();
    required_with_debug.push(
        DebugUtils::name()
            .to_str()
            .map_err(InstanceError::InvalidDebugLayerName)?
            .to_owned(),
    );

    extensions::check_extensions(&entry, &required_with_debug)?;
    layers::check_layers(&entry, &debug_layers())?;

    log::debug!("Required Extensions: {}", MdList(required_extensions));

    let app_name = CString::new("ash starter").unwrap();
    let engine_name = CString::new("no engine").unwrap();

    let app_info = vk::ApplicationInfo {
        p_engine_name: engine_name.as_ptr(),
        p_application_name: app_name.as_ptr(),
        application_version: vk::make_version(1, 0, 0),
        engine_version: vk::make_version(1, 0, 0),
        api_version: vk::make_version(1, 1, 0),
        ..Default::default()
    };

    let (_layer_names, layer_ptrs) = unsafe { to_os_ptrs(&debug_layers()) };
    let (_ext_names, ext_ptrs) = unsafe { to_os_ptrs(&required_with_debug) };

    let create_info = vk::InstanceCreateInfo {
        p_application_info: &app_info,
        pp_enabled_layer_names: layer_ptrs.as_ptr(),
        enabled_layer_count: layer_ptrs.len() as u32,
        pp_enabled_extension_names: ext_ptrs.as_ptr(),
        enabled_extension_count: ext_ptrs.len() as u32,
        ..Default::default()
    };

    let instance = unsafe {
        entry
            .create_instance(&create_info, None)
            .map_err(InstanceError::UnableToCreateInstance)?
    };

    Ok((instance, entry))
}
