//! This module defines functions for selecting a physical device with the
//! features required by this application.

use ash::vk;

use crate::vulkan::{
    render_device::{PhysicalDeviceError, QueueFamilyIndices},
    WindowSurface,
};

/// Return the set of required device features for this application.
pub fn required_features() -> vk::PhysicalDeviceFeatures {
    vk::PhysicalDeviceFeatures {
        geometry_shader: 1,
        ..Default::default()
    }
}

/// Return the set of required device extensions for this application
pub fn required_extensions() -> Vec<String> {
    let swapchain = ash::extensions::khr::Swapchain::name()
        .to_owned()
        .into_string()
        .unwrap();
    vec![swapchain]
}

/// Pick a physical device based on suitability criteria.
pub fn find_optimal(
    ash: &ash::Instance,
    window_surface: &WindowSurface,
) -> Result<vk::PhysicalDevice, PhysicalDeviceError> {
    let physical_devices = unsafe {
        ash.enumerate_physical_devices()
            .map_err(PhysicalDeviceError::UnableToEnumerateDevices)?
    };
    let physical_device = physical_devices
        .iter()
        .find(|device| is_device_suitable(ash, device, window_surface))
        .ok_or(PhysicalDeviceError::NoSuitableDeviceFound)?;
    Ok(*physical_device)
}

/// Return true when the device is suitable for this application.
fn is_device_suitable(
    ash: &ash::Instance,
    physical_device: &vk::PhysicalDevice,
    window_surface: &WindowSurface,
) -> bool {
    let queues_supported =
        QueueFamilyIndices::find(ash, physical_device, window_surface).is_ok();

    let features =
        unsafe { ash.get_physical_device_features(*physical_device) };

    let extensions_supported = check_required_extensions(ash, &physical_device);

    let format_available = if extensions_supported {
        unsafe { !window_surface.supported_formats(physical_device).is_empty() }
    } else {
        false
    };

    let presentation_mode_available = if extensions_supported {
        unsafe {
            !window_surface
                .supported_presentation_modes(physical_device)
                .is_empty()
        }
    } else {
        false
    };

    queues_supported
        && extensions_supported
        && format_available
        && presentation_mode_available
        && features.geometry_shader == vk::TRUE
}

/// Fetch a vector of all missing device extensions based on the required
/// extensions.
fn check_required_extensions(
    ash: &ash::Instance,
    physical_device: &vk::PhysicalDevice,
) -> bool {
    let extensions = unsafe {
        ash.enumerate_device_extension_properties(*physical_device)
            .unwrap_or_else(|_| vec![])
    };
    extensions
        .iter()
        .map(|extension| {
            String::from_utf8(
                extension.extension_name.iter().map(|c| *c as u8).collect(),
            )
        })
        .filter(|item| item.is_ok()) // only take valid utf8 extension names
        .map(|item| item.unwrap())
        .filter(|name| required_extensions().contains(name))
        .collect::<Vec<String>>()
        .is_empty()
}
