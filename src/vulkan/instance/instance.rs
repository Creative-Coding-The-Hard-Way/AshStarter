use ash::{extensions::ext::DebugUtils, vk, Entry};

use crate::{
    markdown::MdList,
    vulkan::{
        ffi::to_os_ptrs,
        instance::{debug_callback, extensions, layers, InstanceError},
    },
};

/// The Instance struct holds the ash entry and ash library handle along with
/// the debug callback.
///
/// # Example
///
///     use ccthw::vulkan::Instance;
///
///     // Typically the required extensions come from the window system.
///     let required_extensions = vec![
///         String::from("some_required_extension"),
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

    pub fn create_logical_device(
        &self,
        physical_device: &vk::PhysicalDevice,
        physical_device_features: vk::PhysicalDeviceFeatures,
        physical_device_extensions: &[String],
        queue_create_infos: &[vk::DeviceQueueCreateInfo],
    ) -> Result<ash::Device, InstanceError> {
        let (_c_names, layer_name_ptrs) = unsafe { to_os_ptrs(&self.layers) };
        let (_c_ext_names, ext_name_ptrs) =
            unsafe { to_os_ptrs(physical_device_extensions) };

        let create_info = vk::DeviceCreateInfo {
            queue_create_info_count: queue_create_infos.len() as u32,
            p_queue_create_infos: queue_create_infos.as_ptr(),
            p_enabled_features: &physical_device_features,
            pp_enabled_layer_names: layer_name_ptrs.as_ptr(),
            enabled_layer_count: layer_name_ptrs.len() as u32,
            pp_enabled_extension_names: ext_name_ptrs.as_ptr(),
            enabled_extension_count: physical_device_extensions.len() as u32,
            ..Default::default()
        };

        unsafe {
            self.ash
                .create_device(*physical_device, &create_info, None)
                .map_err(InstanceError::UnableToCreateLogicalDevice)
        }
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
        "VK_LAYER_LUNARG_monitor".to_owned(),
        //"VK_LAYER_LUNARG_api_dump".to_owned(),
    ]
}

/// Create a Vulkan instance with the required extensions.
fn create_instance(
    required_extensions: &Vec<String>,
) -> Result<(ash::Instance, Entry), InstanceError> {
    use std::ffi::CString;

    let entry = Entry::linked();

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
            .map_err(InstanceError::UnableToCreateInstance)?
    };

    Ok((instance, entry))
}
