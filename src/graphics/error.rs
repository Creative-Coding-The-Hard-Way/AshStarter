use thiserror::Error;

use crate::graphics::vulkan_api::VulkanError;

#[derive(Error, Debug)]
pub enum GraphicsError {
    #[error(transparent)]
    VulkanError(#[from] VulkanError),

    #[error(
        "Unable to acquire Frame resources! Did you forget to return a Frame?"
    )]
    FrameMissing,

    #[error(
        "Unable to take unique ownership of the old swapchain when rebuilding! Did you forget to drop a resource before rebuilding?"
    )]
    SwapchainOwnershipIsNotUnique,
}
