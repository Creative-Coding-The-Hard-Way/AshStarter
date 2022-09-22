use crate::{graphics::vulkan_api::VulkanError, logging::PrettyList};

/// Check that each of the provided extensions is available on the current
/// platform.
pub fn check_extensions(
    entry: &ash::Entry,
    required_extensions: &[String],
) -> Result<(), VulkanError> {
    let missing = missing_extensions(entry, required_extensions)?;
    if !missing.is_empty() {
        Err(VulkanError::RequiredExtensionsNotFound(missing))
    } else {
        Ok(())
    }
}

/// Get a list of all extensions which are required but not available for this
/// vulkan instance.
fn missing_extensions(
    entry: &ash::Entry,
    required_extensions: &[String],
) -> Result<Vec<String>, VulkanError> {
    let available_extensions = entry
        .enumerate_instance_extension_properties()
        .map_err(VulkanError::UnableToListAvailableExtensions)?;

    let available_names: Vec<String> = available_extensions
        .iter()
        .map(|ext| {
            String::from_utf8(
                ext.extension_name.iter().map(|c| *c as u8).collect(),
            )
        })
        // only accept valid utf-8 extension names
        .filter_map(|item| item.ok())
        .collect();

    log::debug!("Available extensions: {}", PrettyList(&available_names));

    Ok(required_extensions
        .iter()
        .cloned()
        .filter(|name| !available_names.iter().any(|item| item.contains(name)))
        .collect())
}
