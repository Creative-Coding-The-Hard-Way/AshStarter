use std::sync::Arc;

use anyhow::Result;
use ash::vk;
use ccthw::{
    graphics::{
        msaa_display::MSAADisplay,
        ortho_projection,
        vulkan_api::{
            CommandBuffer, DescriptorPool, DescriptorSet, DescriptorSetLayout,
            GraphicsPipeline, HostCoherentBuffer, PipelineLayout, RenderDevice,
        },
    },
    math::Mat4,
};

use super::{Particle, SimulationConfig};

/// Used to pass the projection matrix to the vertex shader.
#[derive(Debug, Copy, Clone)]
#[repr(C, packed)]
struct UniformBufferObject {
    projection: Mat4,
}

/// All of the resources needed to render particles to the screen.
pub struct Graphics {
    particles: Arc<HostCoherentBuffer<Particle>>,
    uniform_buffer: HostCoherentBuffer<UniformBufferObject>,
    descriptor_set: DescriptorSet,
    pipeline_layout: PipelineLayout,
    pipeline: GraphicsPipeline,
}

impl Graphics {
    pub fn new(
        render_device: &Arc<RenderDevice>,
        msaa_display: &MSAADisplay,
        particles: Arc<HostCoherentBuffer<Particle>>,
        config: SimulationConfig,
    ) -> Result<Self> {
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
            let descriptor_pool = Arc::new(DescriptorPool::new(
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
                &descriptor_pool,
                pipeline_layout.descriptor_set_layout(0),
                1,
            )?
            .pop()
            .unwrap()
        };
        let pipeline = msaa_display.create_graphics_pipeline_with_topology(
            include_bytes!("../shaders/particle_visualizer.vert.spv"),
            include_bytes!("../shaders/particle_visualizer.frag.spv"),
            &pipeline_layout,
            vk::PrimitiveTopology::POINT_LIST,
        )?;
        let uniform_buffer = HostCoherentBuffer::new_with_data(
            render_device.clone(),
            vk::BufferUsageFlags::UNIFORM_BUFFER,
            &[UniformBufferObject {
                projection: config.projection(),
            }],
        )?;
        unsafe {
            descriptor_set.write_uniform_buffer(0, &uniform_buffer);
            descriptor_set.write_storage_buffer(1, &particles);
        }
        Ok(Self {
            particles,
            uniform_buffer,
            descriptor_set,
            pipeline_layout,
            pipeline,
        })
    }

    /// Rebuild any swapchain-dependent resources.
    ///
    /// # Safety
    ///
    /// Unsafe because:
    ///   - the caller must ensure that no graphics commands are still pending
    ///     execution when this function is called
    pub unsafe fn rebuild_swapchain_resources(
        &mut self,
        msaa_display: &MSAADisplay,
    ) -> Result<()> {
        self.pipeline = msaa_display.create_graphics_pipeline_with_topology(
            include_bytes!("../shaders/particle_visualizer.vert.spv"),
            include_bytes!("../shaders/particle_visualizer.frag.spv"),
            &self.pipeline_layout,
            vk::PrimitiveTopology::POINT_LIST,
        )?;
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
        self.uniform_buffer.as_slice_mut()?[0] = UniformBufferObject {
            projection: config.projection(),
        };
        Ok(())
    }

    /// Draw the given particle buffer to the screen.
    ///
    /// # Safety
    ///
    /// Unsafe because:
    ///   - the render pass must already be started before calling this function
    ///   - read access to the vertex buffer must be synchronized in the command
    ///     buffer (using memory barriers or something) prior to calling this
    ///     function
    pub unsafe fn draw(
        &self,
        command_buffer: &mut CommandBuffer,
        viewport_extent: vk::Extent2D,
    ) -> Result<()> {
        command_buffer
            .bind_graphics_pipeline(&self.pipeline)
            .set_viewport(viewport_extent)
            .set_scissor(0, 0, viewport_extent)
            .bind_graphics_descriptor_sets(
                &self.pipeline_layout,
                &[&self.descriptor_set],
            )
            .draw(self.particles.element_count() as u32, 0);
        Ok(())
    }
}

impl SimulationConfig {
    /// Get the ortho projection matrix which places the entire simulation on
    /// screen.
    fn projection(&self) -> Mat4 {
        let right = self.dimensions[0] / 2.0;
        let left = -right;
        let top = self.dimensions[1] / 2.0;
        let bottom = -top;
        ortho_projection(left, right, bottom, top, 0.0, 1.0)
    }
}
