use std::ffi::c_void;

use ash::{extensions::ext::DebugUtils, vk};

pub use self::physical_device_features::{
    ArePhysicalDeviceFeaturesSuitableFn, PhysicalDeviceFeatures,
};
use crate::{
    graphics::vulkan_api::{ffi::to_os_ptrs, VulkanError},
    logging::PrettyList,
};

mod api;
mod debug_callback;
mod extensions;
mod layers;
mod physical_device_features;

/// The Vulkan library instance.
pub struct Instance {
    layers: Vec<String>,
    debug_messenger: Option<vk::DebugUtilsMessengerEXT>,
    debug: Option<DebugUtils>,
    entry: ash::Entry,
    ash: ash::Instance,
}

impl Instance {
    pub fn new(required_extensions: &[String]) -> Result<Self, VulkanError> {
        let (ash, entry) = create_instance(required_extensions)?;

        let (debug, debug_messenger) = {
            if cfg!(debug_assertions) {
                let (debug, debug_messenger) =
                    debug_callback::create_debug_logger(&entry, &ash)?;
                (Some(debug), Some(debug_messenger))
            } else {
                (None, None)
            }
        };

        Ok(Self {
            layers: debug_layers(),
            debug_messenger,
            debug,
            entry,
            ash,
        })
    }

    /// Get the raw Vulkan instance handle.
    ///
    /// # Safety
    ///
    /// Unsafe because ownership is *not* transferred. It is up to the caller
    /// to make sure that however the handle is used, it does not outlive this
    /// struct.
    pub unsafe fn vulkan_instance_handle(&self) -> vk::Instance {
        self.ash.handle()
    }

    /// Create the ash extension loader for a KHR Surface.
    pub fn create_surface_loader(&self) -> ash::extensions::khr::Surface {
        ash::extensions::khr::Surface::new(&self.entry, &self.ash)
    }

    /// Create the ash extension loader for a KHR Swapchain.
    pub fn create_swapchain_loader(
        &self,
        logical_device: &ash::Device,
    ) -> ash::extensions::khr::Swapchain {
        ash::extensions::khr::Swapchain::new(&self.ash, logical_device)
    }

    /// Create the logical device with the requested queues.
    pub fn create_logical_device(
        &self,
        physical_device: &vk::PhysicalDevice,
        physical_device_extensions: &[String],
        queue_create_infos: &[vk::DeviceQueueCreateInfo],
        mut physical_device_features: PhysicalDeviceFeatures,
    ) -> Result<ash::Device, VulkanError> {
        let (_c_layer_names, layer_name_ptrs) =
            unsafe { to_os_ptrs(&self.layers) };
        let (_c_ext_names, ext_name_ptrs) =
            unsafe { to_os_ptrs(physical_device_extensions) };

        let physical_device_features_v2 = vk::PhysicalDeviceFeatures2 {
            p_next: &mut physical_device_features.descriptor_indexing_features
                as *mut vk::PhysicalDeviceDescriptorIndexingFeatures
                as *mut c_void,
            features: physical_device_features.features,
            ..Default::default()
        };
        let create_info = vk::DeviceCreateInfo {
            p_next: &physical_device_features_v2
                as *const vk::PhysicalDeviceFeatures2
                as *const c_void,
            queue_create_info_count: queue_create_infos.len() as u32,
            p_queue_create_infos: queue_create_infos.as_ptr(),
            p_enabled_features: std::ptr::null(),
            pp_enabled_layer_names: layer_name_ptrs.as_ptr(),
            enabled_layer_count: layer_name_ptrs.len() as u32,
            pp_enabled_extension_names: ext_name_ptrs.as_ptr(),
            enabled_extension_count: ext_name_ptrs.len() as u32,
            ..Default::default()
        };

        unsafe {
            self.ash
                .create_device(*physical_device, &create_info, None)
                .map_err(VulkanError::UnableToCreateLogicalDevice)
        }
    }

    #[cfg(debug_assertions)]
    /// Set the debug name for an object owned by the provided logical device.
    ///
    /// Logs a warning if the name cannot be set for any reason.
    pub fn debug_utils_set_object_name(
        &self,
        logical_device: &ash::Device,
        name_info: &vk::DebugUtilsObjectNameInfoEXT,
    ) {
        let result = unsafe {
            self.debug
                .as_ref()
                .unwrap()
                .debug_utils_set_object_name(logical_device.handle(), name_info)
        };
        if result.is_err() {
            log::warn!(
                "Unable to set debug name for device! {:#?} {:#?}",
                name_info,
                result.err().unwrap()
            );
        }
    }

    #[cfg(not(debug_assertions))]
    pub fn debug_utils_set_object_name(
        &self,
        _logical_device: &ash::Device,
        _name_info: &vk::DebugUtilsObjectNameInfoEXT,
    ) {
        // no-op
    }
}

impl Drop for Instance {
    fn drop(&mut self) {
        unsafe {
            if cfg!(debug_assertions) {
                self.debug.take().unwrap().destroy_debug_utils_messenger(
                    self.debug_messenger.take().unwrap(),
                    None,
                )
            }
            self.ash.destroy_instance(None);
        }
    }
}

/// The set of all debug layers used by this application.
fn debug_layers() -> Vec<String> {
    if cfg!(debug_assertions) {
        vec![
            "VK_LAYER_KHRONOS_validation".to_owned(),
            "VK_LAYER_LUNARG_monitor".to_owned(),
        ]
    } else {
        vec![]
    }
}

fn create_instance(
    required_extensions: &[String],
) -> Result<(ash::Instance, ash::Entry), VulkanError> {
    use std::ffi::CString;

    let entry = ash::Entry::linked();

    let mut required_with_debug = Vec::new();
    required_with_debug.extend_from_slice(required_extensions);
    if cfg!(debug_assertions) {
        required_with_debug.push(
            DebugUtils::name()
                .to_str()
                .map_err(VulkanError::InvalidDebugLayerName)?
                .to_owned(),
        );
    }

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
