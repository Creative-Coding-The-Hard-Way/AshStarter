use crate::vulkan::{Buffer, RenderDevice};

use ::{
    anyhow::Result,
    ash::{version::DeviceV1_0, vk},
    std::ffi::CString,
};

pub(super) fn update_descriptor_sets(
    vk_dev: &RenderDevice,
    descriptor_set: &Vec<vk::DescriptorSet>,
    buffers: &Vec<Buffer>,
) {
    let descriptor_buffer_infos: Vec<vk::DescriptorBufferInfo> = buffers
        .iter()
        .map(|buffer| vk::DescriptorBufferInfo {
            buffer: buffer.raw,
            offset: 0,
            range: vk::WHOLE_SIZE,
        })
        .collect();
    let writes: Vec<vk::WriteDescriptorSet> = descriptor_buffer_infos
        .iter()
        .zip(descriptor_set.iter())
        .map(|(descriptor_buffer_info, descriptor_set)| {
            vk::WriteDescriptorSet {
                dst_set: *descriptor_set,
                dst_binding: 0,
                dst_array_element: 0,
                descriptor_count: 1,
                descriptor_type: vk::DescriptorType::STORAGE_BUFFER,
                p_image_info: std::ptr::null(),
                p_texel_buffer_view: std::ptr::null(),
                p_buffer_info: descriptor_buffer_info,
                ..Default::default()
            }
        })
        .collect();
    unsafe {
        vk_dev.logical_device.update_descriptor_sets(&writes, &[]);
    }
}

pub(super) fn allocate_descriptor_sets(
    vk_dev: &RenderDevice,
    descriptor_pool: vk::DescriptorPool,
    layout: vk::DescriptorSetLayout,
    debug_name: impl Into<String>,
) -> Result<Vec<vk::DescriptorSet>> {
    let descriptor_set_count = vk_dev.swapchain().image_views.len();
    let mut layouts = vec![];
    for i in 0..descriptor_set_count {
        layouts.push(layout);
    }
    let allocate_info = vk::DescriptorSetAllocateInfo {
        descriptor_pool,
        descriptor_set_count: layouts.len() as u32,
        p_set_layouts: layouts.as_ptr(),
        ..Default::default()
    };
    let descriptor_sets = unsafe {
        vk_dev
            .logical_device
            .allocate_descriptor_sets(&allocate_info)?
    };
    let owned_name = debug_name.into();
    for (i, descriptor_set) in descriptor_sets.iter().enumerate() {
        vk_dev.name_vulkan_object(
            format!("{} - {}", owned_name, i),
            vk::ObjectType::DESCRIPTOR_SET,
            *descriptor_set,
        )?;
    }
    Ok(descriptor_sets)
}

pub(super) fn create_descriptor_pool(
    vk_dev: &RenderDevice,
    debug_name: impl Into<String>,
) -> Result<vk::DescriptorPool> {
    let descriptor_count = vk_dev.swapchain().image_views.len() as u32;
    let pool_size = vk::DescriptorPoolSize {
        ty: vk::DescriptorType::STORAGE_BUFFER,
        descriptor_count,
    };
    let pool_create_info = vk::DescriptorPoolCreateInfo {
        flags: vk::DescriptorPoolCreateFlags::empty(),
        max_sets: descriptor_count,
        pool_size_count: 1,
        p_pool_sizes: &pool_size,
        ..Default::default()
    };
    let pool = unsafe {
        vk_dev
            .logical_device
            .create_descriptor_pool(&pool_create_info, None)?
    };
    vk_dev.name_vulkan_object(
        debug_name,
        vk::ObjectType::DESCRIPTOR_POOL,
        pool,
    )?;
    Ok(pool)
}

pub(super) fn create_descriptor_set_layout(
    vk_dev: &RenderDevice,
    debug_name: impl Into<String>,
) -> Result<vk::DescriptorSetLayout> {
    let descriptor_set_layout_binding = vk::DescriptorSetLayoutBinding {
        binding: 0,
        stage_flags: vk::ShaderStageFlags::VERTEX,
        descriptor_type: vk::DescriptorType::STORAGE_BUFFER,
        descriptor_count: 1,
        p_immutable_samplers: std::ptr::null(),
    };
    let descriptor_set_layout_create_info = vk::DescriptorSetLayoutCreateInfo {
        flags: vk::DescriptorSetLayoutCreateFlags::empty(),
        p_bindings: &descriptor_set_layout_binding,
        binding_count: 1,
        ..Default::default()
    };
    let layout = unsafe {
        vk_dev.logical_device.create_descriptor_set_layout(
            &descriptor_set_layout_create_info,
            None,
        )?
    };
    vk_dev.name_vulkan_object(
        debug_name,
        vk::ObjectType::DESCRIPTOR_SET_LAYOUT,
        layout,
    )?;
    Ok(layout)
}

pub(super) fn create_pipeline_layout(
    vk_dev: &RenderDevice,
    descriptor_set_layout: vk::DescriptorSetLayout,
    debug_name: impl Into<String>,
) -> Result<vk::PipelineLayout> {
    let pipeline_layout_create_info = vk::PipelineLayoutCreateInfo {
        p_set_layouts: &descriptor_set_layout,
        set_layout_count: 1,
        p_push_constant_ranges: std::ptr::null(),
        push_constant_range_count: 0,
        ..Default::default()
    };
    let pipeline_layout = unsafe {
        vk_dev
            .logical_device
            .create_pipeline_layout(&pipeline_layout_create_info, None)?
    };
    vk_dev.name_vulkan_object(
        debug_name,
        vk::ObjectType::PIPELINE_LAYOUT,
        pipeline_layout,
    )?;
    Ok(pipeline_layout)
}

