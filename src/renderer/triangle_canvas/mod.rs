mod pipeline;

use std::sync::Arc;

use anyhow::Result;
use ash::{version::DeviceV1_0, vk};

use crate::{
    math::Mat4,
    renderer::{FramebufferRenderPass, RenderPassArgs, Renderer},
    vulkan::{
        errors::VulkanError, Buffer, CommandBuffer, DescriptorPool,
        DescriptorSet, DescriptorSetLayout, GpuVec, ImageView, MemoryAllocator,
        Pipeline, PipelineLayout, RenderDevice, VulkanDebug,
    },
    vulkan_ext::CommandBufferExt,
};

const NAME: &str = "TriangleCanvas";

#[derive(Debug, Copy, Clone)]
pub struct Vertex2D {
    pub pos: [f32; 2],
    pub rgba: [f32; 4],
}

/// A renderer which just draws triangles on the screen.
pub struct TriangleCanvas {
    current_image: usize,
    current_color: [f32; 4],
    fbrp: FramebufferRenderPass,
    vertex_data: Vec<GpuVec<Vertex2D>>,
    indices: Vec<GpuVec<u32>>,
    pipeline: Pipeline,
    pipeline_layout: PipelineLayout,
    ubo: Buffer,
    descriptor_sets: Vec<DescriptorSet>,
    descriptor_layout: DescriptorSetLayout,
    descriptor_pool: DescriptorPool,
    vk_dev: Arc<RenderDevice>,
    vk_alloc: Arc<dyn MemoryAllocator>,
}

