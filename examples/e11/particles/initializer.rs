use std::sync::Arc;

use anyhow::Result;
use ash::vk;
use ccthw::graphics::vulkan_api::{
    Buffer, CommandBuffer, CommandPool, ComputePipeline, DescriptorPool,
    DescriptorSet, DescriptorSetLayout, Fence, HostCoherentBuffer,
    PipelineLayout, RenderDevice, ShaderModule, VulkanError,
};

use super::SimulationConfig;

/// All Vulkan resources needed to run the Particle initialization compute
/// shader.
pub struct Initializer {
    fence: Fence,
    uniform_buffer: HostCoherentBuffer<SimulationConfig>,
    command_pool: Arc<CommandPool>,
    command_buffer: CommandBuffer,
    pipeline_layout: PipelineLayout,
    descriptor_set: DescriptorSet,
    pipeline: ComputePipeline,
}

impl Initializer {
    pub fn new(render_device: &Arc<RenderDevice>) -> Result<Self, VulkanError> {
        let pipeline_layout = {
            let descriptor_set_layout = Arc::new(DescriptorSetLayout::new(
                render_device.clone(),
                &[
                    vk::DescriptorSetLayoutBinding {
                        binding: 0,
                        descriptor_type: vk::DescriptorType::UNIFORM_BUFFER,
                        descriptor_count: 1,
                        stage_flags: vk::ShaderStageFlags::COMPUTE,
                        p_immutable_samplers: std::ptr::null(),
                    },
                    vk::DescriptorSetLayoutBinding {
                        binding: 1,
                        descriptor_type: vk::DescriptorType::STORAGE_BUFFER,
                        descriptor_count: 1,
                        stage_flags: vk::ShaderStageFlags::COMPUTE,
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
                include_bytes!("../shaders/initialize.comp.spv"),
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
            render_device.graphics_queue_family_index(),
            vk::CommandPoolCreateFlags::TRANSIENT,
        )?);
        let command_buffer = CommandBuffer::new(
            render_device.clone(),
            command_pool.clone(),
            vk::CommandBufferLevel::PRIMARY,
        )?;
        let fence = Fence::new(render_device.clone())?;
        fence.reset()?;
        let uniform_buffer = HostCoherentBuffer::new(
            render_device.clone(),
            vk::BufferUsageFlags::UNIFORM_BUFFER,
            1,
        )?;
        Ok(Initializer {
            fence,
            uniform_buffer,
            command_buffer,
            command_pool,
            pipeline_layout,
            descriptor_set,
            pipeline,
        })
    }

    /// Initialize the given particle buffer by running the initialize compute
    /// shader.
    ///
    /// # Safety
    ///
    /// Unsafe because:
    ///   - the application must ensure no other operations are currently
    ///     reading or writing to the particle buffer.
    pub unsafe fn initialize_particles(
        &mut self,
        particles: &impl Buffer,
        simulation_config: SimulationConfig,
    ) -> Result<(), VulkanError> {
        self.uniform_buffer.as_slice_mut()?[0] = simulation_config;
        self.descriptor_set
            .write_uniform_buffer(0, &self.uniform_buffer);
        self.descriptor_set.write_storage_buffer(1, particles);

        let group_count_x = ((particles.element_count() / 64) + 1) * 64;

        self.command_buffer.begin_one_time_submit()?;
        self.command_buffer
            .bind_compute_pipeline(&self.pipeline)
            .bind_compute_descriptor_sets(
                &self.pipeline_layout,
                &[&self.descriptor_set],
            )
            .dispatch(group_count_x as u32, 1, 1);
        self.command_buffer.end_command_buffer()?;

        self.command_buffer.submit_compute_commands(
            &[],
            &[],
            &[],
            Some(&self.fence),
        )?;

        self.fence.wait_and_reset()?;
        self.command_pool.reset()
    }
}
