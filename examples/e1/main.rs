use std::sync::Arc;

use anyhow::{Error, Result};
use ash::vk;
use ccthw::{
    application::{Application, GlfwWindow, State},
    graphics::vulkan_api::{
        CommandBuffer, CommandPool, Fence, Framebuffer, ImageView,
        RenderDevice, RenderPass, Semaphore, SemaphorePool, Swapchain,
        SwapchainStatus, VulkanError,
    },
    logging,
};

/// It's useful and convenient to organize some resources to have a separate
/// instance for each swapchain image. These are the resources which have to be
/// duplicated for each frame. Hence, "PerFrame".
struct Frame {
    command_buffer: CommandBuffer,
    command_pool: Arc<CommandPool>,

    // per-frame resources
    // Signalled when the swapchain image is ready for rendering
    acquire_semaphore: Option<Semaphore>,

    // Signalled when all graphics operations are complete and the frame is
    // ready to present.
    release_semaphore: Semaphore,

    // Signalled when all submitted graphics commands have completed for this
    // frame.
    queue_submit_fence: Fence,

    // A view of the swapchain image used by this frame.
    _swapchain_image_view: Arc<ImageView>,

    // The framebuffer which targets this frame's swapchain image.
    framebuffer: Framebuffer,
}

impl Frame {
    fn new(
        render_device: &Arc<RenderDevice>,
        render_pass: &RenderPass,
        semaphore_pool: &mut SemaphorePool,
        swapchain: &Arc<Swapchain>,
        swapchain_image_index: usize,
    ) -> Result<Self, VulkanError> {
        let acquire_semaphore = None;
        let release_semaphore = semaphore_pool.get_semaphore()?;
        let queue_submit_fence = Fence::new(render_device.clone())?;
        let swapchain_image_view = Arc::new(ImageView::for_swapchain_image(
            render_device.clone(),
            swapchain.clone(),
            swapchain_image_index,
        )?);
        let framebuffer = Framebuffer::new(
            render_device.clone(),
            render_pass,
            &[swapchain_image_view.clone()],
            swapchain.extent(),
        )?;
        let command_pool = Arc::new(CommandPool::new(
            render_device.clone(),
            render_device.graphics_queue_family_index(),
            vk::CommandPoolCreateFlags::empty(),
        )?);
        let command_buffer = CommandBuffer::new(
            render_device.clone(),
            command_pool.clone(),
            vk::CommandBufferLevel::PRIMARY,
        )?;
        Ok(Self {
            command_buffer,
            command_pool,
            acquire_semaphore,
            release_semaphore,
            queue_submit_fence,
            _swapchain_image_view: swapchain_image_view,
            framebuffer,
        })
    }
}

/// Example 1 is to use the Vulkan swapchain with a render pass to clear the
/// screen to a flat color.
struct Example1ClearScreen {
    frames: Vec<Frame>,

    render_pass: RenderPass,
    semaphore_pool: SemaphorePool,

    swapchain: Option<Arc<Swapchain>>,
    render_device: Arc<RenderDevice>,
}

impl Example1ClearScreen {
    fn rebuild_swapchain(
        &mut self,
        framebuffer_size: (i32, i32),
    ) -> Result<()> {
        let (w, h) = framebuffer_size;
        self.render_device.wait_idle()?;
        self.frames.clear();

        let old_swap = if let Some(swap_arc) = self.swapchain.take() {
            let swap_result = Arc::try_unwrap(swap_arc);
            if swap_result.is_err() {
                return Err(Error::msg("Unable to unwrap swapchain!"));
            }
            swap_result.ok()
        } else {
            None
        };
        self.swapchain = Some(Arc::new(Swapchain::new(
            self.render_device.clone(),
            (w as u32, h as u32),
            old_swap,
        )?));
        self.render_pass = RenderPass::single_sampled(
            self.render_device.clone(),
            self.swapchain.as_ref().unwrap().format(),
        )?;

        let image_count =
            self.swapchain.as_ref().unwrap().swapchain_image_count();
        for index in 0..image_count {
            self.frames.push(Frame::new(
                &self.render_device,
                &self.render_pass,
                &mut self.semaphore_pool,
                self.swapchain.as_ref().unwrap(),
                index as usize,
            )?);
        }

        Ok(())
    }
}

