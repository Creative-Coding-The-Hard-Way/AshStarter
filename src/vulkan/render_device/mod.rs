mod physical_device;
mod queue;
mod queue_family_indices;
mod render_device;

use crate::vulkan::{errors::InstanceError, Instance, WindowSurface};

use ash::vk;
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

/// The render device holds the core Vulkan state and devices which are used
/// by all parts of the application.
pub struct RenderDevice {
    /// The physical device used by this application.
    #[allow(unused)]
    physical_device: vk::PhysicalDevice,

    /// The Vulkan logical device used to issue commands to the physical device.
    logical_device: ash::Device,

    /// The gpu command queues used by the application for rendering,
    /// presentation, and compute, operations.
    graphics_queue: GpuQueue,
    present_queue: GpuQueue,

    /// The Vulkan presentation surface for the current window.
    #[allow(unused)]
    window_surface: WindowSurface,

    /// The Vulkan library instance.
    instance: Instance,
}

/// This struct holds all of the queue indices required by this application.
struct QueueFamilyIndices {
    /// the index for the graphics queue
    graphics_family_index: u32,

    /// the index for the presentation queue
    present_family_index: u32,
}
