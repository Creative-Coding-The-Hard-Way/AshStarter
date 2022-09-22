use ash::vk;

use crate::{
    graphics::vulkan_api::{
        render_device::{QueueFamilies, WindowSurface},
        Instance, VulkanError,
    },
    logging::PrettyList,
};

/// Get the set of required device extensions for this application.
pub fn required_device_extensions() -> Vec<String> {
    let swapchain = ash::extensions::khr::Swapchain::name()
        .to_owned()
        .into_string()
        .unwrap();
    vec![swapchain]
}

pub fn find_optimal_physical_device(
    instance: &Instance,
    window_surface: &WindowSurface,
) -> Result<vk::PhysicalDevice, VulkanError> {
    instance
        .enumerate_physical_devices()?
        .into_iter()
        .find(|device| is_device_suitable(instance, window_surface, device))
        .ok_or(VulkanError::NoSuitableDeviceFound)
}

fn is_device_suitable(
    instance: &Instance,
    window_surface: &WindowSurface,
    physical_device: &vk::PhysicalDevice,
) -> bool {
    if any_missing_extensions(instance, physical_device) {
        return false;
    }

    if QueueFamilies::find_for_physical_device(
        instance,
        window_surface,
        physical_device,
    )
    .is_err()
    {
        log::trace!(
            "Could not find suitable queue families for physical device {:?}",
            physical_device
        );
        return false;
    }

    unsafe {
        if window_surface.supported_formats(physical_device).is_empty() {
            log::trace!(
                "No supported format could be found for physical device {:?}",
                physical_device
            );
            return false;
        }

        if window_surface
            .supported_presentation_modes(physical_device)
            .is_empty()
        {
            log::trace!(
                "No presentation modes could be found for physical device {:?}",
                physical_device
            );
            return false;
        }
    }

    true
}

/// Check that all required device extensions are available.
/// Returns true if there are any required device extensions that are not
/// available.
fn any_missing_extensions(
    instance: &Instance,
    physical_device: &vk::PhysicalDevice,
) -> bool {
    let available_device_extensions: Vec<String> = instance
        .enumerate_device_extension_properties(physical_device)
        .iter()
        .map(|extension| {
            String::from_utf8(
                extension.extension_name.iter().map(|c| *c as u8).collect(),
            )
        })
        .filter(|item| item.is_ok()) // only take valid utf8 extension names
        .map(|item| item.unwrap())
        .collect();

    log::trace!(
        "Available physical device extensions: {}",
        PrettyList(&available_device_extensions),
    );

    log::trace!(
        "Required physical device extensions: {}",
        PrettyList(&required_device_extensions())
    );

    required_device_extensions().iter().any(|required_name| {
        let is_missing = !available_device_extensions
            .iter()
            .any(|name| name.contains(required_name));
        if is_missing {
            log::trace!("Device extension {} is not available", required_name);
        }
        is_missing
    })
}
