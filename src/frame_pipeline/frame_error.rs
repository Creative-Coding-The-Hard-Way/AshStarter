use ::thiserror::Error;

use crate::vulkan::errors::VulkanError;

#[derive(Debug, Error)]
pub enum FrameError {
    #[error("The swapchain needs to be rebuilt")]
    SwapchainNeedsRebuild,

    #[error(transparent)]
    UnexpectedRuntimeError(#[from] anyhow::Error),

    #[error(transparent)]
    UnexpectedVulkanError(#[from] VulkanError),
}
