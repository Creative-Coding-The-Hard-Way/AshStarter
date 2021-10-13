use crate::vulkan::errors::RenderDeviceError;

use ::thiserror::Error;

#[derive(Debug, Error)]
pub enum VulkanDebugError {
    #[error(transparent)]
    UnexpectedRenderDeviceError(#[from] RenderDeviceError),

    #[error(transparent)]
    UnknownRuntimeError(#[from] anyhow::Error),
}

/// Types which implement this trait can set their name in the Vulkan
/// validation layer logs.
pub trait VulkanDebug {
    /// Set the debug name for this resource in Vulkan validation layer logs.
    fn set_debug_name(
        &self,
        debug_name: impl Into<String>,
    ) -> Result<(), VulkanDebugError>;
}
