mod pipeline;

use {
    anyhow::Result,
    ash::vk,
    ccthw::{
        application::{Application, GlfwWindow, State},
        graphics::{
            ortho_projection,
            vulkan_api::{
                DescriptorPool, DescriptorSet, Framebuffer, GraphicsPipeline,
                HostCoherentBuffer, PipelineLayout, RenderDevice, RenderPass,
                VulkanDebug,
            },
            AcquiredFrame, SwapchainFrames,
        },
        logging,
    },
    std::sync::Arc,
};

#[derive(Debug, Copy, Clone)]
#[repr(C, packed)]
struct Vertex {
    pub pos: [f32; 2],
    pub color: [f32; 4],
}

#[derive(Debug, Copy, Clone)]
#[repr(C, packed)]
struct UniformBufferObject {
    pub proj: [[f32; 4]; 4],
}

#[derive(Debug, Copy, Clone)]
#[repr(C, packed)]
struct PushConstant {
    pub color: [f32; 4],
}

/// This example renders a triangle using a vertex buffer and a shader
/// pipeline.
struct Example6PushConstants {
    frame_count: u64,
    descriptor_set: DescriptorSet,
    _descriptor_pool: Arc<DescriptorPool>,
    pipeline_layout: PipelineLayout,
    graphics_pipeline: Option<GraphicsPipeline>,
    vertex_buffer: HostCoherentBuffer<Vertex>,
    uniform_buffer: HostCoherentBuffer<UniformBufferObject>,
    swapchain_frames: SwapchainFrames,
    framebuffers: Vec<Framebuffer>,
    render_pass: Option<RenderPass>,
    render_device: Arc<RenderDevice>,
}

impl Example6PushConstants {
    fn build_swapchain_resources(
        &mut self,
        framebuffer_size: (i32, i32),
    ) -> Result<()> {
        self.swapchain_frames.wait_for_all_frames_to_complete()?;
        self.framebuffers.clear();
        self.swapchain_frames.rebuild_swapchain(framebuffer_size)?;

        self.render_pass = Some(RenderPass::single_sampled(
            self.render_device.clone(),
            self.swapchain_frames.swapchain().format(),
        )?);

        let extent = self.swapchain_frames.swapchain().extent();
        for i in 0..self.swapchain_frames.swapchain_image_count() {
            let image_view = self.swapchain_frames.swapchain_image_view(i)?;
            self.framebuffers.push(Framebuffer::new(
                self.render_device.clone(),
                self.render_pass.as_ref().unwrap(),
                &[image_view.clone()],
                extent,
            )?);
        }

        self.graphics_pipeline = Some(pipeline::create_pipeline(
            &self.render_device,
            self.render_pass.as_ref().unwrap(),
            &self.pipeline_layout,
        )?);

        let right = framebuffer_size.0 as f32 / 2.0;
        let left = -right;
        let top = framebuffer_size.1 as f32 / 2.0;
        let bottom = -top;
        let projection = ortho_projection(left, right, bottom, top, 0.0, 1.0);
        unsafe {
            // Safe because no frames are in-flight (and therefore using this
            // buffer) at the time of writing. See the call to
            // "wait_for_all_frames_to_complete" above.
            self.uniform_buffer.as_slice_mut()?[0] = UniformBufferObject {
                proj: projection.into(),
            };
        }

        Ok(())
    }
}

