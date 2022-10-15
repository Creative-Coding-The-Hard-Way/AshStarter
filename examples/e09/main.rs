use {
    anyhow::Result,
    ash::vk,
    ccthw::{
        application::{Application, GlfwWindow, State},
        graphics::{
            msaa_display::MSAADisplay,
            ortho_projection,
            vulkan_api::{
                DescriptorPool, DescriptorSet, DescriptorSetLayout,
                GraphicsPipeline, HostCoherentBuffer, PipelineLayout,
                RenderDevice, VulkanDebug,
            },
            AcquiredFrame,
        },
    },
    std::{sync::Arc, time::Instant},
};

#[derive(Debug, Copy, Clone)]
#[repr(C, packed)]
struct Vertex {
    pos: [f32; 2],
    pad: [f32; 2],
    color: [f32; 4],
}

impl Vertex {
    fn new(x: f32, y: f32, r: f32, g: f32, b: f32, a: f32) -> Self {
        Self {
            pos: [x, y],
            pad: [0.0, 0.0],
            color: [r, g, b, a],
        }
    }
}

#[derive(Debug, Copy, Clone)]
#[repr(C, packed)]
struct UniformBufferObject {
    pub proj: [[f32; 4]; 4],
}

#[derive(Debug, Copy, Clone)]
#[repr(C, packed)]
struct PushConstant {
    pub angle: f32,
}

/// This example renders a triangle using a vertex buffer and a shader
/// pipeline.
struct Example9MSAADisplay {
    application_start: std::time::Instant,
    _vertex_buffer: HostCoherentBuffer<Vertex>,
    uniform_buffer: HostCoherentBuffer<UniformBufferObject>,
    pipeline_layout: PipelineLayout,
    descriptor_set: DescriptorSet,
    pipeline: GraphicsPipeline,

    msaa_display: MSAADisplay,
    render_device: Arc<RenderDevice>,
}

impl Example9MSAADisplay {
    fn build_swapchain_resources(
        &mut self,
        framebuffer_size: (i32, i32),
    ) -> Result<()> {
        self.msaa_display
            .rebuild_swapchain_resources(framebuffer_size)?;

        self.pipeline = self.msaa_display.create_graphics_pipeline(
            include_bytes!("./shaders/passthrough.vert.spv"),
            include_bytes!("./shaders/passthrough.frag.spv"),
            &self.pipeline_layout,
        )?;

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

impl State for Example9MSAADisplay {
    fn new(window: &mut GlfwWindow) -> Result<Self> {
        window.window_handle.set_key_polling(true);

        let render_device = Arc::new(window.create_render_device()?);
        let msaa_display = MSAADisplay::new(
            render_device.clone(),
            window,
            vk::SampleCountFlags::TYPE_8,
        )?;

        let vertex_buffer = HostCoherentBuffer::new_with_data(
            render_device.clone(),
            vk::BufferUsageFlags::STORAGE_BUFFER,
            &[
                Vertex::new(0.0, 150.0, 0.5, 0.5, 0.8, 1.0),
                Vertex::new(-150.0, 0.0, 0.5, 0.5, 0.8, 1.0),
                Vertex::new(150.0, 0.0, 0.5, 0.5, 0.8, 1.0),
            ],
        )?;
        vertex_buffer.set_debug_name("triangle vertices");

        let vk::Extent2D { width, height } = msaa_display.swapchain_extent();
        let right = width as f32 / 2.0;
        let left = -right;
        let top = height as f32 / 2.0;
        let bottom = -top;
        let uniform_buffer = HostCoherentBuffer::new_with_data(
            render_device.clone(),
            vk::BufferUsageFlags::UNIFORM_BUFFER,
            &[UniformBufferObject {
                proj: ortho_projection(left, right, bottom, top, 0.0, 1.0)
                    .into(),
            }],
        )?;
        uniform_buffer.set_debug_name("uniform buffer");

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
                &[vk::PushConstantRange {
                    stage_flags: vk::ShaderStageFlags::VERTEX,
                    offset: 0,
                    size: std::mem::size_of::<PushConstant>() as u32,
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
                &render_device,
                &pool,
                pipeline_layout.descriptor_set_layout(0),
                1,
            )?
            .pop()
            .unwrap()
        };
        unsafe {
            descriptor_set.write_uniform_buffer(0, &uniform_buffer);
            descriptor_set.write_storage_buffer(1, &vertex_buffer);
        }

        let pipeline = msaa_display.create_graphics_pipeline(
            include_bytes!("./shaders/passthrough.vert.spv"),
            include_bytes!("./shaders/passthrough.frag.spv"),
            &pipeline_layout,
        )?;

        Ok(Self {
            application_start: Instant::now(),
            uniform_buffer,
            _vertex_buffer: vertex_buffer,
            pipeline_layout,
            descriptor_set,
            pipeline,

            msaa_display,
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
                self.msaa_display.invalidate_swapchain();
            }
            _ => (),
        }
        Ok(())
    }

    fn update(&mut self, glfw_window: &mut GlfwWindow) -> Result<()> {
        let mut frame = match self.msaa_display.begin_frame()? {
            AcquiredFrame::SwapchainNeedsRebuild => {
                return self.build_swapchain_resources(
                    glfw_window.window_handle.get_framebuffer_size(),
                );
            }
            AcquiredFrame::Available(frame) => frame,
        };

        let angle = (Instant::now() - self.application_start).as_secs_f32()
            * (std::f32::consts::PI * 2.0 / 5.0);

        // safe because the render pass and framebuffer will always outlive the
        // command buffer
        unsafe {
            self.msaa_display
                .begin_render_pass(&mut frame, [0.2, 0.2, 0.2, 1.0]);
            frame
                .command_buffer()
                .bind_graphics_pipeline(&self.pipeline)
                .set_viewport(self.msaa_display.swapchain_extent())
                .set_scissor(0, 0, self.msaa_display.swapchain_extent())
                .bind_graphics_descriptor_sets(
                    &self.pipeline_layout,
                    &[&self.descriptor_set],
                )
                .push_constant(
                    &self.pipeline_layout,
                    vk::ShaderStageFlags::VERTEX,
                    PushConstant { angle },
                )
                .draw(3, 0)
                .end_render_pass();
        };

        self.msaa_display.end_frame(frame)?;

        Ok(())
    }
}

impl Drop for Example9MSAADisplay {
    fn drop(&mut self) {
        self.render_device
            .wait_idle()
            .expect("Unable to wait for the device to idle");
    }
}

fn main() -> Result<()> {
    Application::<Example9MSAADisplay>::new("Example 9 - MSAADisplay")?.run()
}
