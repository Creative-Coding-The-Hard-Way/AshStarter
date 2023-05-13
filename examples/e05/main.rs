use {
    anyhow::Result,
    ash::vk,
    ccthw::{
        application::{Application, GlfwWindow, State},
        graphics::vulkan_api::{
            raii, ColorPass, FrameStatus, FramesInFlight, RenderDevice,
        },
    },
    ccthw_ash_instance::PhysicalDeviceFeatures,
    std::sync::Arc,
};

mod pipeline;

use self::pipeline::create_pipeline;

struct FirstTriangleExample {
    frames_in_flight: FramesInFlight,
    _descriptor_set_layout: raii::DescriptorSetLayout,
    pipeline_layout: raii::PipelineLayout,
    pipeline: raii::Pipeline,
    color_pass: ColorPass,
    render_device: Arc<RenderDevice>,
}

impl State for FirstTriangleExample {
    fn new(window: &mut GlfwWindow) -> Result<Self> {
        window.set_key_polling(true);

        let render_device = unsafe {
            // SAFE because the render device is destroyed when state is
            // dropped.
            let mut device_features = PhysicalDeviceFeatures::default();
            // enable synchronization2 for queue_submit2
            device_features.vulkan_13_features_mut().synchronization2 =
                vk::TRUE;
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

        let descriptor_set_layout = unsafe {
            raii::DescriptorSetLayout::new_with_bindings(
                render_device.clone(),
                &[],
            )?
        };
        let pipeline_layout = unsafe {
            raii::PipelineLayout::new_with_layouts_and_ranges(
                render_device.clone(),
                &[descriptor_set_layout.raw()],
                &[],
            )?
        };
        let pipeline = unsafe {
            create_pipeline(
                render_device.clone(),
                include_bytes!("./shaders/static_triangle.vert.spv"),
                include_bytes!("./shaders/static_triangle.frag.spv"),
                &pipeline_layout,
                color_pass.render_pass(),
            )?
        };

        Ok(Self {
            _descriptor_set_layout: descriptor_set_layout,
            pipeline_layout,
            pipeline,
            color_pass,
            frames_in_flight,
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

        unsafe {
            self.color_pass
                .begin_render_pass_inline(&frame, [0.2, 0.2, 0.3, 1.0]);

            // draw commands go here
            self.render_device.device().cmd_bind_pipeline(
                frame.command_buffer(),
                vk::PipelineBindPoint::GRAPHICS,
                self.pipeline.raw(),
            );
            let vk::Extent2D { width, height } =
                self.frames_in_flight.swapchain().extent();
            self.render_device.device().cmd_set_viewport(
                frame.command_buffer(),
                0,
                &[vk::Viewport {
                    x: 0.0,
                    y: 0.0,
                    width: width as f32,
                    height: height as f32,
                    min_depth: 0.0,
                    max_depth: 1.0,
                }],
            );
            self.render_device.device().cmd_set_scissor(
                frame.command_buffer(),
                0,
                &[vk::Rect2D {
                    offset: vk::Offset2D { x: 0, y: 0 },
                    extent: vk::Extent2D { width, height },
                }],
            );
            self.render_device.device().cmd_draw(
                frame.command_buffer(),
                3,
                1,
                0,
                0,
            );

            self.render_device
                .device()
                .cmd_end_render_pass(frame.command_buffer());
        }

        self.frames_in_flight.present_frame(frame)?;

        Ok(())
    }
}

impl FirstTriangleExample {
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

            self.pipeline = create_pipeline(
                self.render_device.clone(),
                include_bytes!("./shaders/static_triangle.vert.spv"),
                include_bytes!("./shaders/static_triangle.frag.spv"),
                &self.pipeline_layout,
                self.color_pass.render_pass(),
            )?;
        };

        Ok(())
    }
}

fn main() -> Result<()> {
    Application::<FirstTriangleExample>::run()
}
