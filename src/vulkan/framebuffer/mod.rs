mod framebuffer;

use ::{ash::vk, std::sync::Arc, thiserror::Error};

use crate::vulkan::{errors::VulkanDebugError, RenderDevice};

#[derive(Debug, Error)]
pub enum FramebufferError {
    #[error("Unable to create the framebuffer")]
    UnableToCreateFramebuffer(#[source] vk::Result),

    #[error("Unable to create a framebuffer for swapchain image {}", .0)]
    UnableToCreateSwapchainFramebuffer(usize, #[source] vk::Result),

    #[error(transparent)]
    UnexpectedVulkanDebugError(#[from] VulkanDebugError),
}

/// An owned Vulkan framebuffer which is automatically destroyed when it is
/// dropped.
pub struct Framebuffer {
    pub raw: vk::Framebuffer,
    pub vk_dev: Arc<RenderDevice>,
}
