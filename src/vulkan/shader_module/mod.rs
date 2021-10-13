mod shader_module;

use crate::vulkan::RenderDevice;

use ::{ash::vk, std::sync::Arc, thiserror::Error};

#[derive(Debug, Error)]
pub enum ShaderModuleError {
    #[error(
        "The shader's source bytes must be evenly divisible into u32 words"
    )]
    InvalidSourceLengthInShaderSPIRV,

    #[error("Improper bytes found in compiled SPIRV shader module source")]
    InvalidBytesInShaderSPIRV(#[source] core::array::TryFromSliceError),

    #[error("Unable to create the shader module")]
    UnableToCreateShaderModule(#[source] vk::Result),
}

/// An owned vk::ShaderModule which is destroyed automatically when it falls
/// out of scope.
pub struct ShaderModule {
    pub raw: vk::ShaderModule,
    pub vk_dev: Arc<RenderDevice>,
}
