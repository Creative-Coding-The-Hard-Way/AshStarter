use ::{ash::vk, thiserror::Error};

use crate::vulkan::errors::{InstanceError, WindowSurfaceError};

/// This enum represents the errors which can occur while attempting to find
/// a usable physical device for the application.
#[derive(Debug, Error)]
pub enum PhysicalDeviceError {
    #[error("Unable to enumerate physical devices")]
    UnableToEnumerateDevices(#[source] vk::Result),

    #[error("No suitable physical device could be found for this application")]
    NoSuitableDeviceFound,
}

/// This enum represents errors which can occur while attempting to find all of
/// the Vulkan command queues which are required by the application.
#[derive(Debug, Error)]
pub enum QueueSelectionError {
    #[error("Unable to find a suitable graphics queue")]
    UnableToFindGraphicsQueue,

    #[error("Unable to find a suitable presentation queue")]
    UnableToFindPresentQueue,
}

/// This enum represents erros which can occur while working with the abstract
/// render device.
#[derive(Debug, Error)]
pub enum RenderDeviceError {
    #[error("Unexpected physical device error")]
    UnexpectedPhysicalDeviceError(#[from] PhysicalDeviceError),

    #[error("Unexpected queue selection error")]
    UnexpectedQueueSelectionError(#[from] QueueSelectionError),

    #[error("Unexpected Vulkan instance error")]
    UnexpectedInstanceError(#[from] InstanceError),

    #[error("Unable to set debug name, {}, for {:?}", .0, .1)]
    UnableToSetDebugName(String, vk::ObjectType, #[source] vk::Result),
}

#[derive(Debug, Error)]
pub enum SwapchainError {
    #[error("Unexpected window error in the swapchain")]
    UnexpectedWindowError(#[from] WindowSurfaceError),

    #[error("Unable to create the swapchain")]
    UnableToCreateSwapchain(#[source] vk::Result),

    #[error("Unable to get swapchain images")]
    UnableToGetSwapchainImages(#[source] vk::Result),

    #[error("Unable to create a view for swapchain image {}", .0)]
    UnableToCreateSwapchainImageView(usize, #[source] vk::Result),

    #[error("Unexpected render device error")]
    UnexpectedRenderDeviceError(#[from] RenderDeviceError),

    #[error(
        "Unable to drain graphics queue when destroying the old swapchain"
    )]
    UnableToDrainGraphicsQueue(#[source] vk::Result),

    #[error(
        "Unable to drain presentation queue when destroying the old swapchain"
    )]
    UnableToDrainPresentQueue(#[source] vk::Result),

    #[error(
        "Unable to wait for device idle when destroying the old swapchain"
    )]
    UnableToWaitForDeviceIdle(#[source] vk::Result),

    #[error("The swapchain is invalid and needs to be rebuilt")]
    NeedsRebuild,
}
