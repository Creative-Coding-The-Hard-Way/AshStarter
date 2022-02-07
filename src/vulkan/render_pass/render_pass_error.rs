use ::{ash::vk, thiserror::Error};

#[derive(Debug, Error)]
pub enum RenderPassError {
    #[error("Unable to create a new render pass")]
    UnableToCreateRenderPass(#[source] vk::Result),
}
