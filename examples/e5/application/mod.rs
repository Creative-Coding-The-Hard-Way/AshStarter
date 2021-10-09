//! This module defines the main application initialization, event loop, and
//! rendering.

mod per_frame;

use per_frame::PerFrame;

use ccthw::{
    glfw_window::GlfwWindow,
    renderer::{ClearFrame, FinishFrame, Renderer},
    timing::FrameRateLimit,
    vulkan,
    vulkan::{errors::SwapchainError, SemaphorePool},
};

use anyhow::{Context, Result};
use ash::{version::DeviceV1_0, vk};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum FrameError {
    #[error("The swapchain needs to be rebuilt")]
    SwapchainNeedsRebuild,

    #[error(transparent)]
    UnexpectedRuntimeError(#[from] anyhow::Error),
}

// The main application state.
pub struct Application {
    // renderers
    clear_frame: ClearFrame,
    finish_frame: FinishFrame,

    per_frame: Vec<PerFrame>,
    semaphore_pool: SemaphorePool,
    vk_dev: vulkan::RenderDevice,
    glfw_window: GlfwWindow,
    fps_limit: FrameRateLimit,

    paused: bool,
    swapchain_needs_rebuild: bool,
}

impl Application {
    /// Build a new instance of the application.
    pub fn new() -> Result<Self> {
        let mut glfw_window = GlfwWindow::new("Swapchain")?;
        glfw_window.window.set_key_polling(true);
        glfw_window.window.set_framebuffer_size_polling(true);

        // Create the vulkan render device
        let vk_dev = glfw_window.create_vulkan_device()?;
        let semaphore_pool = SemaphorePool::new();

        // build per-frame resources
        let mut per_frame = vec![];
        for i in 0..vk_dev.swapchain.as_ref().unwrap().image_views.len() {
            per_frame.push(PerFrame::new(&vk_dev, i)?);
        }

        Ok(Self {
            clear_frame: ClearFrame::new(&vk_dev, [0.0, 0.0, 1.0, 1.0])?,
            finish_frame: FinishFrame::new(&vk_dev)?,

            per_frame,
            semaphore_pool,
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
                let result = self.render();
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
    fn render(&mut self) -> Result<(), FrameError> {
        // 1. Acquire the next image from the swapchain and update any related
        //    per-frame resources.
        let index = self.acquire_next_image()?;

        // 2. Draw a single frame. This means build and submit a command-buffer
        //    to the graphics queue.
        self.draw_frame(index)?;

        // 3. Present the swapchain image to the screen. This blocks on the
        //    frame's semaphore which indicates that the graphics operations
        //    have been completed.
        self.present_image(index)?;
        Ok(())
    }

    /// Draw a single frame.
    fn acquire_next_image(&mut self) -> Result<usize, FrameError> {
        let acquire_semaphore =
            self.semaphore_pool.get_semaphore(&self.vk_dev).context(
                "unable to get a semaphore for the next swapchain image",
            )?;
        let index = {
            let result = self.vk_dev.acquire_next_swapchain_image(
                acquire_semaphore,
                vk::Fence::null(),
            );
            if result.is_err() {
                self.semaphore_pool.return_semaphore(acquire_semaphore);
            }
            if let Err(SwapchainError::NeedsRebuild) = result {
                return Err(FrameError::SwapchainNeedsRebuild);
            }
            result.context("unable to acquire the next swapchain image")?
        };

        // Replace the old acquire_semaphore with the new one which will be
        // signaled when this frame is ready.
        self.semaphore_pool
            .return_semaphore(self.per_frame[index].acquire_semaphore);
        self.per_frame[index].acquire_semaphore = acquire_semaphore;

        // This typically is a no-op because multiple other frames have been
        // rendered between this time and the last time the frame was rendered.
        if self.per_frame[index].queue_submit_fence != vk::Fence::null() {
            unsafe {
                self.vk_dev
                    .logical_device
                    .wait_for_fences(
                        &[self.per_frame[index].queue_submit_fence],
                        true,
                        u64::MAX,
                    )
                    .context("error waiting for queue submission fence")?;
                self.vk_dev
                    .logical_device
                    .reset_fences(&[self.per_frame[index].queue_submit_fence])
                    .context("unable to reset queue submission fence")?;
            }
        }

        unsafe {
            self.vk_dev
                .logical_device
                .reset_command_pool(
                    self.per_frame[index].command_pool,
                    vk::CommandPoolResetFlags::empty(),
                )
                .context("unable to reset the frame command pool")?;
        }

        Ok(index)
    }

    fn draw_frame(&mut self, index: usize) -> Result<()> {
        let current_frame = &self.per_frame[index];

        // build the command buffer
        unsafe {
            let begin_info = vk::CommandBufferBeginInfo {
                flags: vk::CommandBufferUsageFlags::ONE_TIME_SUBMIT,
                ..Default::default()
            };
            self.vk_dev.logical_device.begin_command_buffer(
                current_frame.command_buffer,
                &begin_info,
            )?;

            self.clear_frame.fill_command_buffer(
                &self.vk_dev,
                current_frame.command_buffer,
                index as u32,
            )?;
            self.finish_frame.fill_command_buffer(
                &self.vk_dev,
                current_frame.command_buffer,
                index as u32,
            )?;

            self.vk_dev
                .logical_device
                .end_command_buffer(current_frame.command_buffer)?;
        }

        // submit the command buffer
        let wait_stage = vk::PipelineStageFlags::COLOR_ATTACHMENT_OUTPUT;
        let submit_info = vk::SubmitInfo {
            command_buffer_count: 1,
            p_command_buffers: &current_frame.command_buffer,
            wait_semaphore_count: 1,
            p_wait_semaphores: &current_frame.acquire_semaphore,
            p_wait_dst_stage_mask: &wait_stage,
            signal_semaphore_count: 1,
            p_signal_semaphores: &current_frame.release_semaphore,
            ..Default::default()
        };
        unsafe {
            self.vk_dev.logical_device.queue_submit(
                self.vk_dev.graphics_queue.queue,
                &[submit_info],
                current_frame.queue_submit_fence,
            )?;
        }

        Ok(())
    }

    fn present_image(&mut self, index: usize) -> Result<()> {
        let index_u32 = index as u32;
        let current_frame = &self.per_frame[index];
        let present_info = vk::PresentInfoKHR {
            swapchain_count: 1,
            p_swapchains: &self.vk_dev.swapchain().khr,
            p_image_indices: &index_u32,
            wait_semaphore_count: 1,
            p_wait_semaphores: &current_frame.release_semaphore,
            ..Default::default()
        };
        unsafe {
            self.vk_dev.swapchain().loader.queue_present(
                self.vk_dev.present_queue.queue,
                &present_info,
            )?;
        }
        Ok(())
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
        let (width, height) = self.glfw_window.window.get_framebuffer_size();
        self.vk_dev
            .rebuild_swapchain((width as u32, height as u32))?;
        unsafe {
            self.clear_frame.rebuild_swapchain_resources(&self.vk_dev)?;
            self.finish_frame
                .rebuild_swapchain_resources(&self.vk_dev)?;
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
            self.clear_frame.destroy(&self.vk_dev);
            self.finish_frame.destroy(&self.vk_dev);
        }
        for per_frame in self.per_frame.drain(..) {
            per_frame.destroy(&self.vk_dev);
        }
        self.semaphore_pool.destroy(&self.vk_dev);
    }
}
