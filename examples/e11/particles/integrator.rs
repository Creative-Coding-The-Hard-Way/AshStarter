use std::{sync::Arc, time::Instant};

use anyhow::Result;
use ash::vk;
use ccthw::graphics::vulkan_api::{
    Buffer, CommandBuffer, CommandPool, ComputePipeline, DescriptorPool,
    DescriptorSet, DescriptorSetLayout, Fence, HostCoherentBuffer,
    PipelineLayout, RenderDevice, ShaderModule, VulkanDebug, VulkanError,
};

use super::SimulationConfig;

/// Push Constants provided to the integration shader.
#[derive(Debug, Copy, Clone)]
#[repr(C, packed)]
struct IntegrationConstants {
    /// The integration timestep.
    dt: f32,
    x: f32,
    y: f32,
    pressed: f32,
}

/// All Vulkan resources needed to run the Particle initialization compute
/// shader.
pub struct Integrator {
    last_submission: Instant,
    fence: Fence,
    simulation_config: SimulationConfig,
    uniform_buffer: HostCoherentBuffer<SimulationConfig>,
    command_pool: Arc<CommandPool>,
    command_buffer: CommandBuffer,
    pipeline_layout: PipelineLayout,
    descriptor_set: DescriptorSet,
    pipelines: Vec<ComputePipeline>,
}

impl Integrator {
    pub fn new(
        render_device: &Arc<RenderDevice>,
        compute_shader_sources: &[&[u8]],
        simulation_config: SimulationConfig,
    ) -> Result<Self, VulkanError> {
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
                    vk::DescriptorSetLayoutBinding {
                        binding: 2,
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
                &[vk::PushConstantRange {
                    stage_flags: vk::ShaderStageFlags::COMPUTE,
                    offset: 0,
                    size: std::mem::size_of::<IntegrationConstants>() as u32,
                }],
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
                        descriptor_count: 2,
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
        let pipelines = {
            let mut pipelines = vec![];
            for compute_shader_source in compute_shader_sources {
                let shader_entry_name = unsafe {
                    std::ffi::CStr::from_bytes_with_nul_unchecked(b"main\0")
                };
                let module = ShaderModule::from_spirv_bytes(
                    render_device.clone(),
                    compute_shader_source,
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
                pipelines.push(ComputePipeline::new(
                    render_device.clone(),
                    &create_info,
                )?);
            }
            pipelines
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
        let fence = Fence::new(render_device.clone())?;
        fence.set_debug_name("Integrator Fence");
        let uniform_buffer = HostCoherentBuffer::new_with_data(
            render_device.clone(),
            vk::BufferUsageFlags::UNIFORM_BUFFER,
            &[simulation_config],
        )?;

        unsafe {
            descriptor_set.write_uniform_buffer(0, &uniform_buffer);
        }

        Ok(Integrator {
            last_submission: Instant::now(),
            fence,
            simulation_config,
            uniform_buffer,
            command_buffer,
            command_pool,
            pipeline_layout,
            descriptor_set,
            pipelines,
        })
    }

    /// Configure the graphics pipeline to read from the given buffer.
    ///
    /// # Safety
    ///
    /// Unsafe because:
    ///   - the caller must ensure no in-flight frames still reference the
    ///     old buffer.
    pub unsafe fn set_read_buffer(&mut self, buffer: &impl Buffer) {
        self.descriptor_set.write_storage_buffer(1, buffer);
    }

    /// Configure the graphics pipeline to read from the given buffer.
    ///
    /// # Safety
    ///
    /// Unsafe because:
    ///   - the caller must ensure no in-flight frames still reference the
    ///     old buffer.
    pub unsafe fn set_write_buffer(&mut self, buffer: &impl Buffer) {
        self.descriptor_set.write_storage_buffer(2, buffer);
    }

    /// Get the time since the last integration update.
    pub fn time_since_last_update(&self) -> f32 {
        let now = Instant::now();
        (now - self.last_submission).as_secs_f32().clamp(0.001, 0.1)
    }

    /// This is called automatically after every integration. But it can be
    /// useful to manually reset after a long stall (for example after
    /// initializing the particle buffer).
    pub fn reset_start_time(&mut self) {
        self.last_submission = Instant::now();
    }

    /// Integrate the particle buffer once and wait for the commands to finish.
    ///
    /// Shader index indicates which compute shader pipeline to use based on the
    /// sources provided when buliding the integrator.
    ///
    /// # Safety
    ///
    /// Unsafe because:
    ///   - the application must ensure no other operations are currently
    ///     reading or writing to the particle buffer.
    ///   - the application MUST wait for the last integration submission to
    ///     finish before calling this method again.
    pub unsafe fn integrate_particles(
        &mut self,
        shader_index: usize,
        mouse_pos: (f32, f32),
        pressed: bool,
    ) -> Result<(), VulkanError> {
        let dt = self.time_since_last_update();
        self.reset_start_time();

        // do 32 particles per thread
        let adjusted_count = self.simulation_config.particle_count / 32;
        let group_count_x = ((adjusted_count / 64) + 1) * 64;

        self.fence.reset()?;
        self.command_pool.reset()?;
        self.command_buffer.begin_one_time_submit()?;
        self.command_buffer
            .bind_compute_pipeline(&self.pipelines[shader_index])
            .bind_compute_descriptor_sets(
                &self.pipeline_layout,
                &[&self.descriptor_set],
            )
            .push_constant(
                &self.pipeline_layout,
                vk::ShaderStageFlags::COMPUTE,
                IntegrationConstants {
                    dt,
                    x: mouse_pos.0,
                    y: mouse_pos.1,
                    pressed: if pressed { 1.0 } else { 0.0 },
                },
            )
            .dispatch(group_count_x as u32, 1, 1)
            .end_command_buffer()?;
        self.command_buffer.submit_compute_commands(
            &[],
            &[],
            &[],
            Some(&self.fence),
        )
    }

    /// Check if the last integration submission has completed.
    pub fn is_integration_finished(&self) -> Result<bool, VulkanError> {
        unsafe { self.fence.get_status() }
    }

    /// Block until the most recent integration step has finished executing on
    /// the GPU.
    pub fn wait_for_integration_to_complete(&self) -> Result<(), VulkanError> {
        self.fence.wait()
    }

    /// Update internal buffers to reflect the current simulation config.
    ///
    /// # Safety
    ///
    /// Unsafe because:
    ///   - there is no internal synchronization
    ///   - the caller must ensure that no frames passed to draw() are still
    ///     pending execution
    pub unsafe fn update_simulation_config(
        &mut self,
        config: &SimulationConfig,
    ) -> Result<()> {
        self.uniform_buffer.as_slice_mut()?[0] = *config;
        Ok(())
    }
}
