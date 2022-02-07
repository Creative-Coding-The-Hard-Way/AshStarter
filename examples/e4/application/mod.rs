//! This module defines the main application initialization, event loop, and
//! rendering.

mod per_frame;
mod pipeline;
mod renderpass;

use per_frame::PerFrame;
use ::{
    anyhow::{Context, Result},
    ash::vk,
    ccthw::{
        glfw_window::GlfwWindow,
        timing::FrameRateLimit,
        vulkan,
        vulkan::{
            errors::SwapchainError, sync::SemaphorePool, Framebuffer, GpuVec,
            Pipeline, PipelineLayout, RenderPass, VulkanDebug,
        },
    },
    std::sync::Arc,
    thiserror::Error,
};

#[derive(Debug, Error)]
pub enum FrameError {
    #[error("The swapchain needs to be rebuilt")]
    SwapchainNeedsRebuild,

    #[error(transparent)]
    UnexpectedRuntimeError(#[from] anyhow::Error),
}

#[derive(Debug, Copy, Clone, PartialEq)]
pub struct Vertex {
    pub pos: [f32; 2],
    pub rgba: [f32; 4],
}

// The main application state.
pub struct Application {
    fps_limit: FrameRateLimit,

    // rendering resources
    pipeline_layout: PipelineLayout,
    pipeline: Pipeline,
    framebuffers: Vec<Framebuffer>,
    render_pass: RenderPass,
    per_frame: Vec<PerFrame>,
    vertex_data: GpuVec<Vertex>,

    // system resources
    semaphore_pool: SemaphorePool,
    vk_dev: Arc<vulkan::RenderDevice>,
    glfw_window: GlfwWindow,