impl TriangleCanvas {
    pub fn new(
        vk_dev: Arc<RenderDevice>,
        vk_alloc: Arc<dyn MemoryAllocator>,
        msaa_color_target: &Arc<ImageView>,
        projection: Mat4,
    ) -> Result<TriangleCanvas, VulkanError> {
        let fbrp = FramebufferRenderPass::new(
            vk_dev.clone(),
            RenderPassArgs {
                samples: vk_dev
                    .get_supported_msaa(vk::SampleCountFlags::TYPE_4),
                ..Default::default()
            },
            msaa_color_target.clone(),
        )?;
        let descriptor_layout = DescriptorSetLayout::new(
            vk_dev.clone(),
            &[
                vk::DescriptorSetLayoutBinding {
                    binding: 0,
                    descriptor_type: vk::DescriptorType::STORAGE_BUFFER,
                    descriptor_count: 1,
                    stage_flags: vk::ShaderStageFlags::VERTEX,
                    ..Default::default()
                },
                vk::DescriptorSetLayoutBinding {
                    binding: 1,
                    descriptor_type: vk::DescriptorType::UNIFORM_BUFFER,
                    descriptor_count: 1,
                    stage_flags: vk::ShaderStageFlags::VERTEX,
                    ..Default::default()
                },
            ],
        )?;
        let pipeline_layout =
            PipelineLayout::new(vk_dev.clone(), &[descriptor_layout.raw], &[])?;

        let mut ubo = Buffer::new(
            vk_dev.clone(),
            vk_alloc.clone(),
            vk::BufferUsageFlags::UNIFORM_BUFFER,
            vk::MemoryPropertyFlags::HOST_VISIBLE
                | vk::MemoryPropertyFlags::HOST_COHERENT,
            std::mem::size_of::<f32>() as u64 * 16,
        )?;
        ubo.map()?;
        ubo.data_mut::<nalgebra::Matrix4<f32>>()?[0] = projection;

        let (pipeline, descriptor_pool, descriptor_sets, vertex_data, indices) =
            Self::build_swapchain_resources(
                vk_dev.clone(),
                vk_alloc.clone(),
                &fbrp,
                &pipeline_layout,
                &descriptor_layout,
                &ubo,
            )?;

        let triangle_canvas = Self {
            current_image: 0,
            current_color: [1.0, 1.0, 1.0, 1.0],

            fbrp,
            vertex_data,
            indices,
            descriptor_layout,
            descriptor_pool,
            descriptor_sets,
            ubo,
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
        msaa_color_target: &Arc<ImageView>,
        projection: Mat4,
    ) -> Result<(), VulkanError> {
        self.vertex_data.clear();
        self.descriptor_sets.clear();

        self.fbrp
            .rebuild_swapchain_resources(msaa_color_target.clone())?;
        let (pipeline, descriptor_pool, descriptor_sets, vertex_data, indices) =
            Self::build_swapchain_resources(
                self.vk_dev.clone(),
                self.vk_alloc.clone(),
                &self.fbrp,
                &self.pipeline_layout,
                &self.descriptor_layout,
                &self.ubo,
            )?;
        self.ubo.data_mut::<nalgebra::Matrix4<f32>>()?[0] = projection;
        self.pipeline = pipeline;
        self.descriptor_pool = descriptor_pool;
        self.descriptor_sets = descriptor_sets;
        self.vertex_data = vertex_data;
        self.indices = indices;

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
        self.indices[current_image].clear();
    }

    /// Push vertices for a triangle into the vertex buffer.
    pub fn add_triangle(
        &mut self,
        a: [f32; 2],
        b: [f32; 2],
        c: [f32; 2],
    ) -> Result<(), VulkanError> {
        let last_index = self.push_vertices(&[
            Vertex2D {
                pos: a,
                rgba: self.current_color,
            },
            Vertex2D {
                pos: b,
                rgba: self.current_color,
            },
            Vertex2D {
                pos: c,
                rgba: self.current_color,
            },
        ])?;
        self.push_indices(&[last_index - 2, last_index - 1, last_index])?;
        Ok(())
    }

    pub fn add_quad(
        &mut self,
        top_left: [f32; 2],
        top_right: [f32; 2],
        bottom_left: [f32; 2],
        bottom_right: [f32; 2],
    ) -> Result<(), VulkanError> {
        let last_index = self.push_vertices(&[
            Vertex2D {
                pos: top_left,
                rgba: self.current_color,
            },
            Vertex2D {
                pos: top_right,
                rgba: self.current_color,
            },
            Vertex2D {
                pos: bottom_left,
                rgba: self.current_color,
            },
            Vertex2D {
                pos: bottom_right,
                rgba: self.current_color,
            },
        ])?;
        let v_br = last_index;
        let v_bl = last_index - 1;
        let v_tr = last_index - 2;
        let v_tl = last_index - 3;
        self.push_indices(&[
            v_tl, v_tr, v_bl, // first triangle
            v_tr, v_bl, v_br, // second triangle
        ])?;
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
            self.vk_dev.logical_device.cmd_bind_index_buffer(
                cmd.raw,
                self.indices[current_image].buffer.raw,
                0,
                vk::IndexType::UINT32,
            );
            self.vk_dev.logical_device.cmd_draw_indexed(
                cmd.raw,
                self.indices[current_image].len() as u32, // index count
                1,                                        // instance count
                0,                                        // first index
                0,                                        // vertex offset
                0,                                        // first instance
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
        for (i, buf) in self.indices.iter().enumerate() {
            buf.set_debug_name(format!("{} - Indices {}", name, i))?;
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
    fn push_indices(&mut self, indices: &[u32]) -> Result<(), VulkanError> {
        let mut needs_rename = false;
        for index in indices {
            needs_rename |=
                self.indices[self.current_image].push_back(*index)?;
        }
        if needs_rename {
            self.indices[self.current_image].set_debug_name(format!(
                "{} - Indices {}",
                NAME, self.current_image
            ))?;
        }
        Ok(())
    }

    fn push_vertices(
        &mut self,
        vertices: &[Vertex2D],
    ) -> Result<u32, VulkanError> {
        let mut needs_rebound = false;
        for vertex in vertices {
            needs_rebound |=
                self.vertex_data[self.current_image].push_back(*vertex)?;
        }
        if needs_rebound {
            unsafe {
                self.descriptor_sets[self.current_image].bind_buffer(
                    0,
                    &self.vertex_data[self.current_image].buffer.raw,
                    vk::DescriptorType::STORAGE_BUFFER,
                );
            }
            self.vertex_data[self.current_image].set_debug_name(format!(
                "{} - VertexData {}",
                NAME, self.current_image
            ))?;
        }
        let last_index = self.vertex_data[self.current_image].len() - 1;
        Ok(last_index as u32)
    }

    fn build_swapchain_resources(
        vk_dev: Arc<RenderDevice>,
        vk_alloc: Arc<dyn MemoryAllocator>,
        fbrp: &FramebufferRenderPass,
        pipeline_layout: &PipelineLayout,
        descriptor_layout: &DescriptorSetLayout,
        ubo: &Buffer,
    ) -> Result<
        (
            Pipeline,
            DescriptorPool,
            Vec<DescriptorSet>,
            Vec<GpuVec<Vertex2D>>,
            Vec<GpuVec<u32>>,
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
        let mut indices = vec![];
        let mut vertex_data = vec![];
        for _ in 0..image_count {
            vertex_data.push(GpuVec::new(
                vk_dev.clone(),
                vk_alloc.clone(),
                vk::BufferUsageFlags::STORAGE_BUFFER,
                10,
            )?);
            indices.push(GpuVec::new(
                vk_dev.clone(),
                vk_alloc.clone(),
                vk::BufferUsageFlags::INDEX_BUFFER,
                10,
            )?);
        }
        for (set, data) in descriptor_sets.iter().zip(vertex_data.iter()) {
            unsafe {
                set.bind_buffer(
                    0,
                    &data.buffer.raw,
                    vk::DescriptorType::STORAGE_BUFFER,
                );
                set.bind_buffer(
                    1,
                    &ubo.raw,
                    vk::DescriptorType::UNIFORM_BUFFER,
                );
            }
        }

        Ok((
            pipeline,
            descriptor_pool,
            descriptor_sets,
            vertex_data,
            indices,
        ))
    }
}
