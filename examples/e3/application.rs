//! This module defines the main application initialization, event loop, and
//! rendering.

mod per_frame;
mod semaphore_pool;

use per_frame::PerFrame;
use semaphore_pool::SemaphorePool;

use anyhow::{Context, Result};
use ash::{version::DeviceV1_0, vk};
use ccthw::{glfw_window::GlfwWindow, vulkan, vulkan::errors::SwapchainError};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum RenderError {
    #[error("The swapchain needs to be rebuilt")]
    SwapchainNeedsRebuild,

    #[error(transparent)]
    UnexpectedRuntimeError(#[from] anyhow::Error),
}

// The main application state.
pub struct Application {
    per_frame: Vec<PerFrame>,
    semaphore_pool: SemaphorePool,
    vk_dev: vulkan::RenderDevice,
    glfw_window: GlfwWindow,
}

impl Application {
    /// Build a new instance of the application.
    pub fn new() -> Result<Self> {
        let mut glfw_window = GlfwWindow::new("GLFW Lib")?;
        glfw_window.window.set_key_polling(true);
        glfw_window.window.set_framebuffer_size_polling(true);

        let vk_dev = glfw_window.create_vulkan_device()?;
        let semaphore_pool = SemaphorePool::new();

        let mut per_frame = vec![];
        for i in 0..vk_dev.swapchain.as_ref().unwrap().image_views.len() {
            per_frame.push(PerFrame::new(&vk_dev, i)?);
        }

        Ok(Self {
            per_frame,
            semaphore_pool,
            vk_dev,
            glfw_window,
        })
    }

    /// Run the application, blocks until the main event loop exits.
    pub fn run(mut self) -> Result<()> {
        let event_receiver = self.glfw_window.take_event_receiver()?;
        while !self.glfw_window.window.should_close() {
            for (_, event) in
                self.glfw_window.flush_window_events(&event_receiver)
            {
                self.handle_event(event)?;
            }
            let result = self.render();
            match result {
                Err(RenderError::SwapchainNeedsRebuild) => {
                    self.rebuild_swapchain_resources()?;
                }
                _ => result?,
            }
            std::thread::sleep(std::time::Duration::from_millis(100));
        }
        Ok(())
    }

    fn render(&mut self) -> Result<(), RenderError> {
        let index = self.acquire_next_image()?;
        self.draw_frame(index)?;
        self.present_image(index)?;
        Ok(())
    }

    /// Draw a single frame.
    fn acquire_next_image(&mut self) -> Result<usize, RenderError> {
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
                return Err(RenderError::SwapchainNeedsRebuild);
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

            // do something here

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
        unsafe {
            self.vk_dev.logical_device.device_wait_idle()?;
        }
        let (width, height) = self.glfw_window.window.get_framebuffer_size();
        self.vk_dev
            .rebuild_swapchain((width as u32, height as u32))?;
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
            WindowEvent::FramebufferSize(..) => {
                self.rebuild_swapchain_resources()?;
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
        for per_frame in self.per_frame.drain(..) {
            per_frame.destroy(&self.vk_dev);
        }
        self.semaphore_pool.destroy(&self.vk_dev);
    }
}
