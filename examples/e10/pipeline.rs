use std::sync::Arc;

use ash::vk;
use ccthw::graphics::{
    msaa_display::MSAADisplay,
    vulkan_api::{
        CommandBuffer, CommandPool, ComputePipeline, DescriptorPool,
        DescriptorSet, DescriptorSetLayout, GraphicsPipeline, PipelineLayout,
        RenderDevice, ShaderModule, VulkanError,
    },
};

use super::PushConstant;

pub struct Graphics {
    pub pipeline_layout: PipelineLayout,
    pub descriptor_set: DescriptorSet,
    pub pipeline: GraphicsPipeline,
}

impl Graphics {
    pub fn new(
        render_device: &Arc<RenderDevice>,
        msaa_display: &MSAADisplay,
    ) -> Result<Self, VulkanError> {
        let pipeline_layout = {
            let descriptor_set_layout = Arc::new(DescriptorSetLayout::new(
                render_device.clone(),
                &[
                    vk::DescriptorSetLayoutBinding {
                        binding: 0,
                        descriptor_type: vk::DescriptorType::UNIFORM_BUFFER,
                        descriptor_count: 1,
                        stage_flags: vk::ShaderStageFlags::VERTEX,
                        p_immutable_samplers: std::ptr::null(),
                    },
                    vk::DescriptorSetLayoutBinding {
                        binding: 1,
                        descriptor_type: vk::DescriptorType::STORAGE_BUFFER,
                        descriptor_count: 1,
                        stage_flags: vk::ShaderStageFlags::VERTEX,
                        p_immutable_samplers: std::ptr::null(),
                    },
                ],
            )?);
            PipelineLayout::new(
                render_device.clone(),
                &[descriptor_set_layout],
                &[],
            )?
        };
        let descriptor_set = {
            let pool = Arc::new(DescriptorPool::new(
                render_device.clone(),
                &[
                    vk::DescriptorPoolSize {
                        ty: vk::DescriptorType::UNIFORM_BUFFER,
                        descriptor_count: 1,
                    },
                    vk::DescriptorPoolSize {
                        ty: vk::DescriptorType::STORAGE_BUFFER,
                        descriptor_count: 1,
                    },
                ],
                1,
            )?);
            DescriptorSet::allocate(
                render_device,
                &pool,
                pipeline_layout.descriptor_set_layout(0),
                1,
            )?
            .pop()
            .unwrap()
        };
        let pipeline = msaa_display.create_graphics_pipeline(
            include_bytes!("./shaders/passthrough.vert.spv"),
            include_bytes!("./shaders/passthrough.frag.spv"),
            &pipeline_layout,
        )?;
        Ok(Graphics {
            pipeline_layout,
            descriptor_set,
            pipeline,
        })
    }
}

pub struct Compute {
    pub command_pool: Arc<CommandPool>,
    pub command_buffer: CommandBuffer,
    pub pipeline_layout: PipelineLayout,
    pub descriptor_set: DescriptorSet,
    pub pipeline: ComputePipeline,
}

impl Compute {
    pub fn new(render_device: &Arc<RenderDevice>) -> Result<Self, VulkanError> {
        let pipeline_layout = {
            let descriptor_set_layout = Arc::new(DescriptorSetLayout::new(
                render_device.clone(),
                &[vk::DescriptorSetLayoutBinding {
                    binding: 0,
                    descriptor_type: vk::DescriptorType::STORAGE_BUFFER,
                    descriptor_count: 1,
                    stage_flags: vk::ShaderStageFlags::COMPUTE,
                    p_immutable_samplers: std::ptr::null(),
                }],
            )?);
            PipelineLayout::new(
                render_device.clone(),
                &[descriptor_set_layout],
                &[vk::PushConstantRange {
                    stage_flags: vk::ShaderStageFlags::COMPUTE,
                    offset: 0,
                    size: std::mem::size_of::<PushConstant>() as u32,
                }],
            )?
        };
        let descriptor_set = {
            let pool = Arc::new(DescriptorPool::new(
                render_device.clone(),
                &[vk::DescriptorPoolSize {
                    ty: vk::DescriptorType::STORAGE_BUFFER,
                    descriptor_count: 1,
                }],
                1,
            )?);
            DescriptorSet::allocate(
                render_device,
                &pool,
                pipeline_layout.descriptor_set_layout(0),
                1,
            )?
            .pop()
            .unwrap()
        };
        let pipeline = {
            let shader_entry_name = unsafe {
                std::ffi::CStr::from_bytes_with_nul_unchecked(b"main\0")
            };
            let module = ShaderModule::from_spirv_bytes(
                render_device.clone(),
                include_bytes!("./shaders/rotate.comp.spv"),
            )?;
            let stage_create_info = vk::PipelineShaderStageCreateInfo {
                stage: vk::ShaderStageFlags::COMPUTE,
                p_name: shader_entry_name.as_ptr(),
                module: unsafe { module.raw() },
                ..Default::default()
            };
            let create_info = vk::ComputePipelineCreateInfo {
                layout: unsafe { pipeline_layout.raw() },
                stage: stage_create_info,
                ..Default::default()
            };
            ComputePipeline::new(render_device.clone(), &create_info)?
        };
        let command_pool = Arc::new(CommandPool::new(
            render_device.clone(),
            render_device.compute_queue_family_index(),
            vk::CommandPoolCreateFlags::TRANSIENT,
        )?);
        let command_buffer = CommandBuffer::new(
            render_device.clone(),
            command_pool.clone(),
            vk::CommandBufferLevel::PRIMARY,
        )?;
        Ok(Compute {
            command_buffer,
            command_pool,
            pipeline_layout,
            descriptor_set,
            pipeline,
        })
    }
}
