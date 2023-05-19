use {
    super::raii_wrapper,
    crate::graphics::{vulkan_api::RenderDevice, GraphicsError},
    anyhow::{bail, Context, Result},
    ash::vk,
    std::sync::Arc,
};

raii_wrapper!(
    ShaderModule,
    ShaderModuleCreateInfo,
    SHADER_MODULE,
    create_shader_module,
    destroy_shader_module
);

impl ShaderModule {
    /// Build a shader module from the given source bytes.
    ///
    /// # Params
    ///
    /// * `render_device` - the device used to create Vulkan resources.
    /// * `source_bytes` - the raw SPIRV bytes for a compiled shader.
    ///
    /// # Safety
    ///
    /// Unsafe because:
    ///  - the application must destroy the shader module before exit
    ///  - the shader module can be destroyed once the pipeline using it has
    ///    been created
    pub unsafe fn new_from_bytes(
        render_device: Arc<RenderDevice>,
        source_bytes: &[u8],
    ) -> Result<Self, GraphicsError> {
        let aligned_bytes = Self::copy_to_u32(source_bytes)
            .context("Error transforming SPIRV [u8] bytes to [u32]")?;

        let create_info = vk::ShaderModuleCreateInfo {
            p_code: aligned_bytes.as_ptr(),
            code_size: source_bytes.len(),
            ..Default::default()
        };
        Self::new(render_device, &create_info)
    }

    /// Copy a byte slice into a properly-aligned u32 array.
    ///
    /// This is meant to help functions which use `include_bytes!` to load sprv
    /// because Vulkan expects sprv source to be in u32 words but
    /// `include_bytes` imports only u8 bytes.
    ///
    /// A full copy is leveraged to ensure proper alignment.
    ///
    /// Assumes that data is little endian and will break on other
    /// architectures.
    fn copy_to_u32(bytes: &[u8]) -> Result<Vec<u32>> {
        use std::convert::TryInto;
        const U32_SIZE: usize = std::mem::size_of::<u32>();

        if bytes.len() % U32_SIZE != 0 {
            bail!(
                "Invalid size! Cannot evenly divide the buffer into u32 words!"
            );
        }

        let mut buffer: Vec<u32> = vec![];
        let mut input: &[u8] = bytes;
        while !input.is_empty() {
            let (int_slice, rest) = input.split_at(U32_SIZE);
            input = rest;
            let word =
                u32::from_le_bytes(int_slice.try_into().context(
                    "Erro while copying a u32 word from the buffer",
                )?);
            buffer.push(word);
        }

        Ok(buffer)
    }
}
