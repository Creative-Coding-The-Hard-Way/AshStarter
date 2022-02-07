//! This module defines functions for checking supported Vulkan layers.

use ash::{version::EntryV1_0, Entry};

use crate::{markdown::MdList, vulkan::instance::InstanceError};

/// Check that each of the required layers is available on the current platform.
pub fn check_layers(
    entry: &Entry,
    required_layers: &[String],
) -> Result<(), InstanceError> {
    let missing = missing_layers(entry, required_layers)?;
    if !missing.is_empty() {
        Err(InstanceError::RequiredLayersNotFound(missing))
    } else {
        Ok(())
    }
}

/// Get a list of all layers which are required but not avaialable for this
/// vulkan instance.
fn missing_layers(
    entry: &Entry,
    required_layers: &[String],
) -> Result<Vec<String>, InstanceError> {
    let available_layer_properties = entry
        .enumerate_instance_layer_properties()
        .map_err(InstanceError::UnableToListAvailableLayers)?;

    let available_names: Vec<String> = available_layer_properties
        .iter()
        .map(|layer| {
            String::from_utf8(
                layer.layer_name.iter().map(|c| *c as u8).collect(),
            )
            .unwrap()
        })
        .collect();

    log::debug!("Available layers: {}", MdList(&available_names));

    Ok(required_layers
        .iter()
        .cloned()
        .filter(|name| available_names.contains(name))
        .collect())
}
