//! This module provides functions for verifying the available Vulkan
//! extensions.

use ash::Entry;

use crate::{markdown::MdList, vulkan::instance::InstanceError};

/// Check that each of the provided extensions is available on the current
/// platform.
pub fn check_extensions(
    entry: &Entry,
    required_extensions: &Vec<String>,
) -> Result<(), InstanceError> {
    let missing = missing_extensions(entry, required_extensions)?;
    if !missing.is_empty() {
        Err(InstanceError::RequiredExtensionsNotFound(missing))
    } else {
        Ok(())
    }
}

/// Get a list of all extensions which are required but not available for this
/// vulkan instance.
fn missing_extensions(
    entry: &Entry,
    required_extensions: &Vec<String>,
) -> Result<Vec<String>, InstanceError> {
    let available_extensions = entry
        .enumerate_instance_extension_properties()
        .map_err(InstanceError::UnableToListAvailableExtensions)?;

    let available_names: Vec<String> = available_extensions
        .iter()
        .map(|ext| {
            String::from_utf8(
                ext.extension_name.iter().map(|c| *c as u8).collect(),
            )
        })
        // only accept valid utf-8 extension names
        .filter(|item| item.is_ok())
        .map(|item| item.unwrap())
        .collect();

    log::debug!("Available extensions: {}", MdList(&available_names));

    Ok(required_extensions
        .iter()
        .cloned()
        .filter(|name| available_names.contains(name))
        .collect())
}
