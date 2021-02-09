//! Functions for creating an instance with extensions.
//!
//! Instances are not managed and must be destroyed by the caller in a Drop
//! implementation.
//!
//! # Example
//!
//! ```
//! let instance = instance::create_instance(
//!     &glfw.get_required_instance_extensions().unwrap()
//! )?;
//! ```

mod extensions;
mod layers;

use anyhow::Result;
use ash::{version::EntryV1_0, vk, Entry, Instance};
use std::{ffi::CString, os::raw::c_char};

/// Create a Vulkan instance with the required extensions.
/// Returns an `Err()` if any required extensions are unavailable.
pub fn create_instance(
    required_extensions: &Vec<String>,
) -> Result<(Instance, Entry)> {
    let entry = Entry::new()?;

    let required_layers = vec![
        "VK_LAYER_KHRONOS_validation".to_owned(),
        // "VK_LAYER_LUNARG_api_dump".to_owned(),
    ];

    extensions::check_extensions(&entry, required_extensions)?;
    layers::check_layers(&entry, &required_layers)?;

    let (_ext_names, ext_name_ptrs) = unsafe { as_ffi(&required_extensions) };
    let (_layer_names, layer_name_ptrs) = unsafe { as_ffi(&required_layers) };

    log::debug!("Required Extensions {:?}", required_extensions);

    let app_name = CString::new("ash starter").unwrap();
    let engine_name = CString::new("no engine").unwrap();

    let app_info = vk::ApplicationInfo::builder()
        .application_name(&app_name)
        .application_version(vk::make_version(1, 0, 0))
        .engine_name(&engine_name)
        .engine_version(vk::make_version(1, 0, 0))
        .api_version(vk::make_version(1, 1, 0));

    let create_info = vk::InstanceCreateInfo::builder()
        .application_info(&app_info)
        .enabled_extension_names(&ext_name_ptrs)
        .enabled_layer_names(&layer_name_ptrs);

    let instance = unsafe { entry.create_instance(&create_info, None)? };

    Ok((instance, entry))
}

/// Build a vector of pointers to c-style strings from a vector of rust strings.
///
/// Unsafe because the returned vector of pointers is only valid while the
/// cstrings are alive.
unsafe fn as_ffi(strings: &Vec<String>) -> (Vec<CString>, Vec<*const c_char>) {
    let cstrings = strings
        .iter()
        .cloned()
        .map(|str| CString::new(str).unwrap())
        .collect::<Vec<_>>();
    let ptrs = cstrings
        .iter()
        .map(|cstr| cstr.as_ptr())
        .collect::<Vec<_>>();
    (cstrings, ptrs)
}