impl State for Example6PushConstants {
    fn new(window: &mut GlfwWindow) -> Result<Self> {
        window.window_handle.set_key_polling(true);

        let render_device = Arc::new(window.create_render_device()?);
        let swapchain_frames = SwapchainFrames::new(render_device.clone())?;

        let mut vertex_buffer = HostCoherentBuffer::new(
            render_device.clone(),
            vk::BufferUsageFlags::VERTEX_BUFFER,
            3,
        )?;
        vertex_buffer.set_debug_name("triangle vertices");
        unsafe {
            // SAFE because the vertex slice is only being written, so no
            // uninitialized values are being read. The vertex buffer is not
            // in-use by the GPU so there are no races associated with writing
            // here.
            let vertices = vertex_buffer.as_slice_mut()?;
            vertices[0] = Vertex {
                pos: [0.0, 50.0],
                color: [1.0, 1.0, 1.0, 1.0],
            };
            vertices[1] = Vertex {
                pos: [-50.0, 0.0],
                color: [1.0, 1.0, 1.0, 1.0],
            };
            vertices[2] = Vertex {
                pos: [50.0, 0.0],
                color: [1.0, 1.0, 1.0, 1.0],
            };
        }

        let mut uniform_buffer = HostCoherentBuffer::new(
            render_device.clone(),
            vk::BufferUsageFlags::UNIFORM_BUFFER,
            1,
        )?;
        uniform_buffer.set_debug_name("uniform buffer");
        unsafe {
            // SAFE because the slice is only being written, so no
            // uninitialized values are being read. The buffer is not
            // in-use by the GPU so there are no races associated with writing
            // here.
            uniform_buffer.as_slice_mut()?[0] = UniformBufferObject {
                proj: [
                    [1.0, 0.0, 0.0, 0.0], // r1
                    [0.0, 1.0, 0.0, 0.0], // r2
                    [0.0, 0.0, 1.0, 0.0], // r3
                    [0.0, 0.0, 0.0, 1.0], // r4
                ],
            };
        }

        let pipeline_layout =
            pipeline::create_pipeline_layout(render_device.clone())?;
        let descriptor_pool = Arc::new(DescriptorPool::new(
            render_device.clone(),
            &[vk::DescriptorPoolSize {
                ty: vk::DescriptorType::UNIFORM_BUFFER,
                descriptor_count: 1,
            }],
            1,
        )?);
        let descriptor_set = {
            DescriptorSet::allocate(
                &render_device,
                &descriptor_pool,
                pipeline_layout.descriptor_set_layout(0),
                1,
            )?
            .pop()
            .unwrap()
        };

        // Safe because the descriptor set has not been bound yet.
        unsafe {
            descriptor_set.write_uniform_buffer(0, &uniform_buffer);
        }

        Ok(Self {
            frame_count: 0,
            descriptor_set,
            _descriptor_pool: descriptor_pool,
            pipeline_layout,
            graphics_pipeline: None,
            uniform_buffer,
            vertex_buffer,
            framebuffers: vec![],
            render_pass: None,
            swapchain_frames,
            render_device,
        })
    }

    fn handle_event(
        &mut self,
        glfw_window: &mut GlfwWindow,
        window_event: glfw::WindowEvent,
    ) -> Result<()> {
        use glfw::{Action, Key, WindowEvent};
        match window_event {
            WindowEvent::Key(Key::Space, _, Action::Release, _) => {
                glfw_window.toggle_fullscreen()?;
            }
            WindowEvent::Key(Key::Escape, _, Action::Release, _) => {
                glfw_window.window_handle.set_should_close(true);
            }
            WindowEvent::FramebufferSize(_, _) => {
                self.swapchain_frames.invalidate_swapchain();
            }
            _ => (),
        }
        Ok(())
    }

    fn update(&mut self, glfw_window: &mut GlfwWindow) -> Result<()> {
        let mut frame = match self.swapchain_frames.acquire_swapchain_frame()? {
            AcquiredFrame::SwapchainNeedsRebuild => {
                return self.build_swapchain_resources(
                    glfw_window.window_handle.get_framebuffer_size(),
                );
            }
            AcquiredFrame::Available(frame) => frame,
        };

        let swapchain_extent = self.swapchain_frames.swapchain().extent();
        let framebuffer = &self.framebuffers[frame.swapchain_image_index()];
        self.frame_count += 1;
        let green_value = (self.frame_count as f32 / 3000.0).sin().abs();

        // safe because the render pass and framebuffer will always outlive the
        // command buffer
        unsafe {
            frame
                .command_buffer()
                .begin_render_pass_inline(
                    self.render_pass.as_ref().unwrap(),
                    framebuffer,
                    swapchain_extent,
                    [0.0, 0.0, 1.0, 1.0],
                )
                .bind_graphics_pipeline(
                    self.graphics_pipeline.as_ref().unwrap(),
                )
                .set_viewport(swapchain_extent)
                .set_scissor(0, 0, swapchain_extent)
                .bind_vertex_buffer(&self.vertex_buffer, 0)
                .bind_graphics_descriptor_sets(
                    &self.pipeline_layout,
                    &[&self.descriptor_set],
                )
                .push_constant(
                    &self.pipeline_layout,
                    vk::ShaderStageFlags::VERTEX,
                    PushConstant {
                        color: [0.2, green_value, 0.2, 1.0],
                    },
                )
                .draw(3, 0)
                .end_render_pass();
        }

        self.swapchain_frames.present_frame(frame)?;

        Ok(())
    }
}

impl Drop for Example6PushConstants {
    fn drop(&mut self) {
        self.render_device
            .wait_idle()
            .expect("Unable to wait for the device to idle");
    }
}

fn main() -> Result<()> {
    let _logger = logging::setup()?;
    Application::<Example6PushConstants>::new("Example 6 - Push Constants")?
        .run()
}
