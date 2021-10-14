mod pipeline;

use super::{FramebufferRenderPass, Renderer, TriangleCanvas, Vertex2D};

use crate::{
    vulkan::{
        errors::VulkanError, CommandBuffer, DescriptorPool, DescriptorSet,
        DescriptorSetLayout, GpuVec, MemoryAllocator, Pipeline, PipelineLayout,
        RenderDevice, VulkanDebug,
    },
    vulkan_ext::CommandBufferExt,
};

use {
    anyhow::Result,
    ash::{version::DeviceV1_0, vk},
    std::sync::Arc,
};

const NAME: &str = "TriangleCanvas";

impl TriangleCanvas {
    pub fn new(
        vk_dev: Arc<RenderDevice>,
        vk_alloc: Arc<dyn MemoryAllocator>,
    ) -> Result<TriangleCanvas, VulkanError> {
        let fbrp =
            FramebufferRenderPass::new(vk_dev.clone(), Default::default())?;
        let descriptor_layout = DescriptorSetLayout::new(
            vk_dev.clone(),
            &[vk::DescriptorSetLayoutBinding {
                binding: 0,
                descriptor_type: vk::DescriptorType::STORAGE_BUFFER,
                descriptor_count: 1,
                stage_flags: vk::ShaderStageFlags::VERTEX,
                ..Default::default()
            }],
        )?;
        let pipeline_layout =
            PipelineLayout::new(vk_dev.clone(), &[descriptor_layout.raw], &[])?;

        let (pipeline, descriptor_pool, descriptor_sets, vertex_data) =
            Self::build_swapchain_resources(
                vk_dev.clone(),
                vk_alloc.clone(),
                &fbrp,
                &pipeline_layout,
                &descriptor_layout,
            )?;

        let triangle_canvas = Self {
            current_image: 0,
            current_color: [1.0, 1.0, 1.0, 1.0],

            fbrp,
            vertex_data,
            descriptor_layout,
            descriptor_pool,
            descriptor_sets,
            pipeline_layout,
            pipeline,
            vk_dev,
            vk_alloc,
        };
        triangle_canvas.set_debug_name(NAME)?;

        Ok(triangle_canvas)
    }

    pub unsafe fn rebuild_swapchain_resources(
        &mut self,
    ) -> Result<(), VulkanError> {
        self.vertex_data.clear();
        self.descriptor_sets.clear();

        self.fbrp.rebuild_swapchain_resources()?;
        let (pipeline, descriptor_pool, descriptor_sets, vertex_data) =
            Self::build_swapchain_resources(
                self.vk_dev.clone(),
                self.vk_alloc.clone(),
                &self.fbrp,
                &self.pipeline_layout,
                &self.descriptor_layout,
            )?;
        self.pipeline = pipeline;
        self.descriptor_pool = descriptor_pool;
        self.descriptor_sets = descriptor_sets;
        self.vertex_data = vertex_data;

        self.set_debug_name(NAME)?;
        Ok(())
    }

    /// Set the color to be applied to all new verties.
    pub fn set_color(&mut self, rgba: [f32; 4]) {
        self.current_color = rgba;
    }

    /// Clear the current frame's geometry and prepare to have vertices added.
    pub fn clear(&mut self, current_image: usize) {
        self.current_image = current_image;
        self.vertex_data[current_image].clear();
    }

    /// Add a single vertex to the vertex buffer.
    pub fn add_vertex(
        &mut self,
        pos: [f32; 2],
        rgba: [f32; 4],
    ) -> Result<(), VulkanError> {
        let needs_rebound = self.vertex_data[self.current_image]
            .push_back(Vertex2D { pos, rgba })?;
        if needs_rebound {
            unsafe {
                self.descriptor_sets[self.current_image].bind_storage_buffer(
                    0,
                    &self.vertex_data[self.current_image].buffer.raw,
                );
            }
            self.vertex_data[self.current_image].set_debug_name(format!(
                "{} - VertexData {}",
                NAME, self.current_image
            ))?;
        }
        Ok(())
    }

