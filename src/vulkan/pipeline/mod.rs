mod pipeline;
mod pipeline_layout;
mod shader_module;

use crate::vulkan::RenderDevice;

use ::{ash::vk, std::sync::Arc, thiserror::Error};

#[derive(Debug, Error)]
pub enum PipelineError {
    #[error(
        "The shader's source bytes must be evenly divisible into u32 words"
    )]
    InvalidSourceLengthInShaderSPIRV,

    #[error("Improper bytes found in compiled SPIRV shader module source")]
    InvalidBytesInShaderSPIRV(#[source] core::array::TryFromSliceError),

    #[error("Unable to create the shader module")]
    UnableToCreateShaderModule(#[source] vk::Result),

    #[error("Unable to create the pipeline layout")]
    UnableToCreatePipelineLayout(#[source] vk::Result),

    #[error("Unable to create graphics pipeline")]
    UnableToCreateGraphicsPipeline(#[source] vk::Result),
}

/// An owned Pipeline which is destroyed automatically when it's dropped.
pub struct Pipeline {
    pub raw: vk::Pipeline,
    pub bind_point: vk::PipelineBindPoint,
    pub vk_dev: Arc<RenderDevice>,
}

/// An owned Pipeline Layout which is destroyed automatically when it's dropped.
pub struct PipelineLayout {
    pub raw: vk::PipelineLayout,
    pub vk_dev: Arc<RenderDevice>,
}

/// An owned vk::ShaderModule which is destroyed automatically when it falls
/// out of scope.
pub struct ShaderModule {
    pub raw: vk::ShaderModule,
    pub vk_dev: Arc<RenderDevice>,
}
