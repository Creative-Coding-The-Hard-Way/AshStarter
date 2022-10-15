mod pipeline;

use {
    anyhow::Result,
    ash::vk,
    ccthw::{
        application::{Application, GlfwWindow, State},
        graphics::{
            msaa_display::MSAADisplay,
            ortho_projection,
            vulkan_api::{
                HostCoherentBuffer, PhysicalDeviceFeatures, RenderDevice,
                VulkanDebug,
            },
            AcquiredFrame,
        },
        logging,
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

/// This example renders a triangle that rotates.
/// Triangle vertices are updated using a compute shader, rather than updating
/// on the CPU.
///
/// NOTE: This example is outrageously over-synchronized. Compute operations are
/// submitted on a separate GPU queue, but there are two calls to
/// device.wait_idle PER FRAME. This is basically never the right choice in a
/// real application, but it means there's no need to worry about memory
/// barriers or semaphores or fences or anything else. Future examples will
/// show better ways to synchronize this sort of GPU operation.
struct Example10Compute {
    application_start: std::time::Instant,
    _vertex_buffer: HostCoherentBuffer<Vertex>,
    uniform_buffer: HostCoherentBuffer<UniformBufferObject>,
    graphics: pipeline::Graphics,
    compute: pipeline::Compute,

    msaa_display: MSAADisplay,
    render_device: Arc<RenderDevice>,
}

impl Example10Compute {
    fn build_swapchain_resources(
        &mut self,
        framebuffer_size: (i32, i32),
    ) -> Result<()> {
        self.msaa_display
            .rebuild_swapchain_resources(framebuffer_size)?;

        self.graphics.pipeline = self.msaa_display.create_graphics_pipeline(
            include_bytes!("./shaders/passthrough.vert.spv"),
            include_bytes!("./shaders/passthrough.frag.spv"),
            &self.graphics.pipeline_layout,
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

impl State for Example10Compute {
    fn new(window: &mut GlfwWindow) -> Result<Self> {
        window.window_handle.set_key_polling(true);

        let render_device =
            Arc::new(window.create_render_device_with_features(
                PhysicalDeviceFeatures {
                    maintenance4: vk::PhysicalDeviceMaintenance4Features {
                        maintenance4: vk::TRUE,
                        ..Default::default()
                    },
                    ..Default::default()
                },
                |features| features.maintenance4.maintenance4 == vk::TRUE,
            )?);

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

        let graphics = pipeline::Graphics::new(&render_device, &msaa_display)?;
        let compute = pipeline::Compute::new(&render_device)?;
        unsafe {
            graphics
                .descriptor_set
                .write_uniform_buffer(0, &uniform_buffer);
            graphics
                .descriptor_set
                .write_storage_buffer(1, &vertex_buffer);
            compute
                .descriptor_set
                .write_storage_buffer(0, &vertex_buffer);
        }

        Ok(Self {
            application_start: Instant::now(),
            uniform_buffer,
            _vertex_buffer: vertex_buffer,
            graphics,
            compute,

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

        let last_frame = self.application_start;
        self.application_start = Instant::now();

        let angle = (self.application_start - last_frame).as_secs_f32()
            * (std::f32::consts::PI * 2.0 / 5.0);

        let vertex_count = unsafe { self._vertex_buffer.as_slice()?.len() };
        let group_count_x = ((vertex_count / 64) + 1) * 64;

        unsafe { self.compute.command_pool.reset()? };
        self.compute.command_buffer.begin_one_time_submit()?;
        unsafe {
            self.compute
                .command_buffer
                .bind_compute_pipeline(&self.compute.pipeline)
                .push_constant(
                    &self.compute.pipeline_layout,
                    vk::ShaderStageFlags::COMPUTE,
                    PushConstant { angle },
                )
                .bind_compute_descriptor_sets(
                    &self.compute.pipeline_layout,
                    &[&self.compute.descriptor_set],
                )
                .dispatch(group_count_x as u32, 1, 1)
        };
        self.compute.command_buffer.end_command_buffer()?;
        unsafe {
            self.compute.command_buffer.submit_compute_commands(
                &[],
                &[],
                &[],
                None,
            )?;
        }

        // Wait for all Device operations to complete before moving on to
        // render this frame. This means there's no chance of this frame's
        // draw command accidentally reading from the vertex buffer while
        // the compute pipeline is still executing.
        self.render_device.wait_idle()?;

        // Draw the updated triangle
        unsafe {
            self.msaa_display
                .begin_render_pass(&mut frame, [0.2, 0.2, 0.2, 1.0]);

            frame
                .command_buffer()
                .bind_graphics_pipeline(&self.graphics.pipeline)
                .set_viewport(self.msaa_display.swapchain_extent())
                .set_scissor(0, 0, self.msaa_display.swapchain_extent())
                .bind_graphics_descriptor_sets(
                    &self.graphics.pipeline_layout,
                    &[&self.graphics.descriptor_set],
                )
                .draw(3, 0)
                .end_render_pass()
        };

        self.msaa_display.end_frame(frame)?;

        // Wait for all Device operations to complete before finishing this
        // frame. This means there's no chance of the next update writing
        // to the vertex buffer while this frame is attempting to read from it.
        self.render_device.wait_idle()?;

        Ok(())
    }
}

impl Drop for Example10Compute {
    fn drop(&mut self) {
        self.render_device
            .wait_idle()
            .expect("Unable to wait for the device to idle");
    }
}

fn main() -> Result<()> {
    let _logger = logging::setup()?;
    Application::<Example10Compute>::new("Example 10 - Compute")?.run()
}