    // app state
    paused: bool,
    needs_swapchain_rebuild: bool,
}

impl Application {
    /// Build a new instance of the application.
    pub fn new() -> Result<Self> {
        let mut glfw_window = GlfwWindow::new("First Triangle")?;
        glfw_window.window.set_key_polling(true);
        glfw_window.window.set_framebuffer_size_polling(true);

        // Create the vulkan render device
        let vk_dev = Arc::new(glfw_window.create_vulkan_device()?);
        let semaphore_pool = SemaphorePool::new(vk_dev.clone());

        // build per-frame resources
        let mut per_frame = vec![];
        for i in 0..vk_dev.swapchain_image_count() {
            let frame = PerFrame::new(vk_dev.clone())?;
            frame.set_debug_name(format!("Frame {}", i))?;
            per_frame.push(frame);
        }

        // create a render pass
        let render_pass = renderpass::create(vk_dev.clone())?;
        render_pass.set_debug_name("Application Render Pass")?;

        let framebuffers = Framebuffer::with_swapchain_color_attachments(
            vk_dev.clone(),
            render_pass.raw,
            "Application Framebuffer",
        )?;

        let vk_alloc = vulkan::create_default_allocator(vk_dev.clone());

        let mut vertex_data = GpuVec::new(
            vk_dev.clone(),
            vk_alloc.clone(),
            vk::BufferUsageFlags::VERTEX_BUFFER,
            3,
        )?;
        vertex_data.push_back(Vertex {
            pos: [0.0, 0.5],
            rgba: [0.2, 0.2, 0.8, 1.0],
        })?;
        vertex_data.push_back(Vertex {
            pos: [0.5, -0.5],
            rgba: [0.2, 0.2, 0.8, 1.0],
        })?;
        vertex_data.push_back(Vertex {
            pos: [-0.5, -0.5],
            rgba: [0.2, 0.2, 0.8, 1.0],
        })?;
        vertex_data.set_debug_name("Vertex Data")?;

        let (pipeline, pipeline_layout) =
            pipeline::create_pipeline(vk_dev.clone(), render_pass.raw)?;

        Ok(Self {
            fps_limit: FrameRateLimit::new(120, 10),
            per_frame,
            semaphore_pool,
            vk_dev,
            glfw_window,
            render_pass,
            framebuffers,
            vertex_data,
            pipeline,
            pipeline_layout,
            paused: false,
            needs_swapchain_rebuild: false,
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
            if self.needs_swapchain_rebuild {
                self.rebuild_swapchain_resources()?;
                self.needs_swapchain_rebuild = false;
            }
            if !self.paused {
                let result = self.render();
                match result {
                    Err(FrameError::SwapchainNeedsRebuild) => {
                        self.needs_swapchain_rebuild = true;
                    }
                    _ => result?,
                }
            }
            self.fps_limit.sleep_to_limit();
            //log::debug!("Frame Time {:?}", self.fps_limit.avg_frame_time());
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

    fn acquire_next_image(&mut self) -> Result<usize, FrameError> {
        let acquire_semaphore = self.semaphore_pool.get_semaphore().context(
            "unable to get a semaphore for the next swapchain image",
        )?;
        let index = {
            let result = self.vk_dev.acquire_next_swapchain_image(
                acquire_semaphore.raw,
                vk::Fence::null(),
            );
            if let Err(SwapchainError::NeedsRebuild) = result {
                return Err(FrameError::SwapchainNeedsRebuild);
            }
            result.context("unable to acquire the next swapchain image")?
        };

        // Replace the old acquire_semaphore with the new one which will be
        // signaled when this frame is ready.
        let old_semaphore = self.per_frame[index]
            .acquire_semaphore
            .replace(acquire_semaphore);
        if let Some(semaphore) = old_semaphore {
            self.semaphore_pool.return_semaphore(semaphore)
        }

        // This typically is a no-op because multiple other frames have been
        // rendered between this time and the last time the frame was rendered.
        self.per_frame[index]
            .queue_submit_fence
            .wait_and_reset()
            .with_context(|| {
                format!(
                    "error while waiting for frame[{}]'s fence to reset",
                    index
                )
            })?;
        self.per_frame[index]
            .command_pool
            .reset()
            .with_context(|| {
                format!("error while resetting frame[{}]'s command pool", index)
            })?;

        Ok(index)
    }

    fn draw_frame(&mut self, index: usize) -> Result<()> {
        let current_frame = &self.per_frame[index];
        let extent = self.vk_dev.with_swapchain(|swapchain| swapchain.extent);

        // build the command buffer
        unsafe {
            let begin_info = vk::CommandBufferBeginInfo {
                flags: vk::CommandBufferUsageFlags::ONE_TIME_SUBMIT,
                ..Default::default()
            };
            self.vk_dev.logical_device.begin_command_buffer(
                current_frame.command_buffer.raw,
                &begin_info,
            )?;

            let clear_values = [vk::ClearValue {
                color: vk::ClearColorValue {
                    float32: [1.0, 1.0, 1.0, 1.0],
                },
            }];
            let render_pass_begin_info = vk::RenderPassBeginInfo {
                render_pass: self.render_pass.raw,
                framebuffer: self.framebuffers[index].raw,
                render_area: vk::Rect2D {
                    offset: vk::Offset2D { x: 0, y: 0 },
                    extent,
                },
                clear_value_count: 1,
                p_clear_values: clear_values.as_ptr(),
                ..Default::default()
            };
            self.vk_dev.logical_device.cmd_begin_render_pass(
                current_frame.command_buffer.raw,
                &render_pass_begin_info,
                vk::SubpassContents::INLINE,
            );
            self.vk_dev.logical_device.cmd_bind_pipeline(
                current_frame.command_buffer.raw,
                vk::PipelineBindPoint::GRAPHICS,
                self.pipeline.raw,
            );
            self.vk_dev.logical_device.cmd_bind_vertex_buffers(
                current_frame.command_buffer.raw,
                0,
                &[self.vertex_data.buffer.raw],
                &[0],
            );
            self.vk_dev.logical_device.cmd_draw(
                current_frame.command_buffer.raw,
                self.vertex_data.len() as u32, // vertex count
                1,                             // instance count
                0,                             // first vertex index
                0,                             // first instance
            );

            // do something here

            self.vk_dev
                .logical_device
                .cmd_end_render_pass(current_frame.command_buffer.raw);

            self.vk_dev
                .logical_device
                .end_command_buffer(current_frame.command_buffer.raw)?;
        }

        // submit the command buffer
        let wait_stage = vk::PipelineStageFlags::COLOR_ATTACHMENT_OUTPUT;
        let submit_info = vk::SubmitInfo {
            command_buffer_count: 1,
            p_command_buffers: &current_frame.command_buffer.raw,
            wait_semaphore_count: 1,
            p_wait_semaphores: &current_frame
                .acquire_semaphore
                .as_ref()
                .unwrap()
                .raw,
            p_wait_dst_stage_mask: &wait_stage,
            signal_semaphore_count: 1,
            p_signal_semaphores: &current_frame.release_semaphore.raw,
            ..Default::default()
        };
        unsafe {
            self.vk_dev.logical_device.queue_submit(
                self.vk_dev.graphics_queue.queue,
                &[submit_info],
                current_frame.queue_submit_fence.raw,
            )?;
        }

        Ok(())
    }

    fn present_image(&mut self, index: usize) -> Result<()> {
        let index_u32 = index as u32;
        let current_frame = &self.per_frame[index];
        let present_queue = &self.vk_dev.present_queue;

        self.vk_dev.with_swapchain(|swapchain| {
            let present_info = vk::PresentInfoKHR {
                swapchain_count: 1,
                p_swapchains: &swapchain.khr,
                p_image_indices: &index_u32,
                wait_semaphore_count: 1,
                p_wait_semaphores: &current_frame.release_semaphore.raw,
                ..Default::default()
            };
            unsafe {
                swapchain
                    .loader
                    .queue_present(present_queue.queue, &present_info)
            }
        })?;
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
            self.per_frame.clear();
            self.framebuffers.clear();
        }
        let (w, h) = self.glfw_window.window.get_framebuffer_size();
        self.vk_dev.rebuild_swapchain((w as u32, h as u32))?;

        let render_pass = renderpass::create(self.vk_dev.clone())?;
        render_pass.set_debug_name("Application RenderPass")?;

        self.framebuffers = Framebuffer::with_swapchain_color_attachments(
            self.vk_dev.clone(),
            self.render_pass.raw,
            "Application Framebuffer",
        )?;

        for i in 0..self.vk_dev.swapchain_image_count() {
            let frame = PerFrame::new(self.vk_dev.clone())?;
            frame.set_debug_name(format!("Frame {}", i))?;
            self.per_frame.push(frame);
        }

        let (pipeline, pipeline_layout) = pipeline::create_pipeline(
            self.vk_dev.clone(),
            self.render_pass.raw,
        )?;
        self.pipeline = pipeline;
        self.pipeline_layout = pipeline_layout;

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
                self.needs_swapchain_rebuild = true;
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
