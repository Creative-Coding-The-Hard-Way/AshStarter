use {
    crate::graphics::{
        vulkan_api::{raii, RenderDevice},
        GraphicsError,
    },
    ash::vk,
    std::{ffi::CString, sync::Arc},
};

pub unsafe fn create_layouts(
    render_device: Arc<RenderDevice>,
    texture_count: u32,
) -> Result<(raii::DescriptorSetLayout, raii::PipelineLayout), GraphicsError> {
    let descriptor_set_layout = raii::DescriptorSetLayout::new_with_bindings(
        render_device.clone(),
        &[
            vk::DescriptorSetLayoutBinding {
                binding: 0,
                descriptor_type: vk::DescriptorType::STORAGE_BUFFER,
                descriptor_count: 1,
                stage_flags: vk::ShaderStageFlags::VERTEX,
                ..vk::DescriptorSetLayoutBinding::default()
            },
            vk::DescriptorSetLayoutBinding {
                binding: 1,
                descriptor_type: vk::DescriptorType::COMBINED_IMAGE_SAMPLER,
                descriptor_count: texture_count,
                stage_flags: vk::ShaderStageFlags::FRAGMENT,
                ..vk::DescriptorSetLayoutBinding::default()
            },
        ],
    )?;
    let pipeline_layout = raii::PipelineLayout::new_with_layouts_and_ranges(
        render_device,
        &[descriptor_set_layout.raw()],
        &[],
    )?;
    Ok((descriptor_set_layout, pipeline_layout))
}

/// Create the graphics pipeline for this example.
pub unsafe fn create_pipeline(
    render_device: Arc<RenderDevice>,
    vertex_source: &[u8],
    fragment_source: &[u8],
    layout: &raii::PipelineLayout,
    render_pass: &raii::RenderPass,
) -> Result<raii::Pipeline, GraphicsError> {
    let vertex_shader_module = raii::ShaderModule::new_from_bytes(
        render_device.clone(),
        vertex_source,
    )?;
    let fragment_shader_module = raii::ShaderModule::new_from_bytes(
        render_device.clone(),
        fragment_source,
    )?;

    let shader_entry_name = CString::new("main").unwrap();
    let stages = [
        vk::PipelineShaderStageCreateInfo {
            module: vertex_shader_module.raw(),
            stage: vk::ShaderStageFlags::VERTEX,
            p_name: shader_entry_name.as_ptr(),
            ..Default::default()
        },
        vk::PipelineShaderStageCreateInfo {
            module: fragment_shader_module.raw(),
            stage: vk::ShaderStageFlags::FRAGMENT,
            p_name: shader_entry_name.as_ptr(),
            ..Default::default()
        },
    ];
    let vertex_input_state = vk::PipelineVertexInputStateCreateInfo::default();
    let input_assembly = vk::PipelineInputAssemblyStateCreateInfo {
        topology: vk::PrimitiveTopology::TRIANGLE_LIST,
        primitive_restart_enable: vk::FALSE,
        ..Default::default()
    };
    let rasterization_state = vk::PipelineRasterizationStateCreateInfo {
        depth_clamp_enable: vk::FALSE,
        rasterizer_discard_enable: vk::FALSE,
        polygon_mode: vk::PolygonMode::FILL,
        line_width: 1.0,
        cull_mode: vk::CullModeFlags::NONE,
        ..Default::default()
    };
    let multisample_state = vk::PipelineMultisampleStateCreateInfo {
        sample_shading_enable: vk::FALSE,
        rasterization_samples: vk::SampleCountFlags::TYPE_1,
        ..Default::default()
    };
    let color_blend_attachment_states =
        [vk::PipelineColorBlendAttachmentState {
            color_write_mask: vk::ColorComponentFlags::RGBA,
            blend_enable: vk::TRUE,
            src_color_blend_factor: vk::BlendFactor::SRC_ALPHA,
            dst_color_blend_factor: vk::BlendFactor::ONE_MINUS_SRC_ALPHA,
            color_blend_op: vk::BlendOp::ADD,
            src_alpha_blend_factor: vk::BlendFactor::ONE,
            dst_alpha_blend_factor: vk::BlendFactor::ZERO,
            alpha_blend_op: vk::BlendOp::ADD,
        }];
    let color_blend_state = vk::PipelineColorBlendStateCreateInfo {
        attachment_count: color_blend_attachment_states.len() as u32,
        p_attachments: color_blend_attachment_states.as_ptr(),
        ..Default::default()
    };
    let viewports = [vk::Viewport {
        x: 0.0,
        y: 0.0,
        width: 1.0,
        height: 1.0,
        min_depth: 0.0,
        max_depth: 1.0,
    }];
    let scissors = [vk::Rect2D {
        offset: vk::Offset2D { x: 0, y: 0 },
        extent: vk::Extent2D {
            width: 1,
            height: 1,
        },
    }];
    let viewport_state = vk::PipelineViewportStateCreateInfo {
        viewport_count: viewports.len() as u32,
        p_viewports: viewports.as_ptr(),
        scissor_count: scissors.len() as u32,
        p_scissors: scissors.as_ptr(),
        ..Default::default()
    };
    let dynamic_states =
        [vk::DynamicState::VIEWPORT, vk::DynamicState::SCISSOR];
    let dynamic_state = vk::PipelineDynamicStateCreateInfo {
        dynamic_state_count: dynamic_states.len() as u32,
        p_dynamic_states: dynamic_states.as_ptr(),
        ..Default::default()
    };
    let create_info = vk::GraphicsPipelineCreateInfo {
        stage_count: stages.len() as u32,
        p_stages: stages.as_ptr(),
        p_vertex_input_state: &vertex_input_state,
        p_input_assembly_state: &input_assembly,
        p_dynamic_state: &dynamic_state,
        p_rasterization_state: &rasterization_state,
        p_multisample_state: &multisample_state,
        p_color_blend_state: &color_blend_state,
        p_tessellation_state: std::ptr::null(),
        p_viewport_state: &viewport_state,
        p_depth_stencil_state: std::ptr::null(),
        render_pass: render_pass.raw(),
        layout: layout.raw(),
        subpass: 0,

        base_pipeline_handle: vk::Pipeline::null(),
        base_pipeline_index: 0,
        ..Default::default()
    };
    raii::Pipeline::new_graphics_pipeline(render_device, create_info)
}
