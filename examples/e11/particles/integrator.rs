use {
    super::{Particle, SimulationConfig},
    anyhow::Result,
    ash::vk,
    ccthw::graphics::vulkan_api::{
        Buffer, CommandBuffer, ComputePipeline, DescriptorPool, DescriptorSet,
        DescriptorSetLayout, DeviceLocalBuffer, HostCoherentBuffer,
        PipelineLayout, RenderDevice, ShaderModule, VulkanError,
    },
    std::{sync::Arc, time::Instant},
};

/// Push Constants provided to the integration shader.
#[derive(Debug, Copy, Clone)]
#[repr(C, packed)]
struct IntegrationConstants {
    /// The integration timestep.
    dt: f32,
    x: f32,
    y: f32,
    left_pressed: f32,
    right_pressed: f32,
}

/// All Vulkan resources needed to run the Particle initialization compute
/// shader.
pub struct Integrator {
    last_submission: Instant,
    simulation_config: SimulationConfig,
    buffer: Arc<DeviceLocalBuffer<Particle>>,

    uniform_buffer: HostCoherentBuffer<SimulationConfig>,
    pipeline_layout: PipelineLayout,
    descriptor_set: DescriptorSet,
    pipelines: Vec<ComputePipeline>,
}

impl Integrator {
    pub fn new(
        render_device: &Arc<RenderDevice>,
        compute_shader_sources: &[&[u8]],
        simulation_config: SimulationConfig,
        buffer: Arc<DeviceLocalBuffer<Particle>>,
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

        let uniform_buffer = HostCoherentBuffer::new_with_data(
            render_device.clone(),
            vk::BufferUsageFlags::UNIFORM_BUFFER,
            &[simulation_config],
        )?;

        unsafe {
            descriptor_set.write_uniform_buffer(0, &uniform_buffer);
            descriptor_set.write_storage_buffer(1, &buffer);
        }

        Ok(Integrator {
            last_submission: Instant::now(),
            simulation_config,

            buffer,
            uniform_buffer,
            pipeline_layout,
            descriptor_set,
            pipelines,
        })
    }

    /// Add commands to the provided command buffer to dispatch a compute
    /// operation for the associated buffer.
    ///
    /// Memory barriers are placed around the dispatch call so that the vertex
    /// shader doesn't read while the compute shader is writing.
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
        command_buffer: &CommandBuffer,
        shader_index: usize,
        mouse_pos: (f32, f32),
        left_mouse_button_pressed: bool,
        right_mouse_button_pressed: bool,
    ) -> Result<(), VulkanError> {
        let now = Instant::now();
        let dt = (now - self.last_submission).as_secs_f32().min(0.01);
        self.last_submission = now;

        // do 32 particles per thread
        const EXECUTION_SIZE: u32 = 256;
        let particle_count = self.simulation_config.particle_count;
        let group_count_x =
            ((particle_count / EXECUTION_SIZE) + 1) * EXECUTION_SIZE;

        command_buffer
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
                    left_pressed: if left_mouse_button_pressed {
                        1.0
                    } else {
                        0.0
                    },
                    right_pressed: if right_mouse_button_pressed {
                        1.0
                    } else {
                        0.0
                    },
                },
            )
            .pipeline_buffer_memory_barriers(
                vk::PipelineStageFlags::VERTEX_SHADER,
                vk::PipelineStageFlags::COMPUTE_SHADER,
                &[vk::BufferMemoryBarrier {
                    src_access_mask: vk::AccessFlags::SHADER_READ,
                    dst_access_mask: vk::AccessFlags::SHADER_READ
                        | vk::AccessFlags::SHADER_WRITE,
                    offset: 0,
                    buffer: self.buffer.raw(),
                    size: self.buffer.size_in_bytes() as u64,
                    ..Default::default()
                }],
            )
            .dispatch(group_count_x as u32, 1, 1)
            .pipeline_buffer_memory_barriers(
                vk::PipelineStageFlags::COMPUTE_SHADER,
                vk::PipelineStageFlags::VERTEX_SHADER,
                &[vk::BufferMemoryBarrier {
                    src_access_mask: vk::AccessFlags::SHADER_READ
                        | vk::AccessFlags::SHADER_WRITE,
                    dst_access_mask: vk::AccessFlags::SHADER_READ,
                    offset: 0,
                    buffer: self.buffer.raw(),
                    size: self.buffer.size_in_bytes() as u64,
                    ..Default::default()
                }],
            );
        Ok(())
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
