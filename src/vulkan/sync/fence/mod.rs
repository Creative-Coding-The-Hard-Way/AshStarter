mod fence;

use crate::vulkan::RenderDevice;

use ::{ash::vk, std::sync::Arc, thiserror::Error};

#[derive(Debug, Error)]
pub enum FenceError {
    #[error("Unable to create a new fence")]
    UnableToCreateFence(#[source] vk::Result),

    #[error("Error while waiting for fence")]
    UnexpectedWaitError(#[source] vk::Result),

    #[error("Error while resetting fence")]
    UnexpectedResetError(#[source] vk::Result),
}

/// An owned Vulkan fence object which is automatically destroyed when dropped.
pub struct Fence {
    /// The raw fence handle.
    pub raw: vk::Fence,

    /// The device which created the fence.
    pub vk_dev: Arc<RenderDevice>,
}
