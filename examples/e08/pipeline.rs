use std::{ffi::CStr, sync::Arc};

use ash::vk;
use ccthw::graphics::vulkan_api::{
    DescriptorSetLayout, GraphicsPipeline, PipelineLayout, RenderDevice,
    RenderPass, ShaderModule, VulkanDebug, VulkanError,
};
use memoffset::offset_of;

use super::{PushConstant, Vertex};

pub fn create_pipeline_layout(
    render_device: Arc<RenderDevice>,
) -> Result<PipelineLayout, VulkanError> {
    let descriptor_set_layout = Arc::new(DescriptorSetLayout::new(
        render_device.clone(),
        &[vk::DescriptorSetLayoutBinding {
            binding: 0,
            descriptor_type: vk::DescriptorType::UNIFORM_BUFFER,
            descriptor_count: 1,
            stage_flags: vk::ShaderStageFlags::VERTEX,
            p_immutable_samplers: std::ptr::null(),
        }],
    )?);
    descriptor_set_layout
        .set_debug_name("triangle pipeline descriptor set layout");
    let pipeline_layout = PipelineLayout::new(
        render_device,
        &[descriptor_set_layout],
        &[vk::PushConstantRange {
            stage_flags: vk::ShaderStageFlags::VERTEX,
            offset: 0,
            size: std::mem::size_of::<PushConstant>() as u32,
        }],
    )?;
    pipeline_layout.set_debug_name("triangle pipeline layout");
    Ok(pipeline_layout)
}

pub fn create_pipeline(
    render_device: &Arc<RenderDevice>,
    render_pass: &RenderPass,
    layout: &PipelineLayout,
    samples: vk::SampleCountFlags,
) -> Result<GraphicsPipeline, VulkanError> {
    let shader_entry_name =
        unsafe { CStr::from_bytes_with_nul_unchecked(b"main\0") };
    let vertex_shader_program = ShaderModule::from_spirv_bytes(
        render_device.clone(),
        include_bytes!("./shaders/passthrough.vert.spv"),
    )?;
    let fragment_shader_program = ShaderModule::from_spirv_bytes(
        render_device.clone(),
        include_bytes!("./shaders/passthrough.frag.spv"),
    )?;
    let pipeline_stage_create_infos = [
        vk::PipelineShaderStageCreateInfo {
            module: unsafe { vertex_shader_program.raw() },
            stage: vk::ShaderStageFlags::VERTEX,
            p_name: shader_entry_name.as_ptr(),
            ..Default::default()
        },
        vk::PipelineShaderStageCreateInfo {
            module: unsafe { fragment_shader_program.raw() },
            stage: vk::ShaderStageFlags::FRAGMENT,
            p_name: shader_entry_name.as_ptr(),
            ..Default::default()
        },
    ];
    let vertex_input_binding_descriptions =
        [vk::VertexInputBindingDescription {
            binding: 0,
            stride: std::mem::size_of::<Vertex>() as u32,
            input_rate: vk::VertexInputRate::VERTEX,
        }];
    let vertex_input_attribute_descriptions = [
        vk::VertexInputAttributeDescription {
            binding: 0,
            location: 0,
            format: vk::Format::R32G32_SFLOAT,
            offset: offset_of!(Vertex, pos) as u32,
        },
        vk::VertexInputAttributeDescription {
            binding: 0,
            location: 1,
            format: vk::Format::R32G32B32A32_SFLOAT,
            offset: offset_of!(Vertex, color) as u32,
        },
    ];
    let vertex_input_state = vk::PipelineVertexInputStateCreateInfo {
        p_vertex_attribute_descriptions: vertex_input_attribute_descriptions
            .as_ptr(),
        vertex_attribute_description_count: vertex_input_attribute_descriptions
            .len() as u32,
        p_vertex_binding_descriptions: vertex_input_binding_descriptions
            .as_ptr(),
        vertex_binding_description_count: vertex_input_binding_descriptions
            .len() as u32,
        ..Default::default()
    };
    let input_assembly_state = vk::PipelineInputAssemblyStateCreateInfo {
        topology: vk::PrimitiveTopology::TRIANGLE_LIST,
        ..Default::default()
    };
    let dynamic_states =
        [vk::DynamicState::VIEWPORT, vk::DynamicState::SCISSOR];
    let dynamic_state = vk::PipelineDynamicStateCreateInfo {
        p_dynamic_states: dynamic_states.as_ptr(),
        dynamic_state_count: dynamic_states.len() as u32,
        ..Default::default()
    };
    let viewport_state = vk::PipelineViewportStateCreateInfo {
        viewport_count: 1,
        scissor_count: 1,
        ..Default::default()
    };
    let rasterization_state = vk::PipelineRasterizationStateCreateInfo {
        depth_clamp_enable: vk::FALSE,
        rasterizer_discard_enable: vk::FALSE,
        polygon_mode: vk::PolygonMode::FILL,
        line_width: 1.0,
        cull_mode: vk::CullModeFlags::NONE,
        front_face: vk::FrontFace::COUNTER_CLOCKWISE,
        depth_bias_enable: vk::FALSE,
        ..Default::default()
    };
    let multisample_state = vk::PipelineMultisampleStateCreateInfo {
        sample_shading_enable: vk::FALSE,
        rasterization_samples: samples,
        ..Default::default()
    };
    let color_blend_attachment = vk::PipelineColorBlendAttachmentState {
        color_write_mask: vk::ColorComponentFlags::RGBA,
        blend_enable: vk::FALSE,
        ..Default::default()
    };
    let color_blend_state = vk::PipelineColorBlendStateCreateInfo {
        logic_op_enable: vk::FALSE,
        logic_op: vk::LogicOp::COPY,
        attachment_count: 1,
        p_attachments: &color_blend_attachment,
        ..Default::default()
    };

    let graphics_pipeline_create_info = vk::GraphicsPipelineCreateInfo {
        p_stages: pipeline_stage_create_infos.as_ptr(),
        stage_count: pipeline_stage_create_infos.len() as u32,
        p_vertex_input_state: &vertex_input_state,
        p_input_assembly_state: &input_assembly_state,
        p_dynamic_state: &dynamic_state,
        p_viewport_state: &viewport_state,
        p_rasterization_state: &rasterization_state,
        p_multisample_state: &multisample_state,
        p_color_blend_state: &color_blend_state,

        // It is safe to take the raw handle here because it is not retained
        // after the pipeline is constructed.
        render_pass: unsafe { render_pass.raw() },
        layout: unsafe { layout.raw() },

        subpass: 0,
        base_pipeline_index: -1,
        base_pipeline_handle: vk::Pipeline::null(),
        p_tessellation_state: std::ptr::null(),
        p_depth_stencil_state: std::ptr::null(),

        ..Default::default()
    };

    GraphicsPipeline::new(render_device.clone(), &graphics_pipeline_create_info)
}
