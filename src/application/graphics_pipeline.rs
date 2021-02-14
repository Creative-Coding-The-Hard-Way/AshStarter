mod render_pass;
mod shader_module;

use self::{render_pass::create_render_pass, shader_module::ShaderModule};
use crate::application::{Device, Swapchain};

use anyhow::{Context, Result};
use ash::{version::DeviceV1_0, vk};
use std::{ffi::CString, sync::Arc};

/// All vulkan resources related to the graphics pipeline.
pub struct GraphicsPipeline {
    pipeline_layout: vk::PipelineLayout,
    render_pass: vk::RenderPass,

    device: Arc<Device>,
    swapchain: Arc<Swapchain>,
}

impl GraphicsPipeline {
    pub fn new(
        device: &Arc<Device>,
        swapchain: &Arc<Swapchain>,
    ) -> Result<Arc<Self>> {
        let vertex_module = ShaderModule::new(
            device,
            "Vertex Shader",
            std::include_bytes!("../../shaders/sprv/inline_positions.vert.spv"),
        )?;
        let fragment_module = ShaderModule::new(
            device,
            "Fragment Shader",
            std::include_bytes!("../../shaders/sprv/passthrough.frag.spv"),
        )?;

        // Dynamic parts of the pipeline

        let entry = CString::new("main").unwrap();
        let vertex_create_info = vk::PipelineShaderStageCreateInfo::builder()
            .stage(vk::ShaderStageFlags::VERTEX)
            .module(vertex_module.shader_module)
            .name(&entry);
        let fragment_create_info = vk::PipelineShaderStageCreateInfo::builder()
            .stage(vk::ShaderStageFlags::FRAGMENT)
            .module(fragment_module.shader_module)
            .name(&entry);

        // Fixed Function Configuration

        let vertex_input_state =
            vk::PipelineVertexInputStateCreateInfo::builder()
                .vertex_binding_descriptions(&vec![])
                .vertex_attribute_descriptions(&vec![]);

        let input_assembly =
            vk::PipelineInputAssemblyStateCreateInfo::builder()
                .topology(vk::PrimitiveTopology::TRIANGLE_LIST)
                .primitive_restart_enable(false);

        let viewport_state = vk::PipelineViewportStateCreateInfo::builder()
            .viewport_count(1)
            .viewports(&vec![vk::Viewport::builder()
                .x(0.0)
                .y(0.0)
                .width(swapchain.extent.width as f32)
                .height(swapchain.extent.height as f32)
                .min_depth(0.0)
                .max_depth(1.0)
                .build()])
            .scissor_count(1)
            .scissors(&vec![vk::Rect2D::builder()
                .offset(vk::Offset2D { x: 0, y: 0 })
                .extent(swapchain.extent)
                .build()]);

        let raster_state = vk::PipelineRasterizationStateCreateInfo::builder()
            .depth_clamp_enable(false)
            .rasterizer_discard_enable(false)
            .polygon_mode(vk::PolygonMode::FILL)
            .line_width(1.0)
            .cull_mode(vk::CullModeFlags::BACK)
            .front_face(vk::FrontFace::CLOCKWISE)
            .depth_bias_enable(false)
            .depth_bias_constant_factor(0.0)
            .depth_bias_clamp(0.0)
            .depth_bias_slope_factor(0.0);

        let multisample_state =
            vk::PipelineMultisampleStateCreateInfo::builder()
                .sample_shading_enable(false)
                .rasterization_samples(vk::SampleCountFlags::TYPE_1)
                .min_sample_shading(1.0)
                .sample_mask(&vec![])
                .alpha_to_coverage_enable(false)
                .alpha_to_one_enable(false);

        let blend_state = vk::PipelineColorBlendStateCreateInfo::builder()
            .logic_op_enable(false)
            .logic_op(vk::LogicOp::COPY)
            .blend_constants([0.0, 0.0, 0.0, 0.0])
            .attachments(&vec![
                vk::PipelineColorBlendAttachmentState::builder()
                    .color_write_mask(
                        vk::ColorComponentFlags::R
                            | vk::ColorComponentFlags::G
                            | vk::ColorComponentFlags::B
                            | vk::ColorComponentFlags::A,
                    )
                    .blend_enable(false)
                    .src_color_blend_factor(vk::BlendFactor::ONE)
                    .dst_color_blend_factor(vk::BlendFactor::ZERO)
                    .color_blend_op(vk::BlendOp::ADD)
                    .src_alpha_blend_factor(vk::BlendFactor::ONE)
                    .dst_alpha_blend_factor(vk::BlendFactor::ZERO)
                    .alpha_blend_op(vk::BlendOp::ADD)
                    .build(),
            ]);

        let layouts = vec![];
        let push_constant_ranges = vec![];
        let pipeline_layout_create_info =
            vk::PipelineLayoutCreateInfo::builder()
                .set_layouts(&layouts)
                .push_constant_ranges(&push_constant_ranges);

        let pipeline_layout = unsafe {
            device
                .logical_device
                .create_pipeline_layout(&pipeline_layout_create_info, None)?
        };
        device.name_vulkan_object(
            "Graphics Pipeline Layout",
            vk::ObjectType::PIPELINE_LAYOUT,
            &pipeline_layout,
        )?;

        let render_pass = render_pass::create_render_pass(&device, &swapchain)?;

        // build pipeline object

        Ok(Arc::new(Self {
            pipeline_layout,
            render_pass,
            device: device.clone(),
            swapchain: swapchain.clone(),
        }))
    }
}

impl Drop for GraphicsPipeline {
    fn drop(&mut self) {
        unsafe {
            self.device
                .logical_device
                .destroy_render_pass(self.render_pass, None);
            self.device
                .logical_device
                .destroy_pipeline_layout(self.pipeline_layout, None);
        }
    }
}
