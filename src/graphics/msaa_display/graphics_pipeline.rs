use std::ffi::CStr;

use ash::vk;

use super::MSAADisplay;
use crate::graphics::vulkan_api::{
    GraphicsPipeline, PipelineLayout, ShaderModule, VulkanError,
};

impl MSAADisplay {
    pub fn create_graphics_pipeline(
        &self,
        vertex_shader_bytes: &[u8],
        fragment_shader_bytes: &[u8],
        pipeline_layout: &PipelineLayout,
    ) -> Result<GraphicsPipeline, VulkanError> {
        self.create_graphics_pipeline_with_topology(
            vertex_shader_bytes,
            fragment_shader_bytes,
            pipeline_layout,
            vk::PrimitiveTopology::TRIANGLE_LIST,
        )
    }

    pub fn create_graphics_pipeline_with_topology(
        &self,
        vertex_shader_bytes: &[u8],
        fragment_shader_bytes: &[u8],
        pipeline_layout: &PipelineLayout,
        primitive_topology: vk::PrimitiveTopology,
    ) -> Result<GraphicsPipeline, VulkanError> {
        let shader_entry_name =
            unsafe { CStr::from_bytes_with_nul_unchecked(b"main\0") };
        let vertex_shader_program = ShaderModule::from_spirv_bytes(
            self.render_device.clone(),
            vertex_shader_bytes,
        )?;
        let fragment_shader_program = ShaderModule::from_spirv_bytes(
            self.render_device.clone(),
            fragment_shader_bytes,
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
        let vertex_input_state =
            vk::PipelineVertexInputStateCreateInfo::default();
        let input_assembly_state = vk::PipelineInputAssemblyStateCreateInfo {
            topology: primitive_topology,
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
            rasterization_samples: self.samples,
            ..Default::default()
        };
        let color_blend_attachment = vk::PipelineColorBlendAttachmentState {
            color_write_mask: vk::ColorComponentFlags::RGBA,
            blend_enable: vk::TRUE,
            src_color_blend_factor: vk::BlendFactor::SRC_ALPHA,
            dst_color_blend_factor: vk::BlendFactor::ONE_MINUS_SRC_ALPHA,
            color_blend_op: vk::BlendOp::ADD,
            src_alpha_blend_factor: vk::BlendFactor::ONE,
            dst_alpha_blend_factor: vk::BlendFactor::ZERO,
            alpha_blend_op: vk::BlendOp::ADD,
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
            render_pass: unsafe { self.render_pass.raw() },
            layout: unsafe { pipeline_layout.raw() },

            subpass: 0,
            base_pipeline_index: -1,
            base_pipeline_handle: vk::Pipeline::null(),
            p_tessellation_state: std::ptr::null(),
            p_depth_stencil_state: std::ptr::null(),

            ..Default::default()
        };

        GraphicsPipeline::new(
            self.render_device.clone(),
            &graphics_pipeline_create_info,
        )
    }
}
