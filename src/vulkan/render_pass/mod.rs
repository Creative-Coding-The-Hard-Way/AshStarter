mod render_pass;

use crate::vulkan::RenderDevice;

use ::{ash::vk, std::sync::Arc, thiserror::Error};

#[derive(Debug, Error)]
pub enum RenderPassError {
    #[error("Unable to create a new render pass")]
    UnableToCreateRenderPass(#[source] vk::Result),
}

/// An owned Vulkan RenderPass which automatically destroys itself when dropped.
pub struct RenderPass {
    pub raw: vk::RenderPass,
    pub vk_dev: Arc<RenderDevice>,
}
