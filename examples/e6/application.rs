//! This module defines the main application initialization, event loop, and
//! rendering.

use ccthw::{
    frame_pipeline::{FrameError, FramePipeline},
    glfw_window::GlfwWindow,
    math::projections,
    renderer::{ClearFrame, FinishFrame, Renderer, TriangleCanvas},
    timing::FrameRateLimit,
    vulkan::{self, RenderDevice},
};
use ::{anyhow::Result, ash::version::DeviceV1_0, std::sync::Arc};

// The main application state.
pub struct Application {
    // renderers
    clear_frame: ClearFrame,
    finish_frame: FinishFrame,
    triangle_canvas: TriangleCanvas,

    // app state
    fps_limit: FrameRateLimit,
    paused: bool,
    swapchain_needs_rebuild: bool,

    // vulkan core
    frame_pipeline: FramePipeline,
    vk_dev: Arc<RenderDevice>,
    glfw_window: GlfwWindow,
}

impl Application {
    /// Build a new instance of the application.
    pub fn new() -> Result<Self> {
        let mut glfw_window = GlfwWindow::new("Swapchain")?;
        glfw_window.window.set_key_polling(true);
        glfw_window.window.set_framebuffer_size_polling(true);

        // Create the vulkan render device
        let vk_dev = Arc::new(glfw_window.create_vulkan_device()?);
        let vk_alloc = vulkan::create_default_allocator(vk_dev.clone());

        let frame_pipeline = FramePipeline::new(vk_dev.clone())?;
        let (w, h) = glfw_window.window.get_framebuffer_size();

        let clear_frame = ClearFrame::new(
            vk_dev.clone(),
            vk_alloc.clone(),
            [0.1, 0.1, 0.2, 1.0],
        )?;
        let finish_frame = FinishFrame::new(
            vk_dev.clone(),
            clear_frame.color_render_target(),
        )?;
        let triangle_canvas = TriangleCanvas::new(
            vk_dev.clone(),
            vk_alloc.clone(),
            clear_frame.color_render_target(),
            projections::ortho(
                0.0,      // left
                w as f32, // right
                h as f32, // bottom
                0.0,      // top
                -1.0,     // znear
                1.0,      // zfar
            ),
        )?;

        Ok(Self {
            clear_frame,
            finish_frame,
            triangle_canvas,

            fps_limit: FrameRateLimit::new(120, 30),
            paused: false,
            swapchain_needs_rebuild: false,

            frame_pipeline,
            vk_dev,
            glfw_window,
        })
    }

    /// Run the application, blocks until the main event loop exits.
    pub fn run(mut self) -> Result<()> {
        let event_receiver = self.glfw_window.take_event_receiver()?;
        while !self.glfw_window.window.should_close() {
            self.fps_limit.start_frame();
            for (_, event) in
                self.glfw_window.flush_window_events(&event_receiver)
            {
                self.handle_event(event)?;
            }
            if self.swapchain_needs_rebuild {
                self.rebuild_swapchain_resources()?;
                self.swapchain_needs_rebuild = false;
            }
            if !self.paused {
                let result = self.compose_frame();
                match result {
                    Err(FrameError::SwapchainNeedsRebuild) => {
                        self.swapchain_needs_rebuild = true;
                    }
                    _ => result?,
                }
            }
            self.fps_limit.sleep_to_limit();
            //let fps = 1.0 / self.fps_limit.avg_frame_time().as_secs_f64();
            //log::debug!("Avg FPS: {}", fps);
        }
        Ok(())
    }

    /// Render the applications state in in a three-step process.
    fn compose_frame(&mut self) -> Result<(), FrameError> {
        let (index, cmd) = self.frame_pipeline.begin_frame()?;

        self.triangle_canvas.clear(index);
        self.triangle_canvas.set_color([0.2, 0.3, 0.4, 1.0]);
        self.triangle_canvas.add_triangle(
            [50.0, 400.0],
            [50.0, 200.0],
            [200.0, 400.0],
        )?;

        self.triangle_canvas.set_color([0.9, 0.2, 0.1, 1.0]);
        for i in 0..5 {
            let step = i as f32 * 30.0;
            self.triangle_canvas.add_quad(
                [450.0, 100.0 + step],
                [500.0, 100.0 + step],
                [455.0, 113.0 + step],
                [500.0, 110.0 + step],
            )?;
        }

        self.clear_frame.fill_command_buffer(cmd, index)?;
        self.triangle_canvas.fill_command_buffer(cmd, index)?;
        self.finish_frame.fill_command_buffer(cmd, index)?;

        self.frame_pipeline.end_frame(index)
    }

    /// Rebuild the swapchain and any dependent resources.
    fn rebuild_swapchain_resources(&mut self) -> Result<()> {
        if self.paused {
            self.glfw_window.glfw.wait_events();
            return Ok(());
        }
        unsafe {
            self.vk_dev.logical_device.device_wait_idle()?;
        }
        let (w, h) = self.glfw_window.window.get_framebuffer_size();
        self.vk_dev.rebuild_swapchain((w as u32, h as u32))?;

        unsafe {
            self.frame_pipeline.rebuild_swapchain_resources()?;
            self.clear_frame.rebuild_swapchain_resources()?;

            self.finish_frame.rebuild_swapchain_resources(
                self.clear_frame.color_render_target(),
            )?;
            self.triangle_canvas.rebuild_swapchain_resources(
                self.clear_frame.color_render_target(),
                projections::ortho(
                    0.0,      // left
                    w as f32, // right
                    h as f32, // bottom
                    0.0,      // top
                    -1.0,     // znear
                    1.0,      // zfar
                ),
            )?;
        }
        Ok(())
    }

    /// Handle a GLFW window event.
    fn handle_event(&mut self, event: glfw::WindowEvent) -> Result<()> {
        use glfw::{Action, Key, Modifiers, WindowEvent};
        match event {
            WindowEvent::Close => {
                self.glfw_window.window.set_should_close(true);
            }
            WindowEvent::Key(Key::Escape, _, Action::Press, _) => {
                self.glfw_window.window.set_should_close(true);
            }
            WindowEvent::Key(
                Key::Space,
                _,
                Action::Press,
                Modifiers::Control,
            ) => {
                self.glfw_window.toggle_fullscreen()?;
            }
            WindowEvent::FramebufferSize(w, h) => {
                self.paused = w == 0 || h == 0;
                self.swapchain_needs_rebuild = true;
            }
            _ => {}
        }
        Ok(())
    }
}

impl Drop for Application {
    fn drop(&mut self) {
        unsafe {
            self.vk_dev
                .logical_device
                .device_wait_idle()
                .expect("error while waiting for graphics device idle");
        }
    }
}
