use std::sync::Arc;

use ash::vk;

use crate::graphics::vulkan_api::{RenderDevice, VulkanError};

/// An owned Vulkan shader module.
pub struct ShaderModule {
    shader_module: vk::ShaderModule,
    render_device: Arc<RenderDevice>,
}

impl ShaderModule {
    /// Build a new shader module using compiled SPIRV shader source code.
    pub fn from_spirv_bytes(
        render_device: Arc<RenderDevice>,
        source_bytes: &[u8],
    ) -> Result<Self, VulkanError> {
        let source_words = Self::copy_to_u32(source_bytes)?;
        let create_info = vk::ShaderModuleCreateInfo {
            p_code: source_words.as_ptr(),
            code_size: source_words.len() * std::mem::size_of::<u32>(),
            ..Default::default()
        };
        // Safe because the Vulkan resource is destroyed when this object is
        // dropped.
        let shader_module =
            unsafe { render_device.create_shader_module(&create_info)? };
        Ok(Self {
            shader_module,
            render_device,
        })
    }

    /// The raw Vulkan shader module handle.
    ///
    /// # Safety
    ///
    /// Unsafe because:
    ///   - ownership is not transferred
    ///   - the caller is responsible for ensuring no copies of the
    ///     vk::ShaderModule handle exist after the ShaderModule has been
    ///     dropped
    pub unsafe fn raw(&self) -> vk::ShaderModule {
        self.shader_module
    }
}

impl Drop for ShaderModule {
    /// # Safety
    ///
    /// The application must ensure that the shader module is not in use when it
    /// is dropped.
    fn drop(&mut self) {
        unsafe { self.render_device.destroy_shader_module(self.shader_module) }
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
    fn copy_to_u32(bytes: &[u8]) -> Result<Vec<u32>, VulkanError> {
        use std::convert::TryInto;
        const U32_SIZE: usize = std::mem::size_of::<u32>();

        if bytes.len() % U32_SIZE != 0 {
            return Err(VulkanError::InvalidSourceLengthInShaderSPIRV);
        }

        let mut buffer: Vec<u32> = vec![];
        let mut input: &[u8] = bytes;
        while !input.is_empty() {
            let (int_slice, rest) = input.split_at(U32_SIZE);
            input = rest;
            let word = u32::from_le_bytes(
                int_slice
                    .try_into()
                    .map_err(VulkanError::InvalidBytesInShaderSPIRV)?,
            );
            buffer.push(word);
        }

        Ok(buffer)
    }
}
