use crate::{
    graphics::vulkan_api::{ffi::to_os_ptrs, VulkanError},
    logging::PrettyList,
};

use ash::{extensions::ext::DebugUtils, vk};

mod debug_callback;
mod extensions;
mod layers;

/// The Vulkan library instance.
pub struct Instance {
    debug_messenger: vk::DebugUtilsMessengerEXT,
    debug: DebugUtils,
    _entry: ash::Entry,
    ash: ash::Instance,
}

impl Instance {
    pub fn new(required_extensions: &[String]) -> Result<Self, VulkanError> {
        let (ash, entry) = create_instance(required_extensions)?;
        let (debug, debug_messenger) =
            debug_callback::create_debug_logger(&entry, &ash)?;
        Ok(Self {
            debug_messenger,
            debug,
            _entry: entry,
            ash,
        })
    }
}

impl Drop for Instance {
    fn drop(&mut self) {
        unsafe {
            self.debug
                .destroy_debug_utils_messenger(self.debug_messenger, None);
            self.ash.destroy_instance(None);
        }
    }
}

fn debug_layers() -> Vec<String> {
    vec![
        "VK_LAYER_KHRONOS_validation".to_owned(),
        "VK_LAYER_LUNARG_monitor".to_owned(),
    ]
}

fn create_instance(
    required_extensions: &[String],
) -> Result<(ash::Instance, ash::Entry), VulkanError> {
    use std::ffi::CString;

    let entry = ash::Entry::linked();

    let mut required_with_debug = Vec::new();
    required_with_debug.extend_from_slice(required_extensions);
    required_with_debug.push(
        DebugUtils::name()
            .to_str()
            .map_err(VulkanError::InvalidDebugLayerName)?
            .to_owned(),
    );

    extensions::check_extensions(&entry, &required_with_debug)?;
    layers::check_layers(&entry, &debug_layers())?;

    log::debug!("Required Extensions: {}", PrettyList(&required_with_debug));

    let app_name = CString::new("ash starter").unwrap();
    let engine_name = CString::new("no engine").unwrap();

    let app_info = vk::ApplicationInfo {
        p_engine_name: engine_name.as_ptr(),
        p_application_name: app_name.as_ptr(),
        application_version: vk::make_api_version(0, 1, 0, 0),
        engine_version: vk::make_api_version(0, 1, 0, 0),
        api_version: vk::make_api_version(0, 1, 3, 0),
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
            .map_err(VulkanError::UnableToCreateInstance)?
    };

    Ok((instance, entry))
}
