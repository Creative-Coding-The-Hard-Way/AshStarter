//! This module defines the main application initialization, event loop, and
//! rendering.

use ccthw::{
    frame_pipeline::{FrameError, FramePipeline},
    glfw_window::GlfwWindow,
    renderer::{ClearFrame, FinishFrame, Renderer},
    timing::FrameRateLimit,
    vulkan,
};

use ::{anyhow::Result, ash::version::DeviceV1_0, std::sync::Arc};

// The main application state.
pub struct Application {
    // renderers
    clear_frame: ClearFrame,
    finish_frame: FinishFrame,
    frame_pipeline: FramePipeline,

    fps_limit: FrameRateLimit,
    paused: bool,
    swapchain_needs_rebuild: bool,

    vk_dev: Arc<vulkan::RenderDevice>,
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
        let frame_pipeline = FramePipeline::new(vk_dev.clone())?;

        Ok(Self {
            clear_frame: ClearFrame::new(vk_dev.clone(), [0.1, 0.1, 0.2, 1.0])?,
            finish_frame: FinishFrame::new(vk_dev.clone())?,

            frame_pipeline,
            vk_dev,
            glfw_window,

            fps_limit: FrameRateLimit::new(120, 30),
            paused: false,
            swapchain_needs_rebuild: false,
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

        self.clear_frame.fill_command_buffer(cmd, index)?;
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
            self.finish_frame.rebuild_swapchain_resources()?;
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
