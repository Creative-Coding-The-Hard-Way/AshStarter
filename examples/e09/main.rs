use {
    anyhow::Result,
    ash::vk,
    ccthw::{
        application::{Application, GlfwWindow, State},
        graphics::vulkan_api::{
            BindlessTriangles, BindlessVertex, ColorPass, FrameStatus,
            FramesInFlight, RenderDevice, TextureLoader,
        },
    },
    ccthw_ash_instance::PhysicalDeviceFeatures,
    std::sync::Arc,
};

struct BindlessTrianglesExample {
    vertices: Vec<BindlessVertex>,
    frames_in_flight: FramesInFlight,
    color_pass: ColorPass,
    bindless_triangles: BindlessTriangles,
    render_device: Arc<RenderDevice>,
}

impl State for BindlessTrianglesExample {
    fn new(window: &mut GlfwWindow) -> Result<Self> {
        window.set_key_polling(true);

        let render_device = unsafe {
            // SAFE because the render device is destroyed when state is
            // dropped.
            let mut device_features = PhysicalDeviceFeatures::default();
            // enable synchronization2 for queue_submit2
            device_features.vulkan_13_features_mut().synchronization2 =
                vk::TRUE;

            // enable descriptor indexing for bindless graphics
            device_features
                .descriptor_indexing_features_mut()
                .shader_sampled_image_array_non_uniform_indexing = vk::TRUE;
            device_features
                .descriptor_indexing_features_mut()
                .runtime_descriptor_array = vk::TRUE;

            window.create_default_render_device(device_features)?
        };

        let frames_in_flight = unsafe {
            // SAFE because the render device is destroyed when state is dropped
            FramesInFlight::new(
                render_device.clone(),
                window.get_framebuffer_size(),
                3,
            )?
        };

        let color_pass = unsafe {
            ColorPass::new(render_device.clone(), frames_in_flight.swapchain())?
        };

        let textures =
            unsafe {
                let mut loader = TextureLoader::new(render_device.clone())?;
                vec![
                    Arc::new(loader.load_texture_2d(
                        "examples/e09/my_example_texture.png",
                    )?),
                    Arc::new(loader.load_texture_2d(
                        "examples/e09/my_example_texture_2.png",
                    )?),
                ]
            };

        let bindless_triangles = unsafe {
            BindlessTriangles::new(
                render_device.clone(),
                color_pass.render_pass(),
                &frames_in_flight,
                &textures,
            )?
        };

        Ok(Self {
            vertices: Vec::with_capacity(10_000),
            frames_in_flight,
            color_pass,
            bindless_triangles,
            render_device,
        })
    }

    fn handle_event(
        &mut self,
        window: &mut GlfwWindow,
        window_event: glfw::WindowEvent,
    ) -> Result<()> {
        use glfw::{Action, Key, WindowEvent};
        match window_event {
            WindowEvent::Key(Key::Space, _, Action::Release, _) => {
                window.toggle_fullscreen()?;
            }
            WindowEvent::Key(Key::Escape, _, Action::Release, _) => {
                window.set_should_close(true);
            }
            _ => (),
        }
        Ok(())
    }

    fn update(&mut self, window: &mut GlfwWindow) -> Result<()> {
        let frame = match self.frames_in_flight.acquire_frame()? {
            FrameStatus::FrameAcquired(frame) => frame,
            FrameStatus::SwapchainNeedsRebuild => {
                return self.rebuild_swapchain(window);
            }
        };

        let quad_at =
            |x: f32, y: f32, texture_index: i32| -> [BindlessVertex; 6] {
                let w = 0.5;
                let h = 0.5;
                let top = 0.0;
                let bottom = 1.0;
                let left = 0.0;
                let right = 1.0;
                let tex = texture_index as f32;
                [
                    // --------------
                    // upper triangle
                    BindlessVertex {
                        pos: [x, y, 0.0, 1.0],
                        uv: [left, top, tex],
                        color: [1.0, 1.0, 1.0, 1.0],
                        ..Default::default()
                    },
                    BindlessVertex {
                        pos: [x + w, y, 0.0, 1.0],
                        uv: [right, top, tex],
                        color: [1.0, 1.0, 1.0, 1.0],
                        ..Default::default()
                    },
                    BindlessVertex {
                        pos: [x, y + h, 0.0, 1.0],
                        uv: [left, bottom, tex],
                        color: [1.0, 1.0, 1.0, 1.0],
                        ..Default::default()
                    },
                    // --------------
                    // lower triangle
                    BindlessVertex {
                        pos: [x, y + h, 0.0, 1.0],
                        uv: [left, bottom, tex],
                        color: [1.0, 1.0, 1.0, 1.0],
                        ..Default::default()
                    },
                    BindlessVertex {
                        pos: [x + w, y, 0.0, 1.0],
                        uv: [right, top, tex],
                        color: [1.0, 1.0, 1.0, 1.0],
                        ..Default::default()
                    },
                    BindlessVertex {
                        pos: [x + w, y + h, 0.0, 1.0],
                        uv: [right, bottom, tex],
                        color: [1.0, 1.0, 1.0, 1.0],
                        ..Default::default()
                    },
                ]
            };

        self.vertices.clear();
        self.vertices.extend_from_slice(&quad_at(-0.75, -0.25, 0));
        self.vertices.extend_from_slice(&quad_at(0.25, -0.25, 1));

        unsafe {
            self.color_pass
                .begin_render_pass_inline(&frame, [0.2, 0.2, 0.3, 1.0]);

            self.bindless_triangles
                .write_vertices_for_frame(&frame, &self.vertices)?;

            self.bindless_triangles.draw_vertices(
                &frame,
                self.frames_in_flight.swapchain().extent(),
            )?;

            self.render_device
                .device()
                .cmd_end_render_pass(frame.command_buffer());
        }

        self.frames_in_flight.present_frame(frame)?;

        Ok(())
    }
}

impl BindlessTrianglesExample {
    /// Rebuild the swapchain (typically because the current swapchain is
    /// out of date.
    fn rebuild_swapchain(&mut self, window: &GlfwWindow) -> Result<()> {
        unsafe {
            self.frames_in_flight
                .stall_and_rebuild_swapchain(window.get_framebuffer_size())?;

            self.color_pass = ColorPass::new(
                self.render_device.clone(),
                self.frames_in_flight.swapchain(),
            )?;
        };

        Ok(())
    }
}

fn main() -> Result<()> {
    Application::<BindlessTrianglesExample>::run()
}
