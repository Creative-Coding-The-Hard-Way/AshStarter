mod physical_device;
mod queue;
mod queue_family_indices;
mod render_device;
mod swapchain;
mod vulkan_debug_name;

use crate::vulkan::{
    errors::{InstanceError, WindowSurfaceError},
    Instance, WindowSurface,
};

use ash::{extensions::khr, vk};
use thiserror::Error;

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

/// Types which implement this trait can be assigned a debug name in the Vulkan
/// debug callback logs.
pub trait VulkanDebugName<T>
where
    T: vk::Handle + Copy,
{
    fn type_and_handle(&self) -> (vk::ObjectType, T);
}

/// This struct bundles a Vulkan queue with related data for easy tracking.
#[derive(Debug, Clone, Copy)]
pub struct GpuQueue {
    pub queue: vk::Queue,
    pub family_id: u32,
    pub index: u32,
}

/// All swapchain-related resources - things which need replaced when the
/// swapchain is rebuilt.
pub struct Swapchain {
    /// The swapchain extension function loader provided by the ash library.
    pub loader: khr::Swapchain,

    /// The Vulkan SwapchainKHR used for most swapchain operations.
    pub khr: vk::SwapchainKHR,

    /// The array of image views for this swapchain's images.
    pub image_views: Vec<vk::ImageView>,

    /// The image format for this swapchain's images.
    pub format: vk::Format,

    /// The color space used for this swapchain's images.
    pub color_space: vk::ColorSpaceKHR,

    /// The hardware pixel extent for this swapchain's images.
    pub extent: vk::Extent2D,
}

/// The render device holds the core Vulkan state and devices which are used
/// by all parts of the application.
pub struct RenderDevice {
    /// The physical device used by this application.
    #[allow(unused)]
    pub physical_device: vk::PhysicalDevice,

    /// The Vulkan logical device used to issue commands to the physical device.
    pub logical_device: ash::Device,

    /// The GPU queue used to submit graphics commands.
    pub graphics_queue: GpuQueue,

    /// The GPU queue used to submit presentation commands.
    pub present_queue: GpuQueue,

    /// The window's swapchain and related resources.
    pub swapchain: Option<Swapchain>,

    /// The Vulkan presentation surface for the current window.
    pub window_surface: WindowSurface,

    /// The Vulkan library instance.
    pub instance: Instance,
}

/// This struct holds all of the queue indices required by this application.
struct QueueFamilyIndices {
    /// the index for the graphics queue
    graphics_family_index: u32,

    /// the index for the presentation queue
    present_family_index: u32,
}