impl State for Example1ClearScreen {
    fn new(window: &mut GlfwWindow) -> Result<Self> {
        window.window_handle.set_key_polling(true);

        let (w, h) = window.window_handle.get_framebuffer_size();
        let render_device = Arc::new(window.create_render_device()?);
        let mut semaphore_pool = SemaphorePool::new(render_device.clone());
        let swapchain = Arc::new(Swapchain::new(
            render_device.clone(),
            (w as u32, h as u32),
            None,
        )?);
        let render_pass = RenderPass::single_sampled(
            render_device.clone(),
            swapchain.format(),
        )?;
        let mut frames = vec![];
        for index in 0..swapchain.swapchain_image_count() {
            frames.push(Frame::new(
                &render_device,
                &render_pass,
                &mut semaphore_pool,
                &swapchain,
                index as usize,
            )?);
        }

        Ok(Self {
            frames,
            render_pass,
            semaphore_pool,
            swapchain: Some(swapchain),
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
            WindowEvent::FramebufferSize(w, h) => {
                self.rebuild_swapchain((w, h))?;
            }
            _ => (),
        }
        Ok(())
    }

    fn update(&mut self, glfw_window: &mut GlfwWindow) -> Result<()> {
        // Get Swapchain index
        // -------------------
        let acquire_semaphore = self.semaphore_pool.get_semaphore()?;
        let result = self
            .swapchain
            .as_ref()
            .unwrap()
            .acquire_next_swapchain_image(Some(&acquire_semaphore), None)?;

        let index = match result {
            SwapchainStatus::NeedsRebuild => {
                return self.rebuild_swapchain(
                    glfw_window.window_handle.get_framebuffer_size(),
                );
            }
            SwapchainStatus::ImageAcquired(index) => {
                let old_semaphore = self.frames[index]
                    .acquire_semaphore
                    .replace(acquire_semaphore);
                if let Some(semaphore) = old_semaphore {
                    self.semaphore_pool.return_semaphore(semaphore);
                }
                index
            }
        };

        self.frames[index].queue_submit_fence.wait_and_reset()?;

        // safe because the queue submit fence ensures that the command buffer
        // is done executing before being reset
        unsafe {
            self.frames[index].command_pool.reset()?;
        }

        // draw frame
        // ----------
        self.frames[index].command_buffer.begin_one_time_submit()?;

        // safe because the render pass and framebuffer will always outlive the
        // command buffer
        unsafe {
            self.frames[index].command_buffer.begin_render_pass_inline(
                &self.render_pass,
                &self.frames[index].framebuffer,
                self.swapchain.as_ref().unwrap().extent(),
                [0.0, 0.0, 1.0, 1.0],
            );
        }
        self.frames[index].command_buffer.end_render_pass();

        self.frames[index].command_buffer.end_command_buffer()?;

        unsafe {
            self.frames[index].command_buffer.submit_graphics_commands(
                &[self.frames[index].acquire_semaphore.as_ref().unwrap()],
                &[vk::PipelineStageFlags::COLOR_ATTACHMENT_OUTPUT],
                &[&self.frames[index].release_semaphore],
                Some(&self.frames[index].queue_submit_fence),
            )?;
        }

        // present swapchain image
        // -----------------------

        self.swapchain.as_ref().unwrap().present_swapchain_image(
            index,
            &self.frames[index].release_semaphore,
        )?;

        Ok(())
    }
}

impl Drop for Example1ClearScreen {
    fn drop(&mut self) {
        self.render_device
            .wait_idle()
            .expect("Unable to wait for the device to idle");
    }
}

fn main() -> Result<()> {
    logging::setup()?;
    Application::<Example1ClearScreen>::new("Example 1 - Clear Screen")?.run()
}
