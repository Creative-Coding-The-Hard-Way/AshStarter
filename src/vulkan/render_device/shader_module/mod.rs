use super::ShaderModuleError;

use crate::vulkan::RenderDevice;

use ash::{version::DeviceV1_0, vk};
use std::convert::TryInto;

impl RenderDevice {
    /// Create a new shader module using the provided SPIRV source.
    ///
    /// The caller is responsible for destroying the shader module before the
    /// application exits.
    pub fn create_shader_module(
        &self,
        source: &'static [u8],
    ) -> Result<vk::ShaderModule, ShaderModuleError> {
        let source_u32 = copy_to_u32(source)?;
        let create_info = vk::ShaderModuleCreateInfo {
            p_code: source_u32.as_ptr(),
            code_size: source_u32.len() * std::mem::size_of::<u32>(),
            ..Default::default()
        };
        let shader_module = unsafe {
            self.logical_device
                .create_shader_module(&create_info, None)
                .map_err(ShaderModuleError::UnableToCreateShaderModule)?
        };
        Ok(shader_module)
    }
}

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
fn copy_to_u32(bytes: &'static [u8]) -> Result<Vec<u32>, ShaderModuleError> {
    const U32_SIZE: usize = std::mem::size_of::<u32>();
    if bytes.len() % U32_SIZE != 0 {
        return Err(ShaderModuleError::InvalidSourceLengthInShaderSPIRV);
    }

    let mut buffer: Vec<u32> = vec![];
    let mut input: &[u8] = &bytes;
    while input.len() > 0 {
        let (int_slice, rest) = input.split_at(U32_SIZE);
        input = rest;
        let word = u32::from_le_bytes(
            int_slice
                .try_into()
                .map_err(ShaderModuleError::InvalidBytesInShaderSPIRV)?,
        );
        buffer.push(word);
    }

    Ok(buffer)
}
