use {
    crate::graphics::GraphicsError,
    anyhow::{bail, Context, Result},
    ash::vk,
};

/// Create a new Vulkan descriptor set layout.
///
/// # Params
///
/// * `device` - the Vulkan device used to create resources
/// * `bindings` - the descriptor set layout bindings
///
/// # Safety
///
/// Unsafe because:
///   - the descriptor set layout must be destroyed before exit
pub unsafe fn create_descriptor_set_layout(
    device: &ash::Device,
    bindings: &[vk::DescriptorSetLayoutBinding],
) -> Result<vk::DescriptorSetLayout, GraphicsError> {
    let create_info = vk::DescriptorSetLayoutCreateInfo {
        binding_count: bindings.len() as u32,
        p_bindings: if bindings.is_empty() {
            std::ptr::null()
        } else {
            bindings.as_ptr()
        },
        ..Default::default()
    };
    let layout = device
        .create_descriptor_set_layout(&create_info, None)
        .context("Error creating the descriptor set layout!")?;
    Ok(layout)
}

/// Create a new Vulkan pipeline layout.
///
/// # Params
///
/// * `device` - the Vulkan device used to create resources
/// * `descriptor_set_layouts` - the descriptor set layouts used by the pipeline
/// * `push_constant_ranges` - the push constants used by the pipeline
///
/// # Safety
///
/// Unsafe because:
///   - any descriptor set layouts must live at least as long as the pipeline
///     layout
///   - the pipeline layout must be destroyed before exit
pub unsafe fn create_pipeline_layout(
    device: &ash::Device,
    descriptor_set_layouts: &[vk::DescriptorSetLayout],
    push_constant_ranges: &[vk::PushConstantRange],
) -> Result<vk::PipelineLayout, GraphicsError> {
    let create_info = vk::PipelineLayoutCreateInfo {
        set_layout_count: descriptor_set_layouts.len() as u32,
        p_set_layouts: if descriptor_set_layouts.is_empty() {
            std::ptr::null()
        } else {
            descriptor_set_layouts.as_ptr()
        },
        push_constant_range_count: push_constant_ranges.len() as u32,
        p_push_constant_ranges: if push_constant_ranges.is_empty() {
            std::ptr::null()
        } else {
            push_constant_ranges.as_ptr()
        },
        ..Default::default()
    };
    let layout = device
        .create_pipeline_layout(&create_info, None)
        .context("Error creating a pipeline layout!")?;
    Ok(layout)
}

/// Build a shader module from the given source bytes.
///
/// # Params
///
/// * `device` - the device used to create Vulkan resources.
/// * `source_bytes` - the raw SPIRV bytes for a compiled shader.
///
/// # Safety
///
/// Unsafe because:
///  - the application must destroy the shader module before exit
///  - the shader module can be destroyed once the pipeline using it has been
///    created
pub unsafe fn create_shader_module(
    device: &ash::Device,
    source_bytes: &[u8],
) -> Result<vk::ShaderModule, GraphicsError> {
    let aligned_bytes = copy_to_u32(source_bytes)
        .context("Error transforming SPIRV [u8] bytes to [u32]")?;

    let create_info = vk::ShaderModuleCreateInfo {
        p_code: aligned_bytes.as_ptr(),
        code_size: source_bytes.len(),
        ..Default::default()
    };
    let module = device
        .create_shader_module(&create_info, None)
        .context("Error while creating shader module!")?;
    Ok(module)
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
        bail!("Invalid size! Cannot evenly divide the buffer into u32 words!");
    }

    let mut buffer: Vec<u32> = vec![];
    let mut input: &[u8] = bytes;
    while !input.is_empty() {
        let (int_slice, rest) = input.split_at(U32_SIZE);
        input = rest;
        let word = u32::from_le_bytes(
            int_slice
                .try_into()
                .context("Erro while copying a u32 word from the buffer")?,
        );
        buffer.push(word);
    }

    Ok(buffer)
}
