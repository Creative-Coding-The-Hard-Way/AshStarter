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
            &[descriptor_layout],
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
        let descriptor_set = pipeline::allocate_descriptor_set(
            vk_dev,
            descriptor_pool,
            descriptor_layout,
            "TriangleCanvas Descriptor Set",
        )?;
        let mut vertex_data = vk_alloc.create_buffer(
            vk_dev,
            vk::BufferUsageFlags::STORAGE_BUFFER,
            vk::MemoryPropertyFlags::HOST_VISIBLE
                | vk::MemoryPropertyFlags::HOST_COHERENT,
            (MAX_VERTEX_COUNT * std::mem::size_of::<Vertex>()) as u64,
        )?;
        pipeline::update_descriptor_set(
            vk_dev,
            descriptor_set,
            vertex_data.raw,
        );

        vertex_data = {
            let mapped = vertex_data.map::<Vertex>(vk_dev)?;
            mapped.data[0] = Vertex {
                pos: [0.0, 0.5],
                rgba: [0.8, 0.1, 0.1, 1.0],
            };
            mapped.data[1] = Vertex {
                pos: [0.5, -0.5],
                rgba: [0.1, 0.8, 0.1, 1.0],
            };
            mapped.data[2] = Vertex {
                pos: [-0.5, -0.5],
                rgba: [0.1, 0.1, 0.8, 1.0],
            };
            mapped.unmap(vk_dev)
        };

        Ok(Self {
            vertex_data,
            render_pass,
            pipeline,
            pipeline_layout,
            descriptor_layout,
            descriptor_pool,
            descriptor_set,
        })
    }

    /// Destroy all vulkan resources owned by the renderer.
    pub unsafe fn destroy(
        &mut self,
        vk_dev: &RenderDevice,
        vk_alloc: &mut impl BufferAllocator,
    ) -> Result<()> {
        self.destroy_swapchain_resources(vk_dev);
        vk_dev
            .logical_device
            .destroy_pipeline_layout(self.pipeline_layout, None);
        vk_dev
            .logical_device
            .destroy_descriptor_set_layout(self.descriptor_layout, None);
        vk_dev
            .logical_device
            .destroy_descriptor_pool(self.descriptor_pool, None);
        vk_alloc.destroy_buffer(vk_dev, &mut self.vertex_data)?;
        Ok(())
    }
}

impl TriangleCanvas {
    /// Destroy only the swapchain-dependent resources owned by this renderer.
    unsafe fn destroy_swapchain_resources(&mut self, vk_dev: &RenderDevice) {
        self.render_pass.destroy(vk_dev);
        vk_dev.logical_device.destroy_pipeline(self.pipeline, None);
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
                &[self.descriptor_set],
                &[],
            );
            vk_dev.logical_device.cmd_draw(cmd, 3, 1, 0, 0);
        }

        self.render_pass.end_render_pass(vk_dev, cmd);
        Ok(())
    }

    unsafe fn rebuild_swapchain_resources(
        &mut self,
        vk_dev: &RenderDevice,
    ) -> Result<()> {
        self.destroy_swapchain_resources(vk_dev);
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
        Ok(())
    }
}