pub(super) fn create_pipeline(
    vk_dev: &RenderDevice,
    render_pass: vk::RenderPass,
    pipeline_layout: vk::PipelineLayout,
    debug_name: impl Into<String>,
) -> Result<vk::Pipeline> {
    let vertex_module: vk::ShaderModule = vk_dev.create_shader_module(
        std::include_bytes!("shaders/triangle.vert.sprv"),
    )?;
    let fragment_module: vk::ShaderModule = vk_dev.create_shader_module(
        std::include_bytes!("shaders/triangle.frag.sprv"),
    )?;

    let shader_entry_point = CString::new("main")?;
    let vertex_create_info = vk::PipelineShaderStageCreateInfo {
        stage: vk::ShaderStageFlags::VERTEX,
        module: vertex_module,
        p_name: shader_entry_point.as_ptr(),
        ..Default::default()
    };
    let fragment_create_info = vk::PipelineShaderStageCreateInfo {
        stage: vk::ShaderStageFlags::FRAGMENT,
        module: fragment_module,
        p_name: shader_entry_point.as_ptr(),
        ..Default::default()
    };

    let vertex_input_state = vk::PipelineVertexInputStateCreateInfo {
        ..Default::default()
    };

    let input_assembly = vk::PipelineInputAssemblyStateCreateInfo {
        topology: vk::PrimitiveTopology::TRIANGLE_LIST,
        primitive_restart_enable: 0,
        ..Default::default()
    };
    let viewports = [vk::Viewport {
        x: 0.0,
        y: 0.0,
        width: vk_dev.swapchain().extent.width as f32,
        height: vk_dev.swapchain().extent.height as f32,
        min_depth: 0.0,
        max_depth: 1.0,
    }];
    let scissors = [vk::Rect2D {
        offset: vk::Offset2D { x: 0, y: 0 },
        extent: vk_dev.swapchain().extent,
    }];
    let viewport_state = vk::PipelineViewportStateCreateInfo {
        p_viewports: viewports.as_ptr(),
        viewport_count: viewports.len() as u32,
        p_scissors: scissors.as_ptr(),
        scissor_count: scissors.len() as u32,
        ..Default::default()
    };
    let raster_state = vk::PipelineRasterizationStateCreateInfo {
        depth_clamp_enable: 0,
        rasterizer_discard_enable: 0,
        polygon_mode: vk::PolygonMode::FILL,
        line_width: 1.0,
        cull_mode: vk::CullModeFlags::NONE,
        front_face: vk::FrontFace::CLOCKWISE,
        depth_bias_enable: 0,
        depth_bias_constant_factor: 0.0,
        depth_bias_clamp: 0.0,
        depth_bias_slope_factor: 0.0,
        ..Default::default()
    };
    let multisample_state = vk::PipelineMultisampleStateCreateInfo {
        sample_shading_enable: 0,
        rasterization_samples: vk::SampleCountFlags::TYPE_1,
        p_sample_mask: std::ptr::null(),
        min_sample_shading: 1.0,
        alpha_to_coverage_enable: 0,
        alpha_to_one_enable: 0,
        ..Default::default()
    };
    let blend_attachments = [vk::PipelineColorBlendAttachmentState {
        color_write_mask: vk::ColorComponentFlags::R
            | vk::ColorComponentFlags::G
            | vk::ColorComponentFlags::B
            | vk::ColorComponentFlags::A,
        blend_enable: 1,
        src_color_blend_factor: vk::BlendFactor::SRC_ALPHA,
        dst_color_blend_factor: vk::BlendFactor::ONE_MINUS_SRC_ALPHA,
        color_blend_op: vk::BlendOp::ADD,
        src_alpha_blend_factor: vk::BlendFactor::ONE,
        dst_alpha_blend_factor: vk::BlendFactor::ZERO,
        alpha_blend_op: vk::BlendOp::ADD,
    }];
    let blend_state = vk::PipelineColorBlendStateCreateInfo {
        logic_op_enable: 0,
        logic_op: vk::LogicOp::COPY,
        blend_constants: [0.0, 0.0, 0.0, 0.0],
        p_attachments: blend_attachments.as_ptr(),
        attachment_count: blend_attachments.len() as u32,
        ..Default::default()
    };

    let stages = [vertex_create_info, fragment_create_info];
    let pipeline_create_info = vk::GraphicsPipelineCreateInfo {
        p_stages: stages.as_ptr(),
        stage_count: stages.len() as u32,
        p_vertex_input_state: &vertex_input_state,
        p_input_assembly_state: &input_assembly,
        p_viewport_state: &viewport_state,
        p_rasterization_state: &raster_state,
        p_multisample_state: &multisample_state,
        p_color_blend_state: &blend_state,

        p_tessellation_state: std::ptr::null(),
        p_dynamic_state: std::ptr::null(),
        p_depth_stencil_state: std::ptr::null(),

        layout: pipeline_layout,
        render_pass,
        subpass: 0,
        base_pipeline_index: -1,
        base_pipeline_handle: vk::Pipeline::null(),

        ..Default::default()
    };

    let pipelines = unsafe {
        vk_dev
            .logical_device
            .create_graphics_pipelines(
                vk::PipelineCache::null(),
                &[pipeline_create_info],
                None,
            )
            .map_err(|(_, err)| err)?
    };
    let pipeline = pipelines[0];
    vk_dev.name_vulkan_object(
        debug_name,
        vk::ObjectType::PIPELINE,
        pipeline,
    )?;

    unsafe {
        vk_dev
            .logical_device
            .destroy_shader_module(vertex_module, None);
        vk_dev
            .logical_device
            .destroy_shader_module(fragment_module, None);
    }

    Ok(pipeline)
}