    /// Push vertices for a triangle into the vertex buffer.
    pub fn add_triangle(
        &mut self,
        a: [f32; 2],
        b: [f32; 2],
        c: [f32; 2],
    ) -> Result<(), VulkanError> {
        self.add_vertex(a, self.current_color)?;
        self.add_vertex(b, self.current_color)?;
        self.add_vertex(c, self.current_color)?;
        Ok(())
    }
}

impl Renderer for TriangleCanvas {
    fn fill_command_buffer(
        &self,
        cmd: &CommandBuffer,
        current_image: usize,
    ) -> Result<()> {
        unsafe {
            self.fbrp.begin_framebuffer_renderpass(
                cmd,
                current_image,
                vk::SubpassContents::INLINE,
            );
            self.vk_dev.logical_device.cmd_bind_pipeline(
                cmd.raw,
                vk::PipelineBindPoint::GRAPHICS,
                self.pipeline.raw,
            );
            self.vk_dev.logical_device.cmd_bind_descriptor_sets(
                cmd.raw,
                vk::PipelineBindPoint::GRAPHICS,
                self.pipeline_layout.raw,
                0,
                &[self.descriptor_sets[current_image].raw],
                &[],
            );
            self.vk_dev.logical_device.cmd_draw(
                cmd.raw,
                self.vertex_data[current_image].len() as u32,
                1,
                0,
                0,
            );
            cmd.end_renderpass();
        }
        Ok(())
    }
}

impl VulkanDebug for TriangleCanvas {
    fn set_debug_name(
        &self,
        debug_name: impl Into<String>,
    ) -> Result<(), crate::vulkan::errors::VulkanDebugError> {
        let name = debug_name.into();
        for (i, buf) in self.vertex_data.iter().enumerate() {
            buf.set_debug_name(format!("{} - VertexData {}", name, i))?;
        }
        for (i, set) in self.descriptor_sets.iter().enumerate() {
            set.set_debug_name(format!("{} - Descriptor Set {}", name, i))?;
        }

        self.pipeline_layout
            .set_debug_name(format!("{} - Pipeline Layout", name))?;
        self.pipeline
            .set_debug_name(format!("{} - Graphics Pipeline", name))?;
        self.descriptor_pool
            .set_debug_name(format!("{} - Descriptor Pool", name))?;
        self.descriptor_layout
            .set_debug_name(format!("{} - Descriptor Set Layout", name))?;
        self.fbrp.set_debug_name(format!("{} - FbRp", name))?;

        Ok(())
    }
}

impl TriangleCanvas {
    fn build_swapchain_resources(
        vk_dev: Arc<RenderDevice>,
        vk_alloc: Arc<dyn MemoryAllocator>,
        fbrp: &FramebufferRenderPass,
        pipeline_layout: &PipelineLayout,
        descriptor_layout: &DescriptorSetLayout,
    ) -> Result<
        (
            Pipeline,
            DescriptorPool,
            Vec<DescriptorSet>,
            Vec<GpuVec<Vertex2D>>,
        ),
        VulkanError,
    > {
        let image_count = vk_dev.swapchain_image_count();
        let pipeline = pipeline::create_pipeline(
            vk_dev.clone(),
            &fbrp.render_pass,
            &pipeline_layout,
        )?;
        let descriptor_pool = DescriptorPool::for_each_swapchain_image(
            vk_dev.clone(),
            0,
            image_count,
            0,
        )?;
        let descriptor_sets =
            descriptor_pool.allocate(descriptor_layout, image_count)?;
        let mut vertex_data = vec![];
        for _ in 0..image_count {
            vertex_data.push(GpuVec::new(
                vk_dev.clone(),
                vk_alloc.clone(),
                vk::BufferUsageFlags::STORAGE_BUFFER,
                10,
            )?);
        }
        for (set, data) in descriptor_sets.iter().zip(vertex_data.iter()) {
            unsafe {
                set.bind_storage_buffer(0, &data.buffer.raw);
            }
        }

        Ok((pipeline, descriptor_pool, descriptor_sets, vertex_data))
    }
}
