use std::sync::Arc;

use ash::vk;

use crate::vulkan::{
    errors::VulkanDebugError, pipeline::PipelineError, RenderDevice,
    VulkanDebug,
};

const DEFAULT_ENTRY_POINT: &'static [u8] = b"main\0";

/// An owned vk::ShaderModule which is destroyed automatically when it falls
/// out of scope.
pub struct ShaderModule {
    pub raw: vk::ShaderModule,
    pub vk_dev: Arc<RenderDevice>,
}

impl ShaderModule {
    /// Create a new owned shader module.
    pub fn from_spirv(
        vk_dev: Arc<RenderDevice>,
        source: &'static [u8],
    ) -> Result<Self, PipelineError> {
        let source_u32 = Self::copy_to_u32(source)?;
        let create_info = vk::ShaderModuleCreateInfo {
            p_code: source_u32.as_ptr(),
            code_size: source_u32.len() * std::mem::size_of::<u32>(),
            ..Default::default()
        };
        let shader_module = unsafe {
            vk_dev
                .logical_device
                .create_shader_module(&create_info, None)
                .map_err(PipelineError::UnableToCreateShaderModule)?
        };
        Ok(Self {
            raw: shader_module,
            vk_dev,
        })
    }

    /// Get the vulkan stage create info for this shader module.
    ///
    /// Note: assumes "main" entrypoint.
    pub fn stage_create_info(
        &self,
        stage: vk::ShaderStageFlags,
    ) -> vk::PipelineShaderStageCreateInfo {
        vk::PipelineShaderStageCreateInfo {
            stage,
            module: self.raw,
            p_name: DEFAULT_ENTRY_POINT.as_ptr() as *const i8,
            ..Default::default()
        }
    }
}

impl VulkanDebug for ShaderModule {
    fn set_debug_name(
        &self,
        debug_name: impl Into<String>,
    ) -> Result<(), VulkanDebugError> {
        self.vk_dev.name_vulkan_object(
            debug_name,
            vk::ObjectType::SHADER_MODULE,
            self.raw,
        )?;
        Ok(())
    }
}

impl Drop for ShaderModule {
    /// # DANGER
    ///
    /// There is no internal synchronization for this type. Unexpected behavior
    /// can occur if this instance is still in-use by the GPU when it is
    /// dropped.
    fn drop(&mut self) {
        unsafe {
            self.vk_dev
                .logical_device
                .destroy_shader_module(self.raw, None);
        }
    }
}

impl ShaderModule {
    /// Copy a byte slice into a properly-aligned u32 array.
    ///
    /// This is meant to help functions which use `include_bytes!` to load sprv
    /// because Vulkan expects sprv source to be in u32 words but `include_bytes`
    /// imports only u8 bytes.
    ///
    /// A full copy is leveraged to handle endianess issues and to ensure proper
    /// alignment.
    ///
    /// Assumes that data is little endian and will break on other architectures.
    fn copy_to_u32(bytes: &'static [u8]) -> Result<Vec<u32>, PipelineError> {
        use std::convert::TryInto;

        const U32_SIZE: usize = std::mem::size_of::<u32>();
        if bytes.len() % U32_SIZE != 0 {
            return Err(PipelineError::InvalidSourceLengthInShaderSPIRV);
        }

        let mut buffer: Vec<u32> = vec![];
        let mut input: &[u8] = &bytes;
        while input.len() > 0 {
            let (int_slice, rest) = input.split_at(U32_SIZE);
            input = rest;
            let word = u32::from_le_bytes(
                int_slice
                    .try_into()
                    .map_err(PipelineError::InvalidBytesInShaderSPIRV)?,
            );
            buffer.push(word);
        }

        Ok(buffer)
    }
}
