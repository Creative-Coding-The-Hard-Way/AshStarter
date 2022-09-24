use std::str::Utf8Error;

use ash::vk;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum VulkanError {
    #[error(transparent)]
    InvalidDebugLayerName(#[from] Utf8Error),

    #[error("The following extensions are required but unavailable {:?}", .0)]
    RequiredExtensionsNotFound(Vec<String>),

    #[error("Unable to get the available Vulkan extensions {:?}", .0)]
    UnableToListAvailableExtensions(#[source] vk::Result),

    #[error("The following layers are required but unavailable {:?}", .0)]
    RequiredLayersNotFound(Vec<String>),

    #[error("Unable to get the available Vulkan layers {:?}", .0)]
    UnableToListAvailableLayers(#[source] vk::Result),

    #[error("Unable to create a Vulkan instance {:?}", .0)]
    UnableToCreateInstance(#[source] vk::Result),

    #[error("Unable to create the Vulkan debug messenger {:?}", .0)]
    UnableToCreateDebugMessenger(#[source] vk::Result),

    #[error("Unable to enumerate physical devices {:?}", .0)]
    UnableToEnumeratePhysicalDevices(#[source] vk::Result),

    #[error("No suitable physical device found")]
    NoSuitableDeviceFound,

    #[error("Unable to the queue family for physical device support {:?}", .0)]
    UnableToCheckPhysicalDeviceSupport(#[source] vk::Result),

    #[error("Unable to find a queue family for submitting graphics commands")]
    UnableToFindGraphicsQueue,

    #[error(
        "Unable to find a queue family for submitting presentation commands"
    )]
    UnableToFindPresentQueue,

    #[error("Unable to create the Vulkan logical device {:?}", .0)]
    UnableToCreateLogicalDevice(#[source] vk::Result),

    #[error("Unable to get surface capabilities for the physical device {:?}", .0)]
    UnableToGetPhysicalDeviceSurfaceCapabilities(#[source] vk::Result),

    #[error("Unable to create swapchain {:?}", .0)]
    UnableToCreateSwapchain(#[source] vk::Result),

    #[error("Unable to get swapchain images {:?}", .0)]
    UnableToGetSwapchainImages(#[source] vk::Result),

    #[error("Unable to create a Vulkan ImageView {:?}", .0)]
    UnableToCreateImageView(#[source] vk::Result),

    #[error("Unable to create a Vulkan Fence {:?}", .0)]
    UnableToCreateFence(#[source] vk::Result),

    #[error("Unexpected error while waiting for a Vulkan fence {:?}", .0)]
    UnexpectedFenceWaitError(#[source] vk::Result),

    #[error("Unexpected error while resetting Vulkan fences {:?}", .0)]
    UnexpectedFenceResetError(#[source] vk::Result),

    #[error("Unable to create a Vulkan semaphore {:?}", .0)]
    UnableToCreateSemaphore(#[source] vk::Result),

    #[error("Unable to acquire Swapchain image {:?}", .0)]
    UnableToAcquireSwapchainImage(#[source] vk::Result),

    #[error("Unable to present Swapchain image {:?}", .0)]
    UnableToPresentSwapchainImage(#[source] vk::Result),

    #[error("Error while waiting for the Vulkan device to idle {:?}", .0)]
    UnableToWaitForDeviceToIdle(#[source] vk::Result),

    #[error("Unable to create Vulkan render pass {:?}", .0)]
    UnableToCreateRenderPass(#[source] vk::Result),

    #[error("Unable to create Vulkan framebuffer {:?}", .0)]
    UnableToCreateFramebuffer(#[source] vk::Result),

    #[error("Unable to create Vulkan command pool {:?}", .0)]
    UnableToCreateCommandPool(#[source] vk::Result),

    #[error("Unable to allocate Vulkan command buffers {:?}", .0)]
    UnableToAllocateCommandBuffers(#[source] vk::Result),

    #[error("Unable to reset Vulkan command pool {:?}", .0)]
    UnableToResetCommandPool(#[source] vk::Result),

    #[error("Unable to begin Vulkan command buffer {:?}", .0)]
    UnableToBeginCommandBuffer(#[source] vk::Result),

    #[error("Unable to end Vulkan command buffer {:?}", .0)]
    UnableToEndCommandBuffer(#[source] vk::Result),

    #[error("Unable to submit graphics commands {:?}", .0)]
    UnableToSubmitGraphicsCommands(#[source] vk::Result),

    #[error("Unable to allocate Vulkan device memory {:?}", .0)]
    UnableToAllocateDeviceMemory(#[source] vk::Result),

    #[error("Unable to map Vulkan device memory to a host-accessible pointer {:?}", .0)]
    UnableToMapDeviceMemory(#[source] vk::Result),

    #[error(
        "Attempted to access device memory from the host without mapping first"
    )]
    DeviceMemoryIsNotMapped,

    #[error("Device memory is not aligned as {:?}", .0)]
    DeviceMemoryIsNotAlignedForType(String),

    #[error("Unable to find a memory type for {:#?} with requirements {:#?}", .0, .1)]
    MemoryTypeNotFound(vk::MemoryPropertyFlags, vk::MemoryRequirements),

    #[error("Unable to create buffer with size {:#?}b flags {:#?}. Error {:#?}", .0, .1, .2)]
    UnableToCreateBuffer(u64, vk::BufferUsageFlags, #[source] vk::Result),

    #[error("Unable to bind Vulkan memory to a buffer {:#?}", .0)]
    UnableToBindBufferMemory(#[source] vk::Result),

    #[error("Unable to flush changes to mapped Vulkan memory {:#?}", .0)]
    UnableToFlushMappedMemoryRanges(#[source] vk::Result),
}
