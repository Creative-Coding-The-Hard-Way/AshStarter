mod pipeline;

use super::{RenderPass, RenderPassArgs, Renderer, TriangleCanvas};

use crate::vulkan::{BufferAllocator, RenderDevice};

use ::{
    anyhow::Result,
    ash::{version::DeviceV1_0, vk},
};

#[derive(Debug, Copy, Clone)]
struct Vertex {
    pos: [f32; 2],
    rgba: [f32; 4],
}

const MAX_TRIANGLE_COUNT: usize = 1;
const MAX_VERTEX_COUNT: usize = MAX_TRIANGLE_COUNT * 3;

impl TriangleCanvas {
    pub fn new(
        vk_dev: &RenderDevice,
        vk_alloc: &mut impl BufferAllocator,
    ) -> Result<Self> {
        let render_pass = RenderPass::new(
            vk_dev,
            "TriangleCanvas",
            RenderPassArgs {
                ..Default::default()
            },
        )?;
        let descriptor_layout = pipeline::create_descriptor_set_layout(
            vk_dev,
            "TriangleCanvas Descriptor Layout",
        )?;
        let pipeline_layout = pipeline::create_pipeline_layout(
            vk_dev,
            descriptor_layout,
            "TriangleCanvas Pipeline Layout",
        )?;
        let pipeline = pipeline::create_pipeline(
            vk_dev,
            render_pass.render_pass,
            pipeline_layout,
            "TriangleCanvas Pipeline",
        )?;
        let descriptor_pool = pipeline::create_descriptor_pool(
            vk_dev,
            "TriangleCanvas Descriptor Pool",
        )?;
        let descriptor_sets = pipeline::allocate_descriptor_sets(
            vk_dev,
            descriptor_pool,
            descriptor_layout,
            "TriangleCanvas Descriptor Set",
        )?;

        let total_frames = vk_dev.swapchain().image_views.len();
        let mut vertex_data = vec![];
        for _ in 0..total_frames {
            let mut buffer = vk_alloc.create_buffer(
                vk_dev,
                vk::BufferUsageFlags::STORAGE_BUFFER,
                vk::MemoryPropertyFlags::HOST_VISIBLE
                    | vk::MemoryPropertyFlags::HOST_COHERENT,
                (MAX_VERTEX_COUNT * std::mem::size_of::<Vertex>()) as u64,
            )?;
            buffer.map(vk_dev)?;
            let data = buffer.data_mut::<Vertex>()?;
            data[0] = Vertex {
                pos: [0.0, 0.5],
                rgba: [0.8, 0.1, 0.1, 1.0],
            };
            data[1] = Vertex {
                pos: [0.5, -0.5],
                rgba: [0.1, 0.8, 0.1, 1.0],
            };
            data[2] = Vertex {
                pos: [-0.5, -0.5],
                rgba: [0.1, 0.1, 0.8, 1.0],
            };
            vertex_data.push(buffer);
        }

        pipeline::update_descriptor_sets(
            vk_dev,
            &descriptor_sets,
            &vertex_data,
        );

        Ok(Self {
            vertex_data,
            descriptor_sets,
            render_pass,
            pipeline,
            pipeline_layout,
            descriptor_layout,
            descriptor_pool,
        })
    }

    /// Destroy all vulkan resources owned by the renderer.
    pub unsafe fn destroy(
        &mut self,
        vk_dev: &RenderDevice,
        vk_alloc: &mut impl BufferAllocator,
    ) -> Result<()> {
        self.destroy_swapchain_resources(vk_dev, vk_alloc)?;
        vk_dev
            .logical_device
            .destroy_pipeline_layout(self.pipeline_layout, None);
        vk_dev
            .logical_device
            .destroy_descriptor_set_layout(self.descriptor_layout, None);
        Ok(())
    }

    pub unsafe fn rebuild_swapchain_resources(
        &mut self,
        vk_dev: &RenderDevice,
        vk_alloc: &mut impl BufferAllocator,
    ) -> Result<()> {
        self.destroy_swapchain_resources(vk_dev, vk_alloc)?;
        self.render_pass = RenderPass::new(
            vk_dev,
            "TriangleCanvas",
            RenderPassArgs {
                ..Default::default()
            },
        )?;
        self.pipeline = pipeline::create_pipeline(
            vk_dev,
            self.render_pass.render_pass,
            self.pipeline_layout,
            "TriangleCanvas Pipeline",
        )?;
        self.descriptor_pool = pipeline::create_descriptor_pool(
            vk_dev,
            "TriangleCanvas Descriptor Pool",
        )?;
        self.descriptor_sets = pipeline::allocate_descriptor_sets(
            vk_dev,
            self.descriptor_pool,
            self.descriptor_layout,
            "TriangleCanvas Descriptor Set",
        )?;

        let total_frames = vk_dev.swapchain().image_views.len();
        for _ in 0..total_frames {
            let mut buffer = vk_alloc.create_buffer(
                vk_dev,
                vk::BufferUsageFlags::STORAGE_BUFFER,
                vk::MemoryPropertyFlags::HOST_VISIBLE
                    | vk::MemoryPropertyFlags::HOST_COHERENT,
                (MAX_VERTEX_COUNT * std::mem::size_of::<Vertex>()) as u64,
            )?;
            buffer.map(vk_dev)?;

            let data = buffer.data_mut::<Vertex>()?;
            data[0] = Vertex {
                pos: [0.0, 0.5],
                rgba: [0.8, 0.1, 0.1, 1.0],
            };
            data[1] = Vertex {
                pos: [0.5, -0.5],
                rgba: [0.1, 0.8, 0.1, 1.0],
            };
            data[2] = Vertex {
                pos: [-0.5, -0.5],
                rgba: [0.1, 0.1, 0.8, 1.0],
            };

            self.vertex_data.push(buffer);
        }
        pipeline::update_descriptor_sets(
            vk_dev,
            &self.descriptor_sets,
            &self.vertex_data,
        );

        Ok(())
    }
}

impl TriangleCanvas {
    /// Destroy only the swapchain-dependent resources owned by this renderer.
    unsafe fn destroy_swapchain_resources(
        &mut self,
        vk_dev: &RenderDevice,
        vk_alloc: &mut impl BufferAllocator,
    ) -> Result<()> {
        self.render_pass.destroy(vk_dev);
        vk_dev.logical_device.destroy_pipeline(self.pipeline, None);
        for mut buffer in self.vertex_data.drain(..) {
            vk_alloc.destroy_buffer(vk_dev, &mut buffer)?;
        }
        vk_dev
            .logical_device
            .destroy_descriptor_pool(self.descriptor_pool, None);
        self.descriptor_sets.clear();
        Ok(())
    }
}

impl Renderer for TriangleCanvas {
    fn fill_command_buffer(
        &self,
        vk_dev: &RenderDevice,
        cmd: vk::CommandBuffer,
        current_image: usize,
    ) -> Result<()> {
        self.render_pass
            .begin_render_pass(vk_dev, cmd, current_image);
        unsafe {
            vk_dev.logical_device.cmd_bind_pipeline(
                cmd,
                vk::PipelineBindPoint::GRAPHICS,
                self.pipeline,
            );
            vk_dev.logical_device.cmd_bind_descriptor_sets(
                cmd,
                vk::PipelineBindPoint::GRAPHICS,
                self.pipeline_layout,
                0,
                &[self.descriptor_sets[current_image]],
                &[],
            );
            vk_dev.logical_device.cmd_draw(cmd, 3, 1, 0, 0);
        }

        self.render_pass.end_render_pass(vk_dev, cmd);
        Ok(())
    }
}
