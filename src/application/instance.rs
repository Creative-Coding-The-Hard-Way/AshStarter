use anyhow::{bail, Result};
use ash::{version::EntryV1_0, vk, Entry, Instance};
use std::ffi::CString;

/// Create a Vulkan instance with the required extensions.
/// Yields an error if any required extensions are unavailable.
pub fn create_instance(
    required_extensions: &Vec<String>,
) -> Result<(Instance, Entry)> {
    let entry = Entry::new()?;

    check_extensions(&entry, required_extensions)?;

    let cstr_required_extensions = required_extensions
        .iter()
        .cloned()
        .map(|str| CString::new(str).unwrap())
        .collect::<Vec<_>>();
    let required_extensions_raw = cstr_required_extensions
        .iter()
        .map(|cstr| cstr.as_ptr())
        .collect::<Vec<_>>();

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
        .enabled_extension_names(&required_extensions_raw);

    let instance = unsafe { entry.create_instance(&create_info, None)? };

    Ok((instance, entry))
}

/// Check that all required extensions are available or else bail.
fn check_extensions(
    entry: &Entry,
    required_extensions: &Vec<String>,
) -> Result<()> {
    let missing = missing_extensions(entry, required_extensions)?;
    if !missing.is_empty() {
        bail!("Some required extensions were not found!\n{:?}", missing);
    }
    Ok(())
}

/// Get a list of all extensions which are not available on this platform.
fn missing_extensions(
    entry: &Entry,
    required_extensions: &Vec<String>,
) -> Result<Vec<String>> {
    let available_extensions =
        entry.enumerate_instance_extension_properties()?;

    let available_names: Vec<String> = available_extensions
        .iter()
        .map(|ext| {
            String::from_utf8(
                ext.extension_name.iter().map(|c| *c as u8).collect(),
            )
            .unwrap()
        })
        .collect();

    log::info!("Available extensions {}", available_names.join("\n"));

    Ok(required_extensions
        .iter()
        .cloned()
        .filter(|name| available_names.contains(name))
        .collect())
}
