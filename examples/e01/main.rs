use {
    anyhow::{Error, Result},
    ash::vk,
    ccthw::{
        application::{Application, GlfwWindow, State},
        graphics::vulkan_api::{
            CommandBuffer, CommandPool, Fence, Framebuffer, ImageView,
            RenderDevice, RenderPass, Semaphore, Swapchain, SwapchainStatus,
        },
        logging,
    },
    std::sync::Arc,
};

/// Example 1 is to use the Vulkan swapchain with a render pass to clear the
/// screen to a flat color.
///
/// This example uses a fence to ensure that only a single frame's commands are
/// ever executing at once. This is needlessly slow because it means Frame 0's
/// commands will block the submission of Frame 1's commands. The next example
/// shows how to use per-frame fences + semaphores to prevent this.
struct Example1ClearScreen {
    command_buffer: CommandBuffer,
    command_pool: Arc<CommandPool>,

    // Signalled when the swapchain image is ready for rendering
    acquire_semaphore: Semaphore,

    // Signalled when all graphics operations are complete and the frame is
    // ready to present.
    release_semaphore: Semaphore,

    // Signalled when all submitted graphics commands have completed for this
    // frame.
    queue_submit_fence: Fence,

    swapchain_image_views: Vec<Arc<ImageView>>,
    framebuffers: Vec<Framebuffer>,

    render_pass: Option<RenderPass>,

    swapchain_needs_rebuild: bool,
    swapchain: Option<Arc<Swapchain>>,
    render_device: Arc<RenderDevice>,
}

impl Example1ClearScreen {
    fn rebuild_swapchain_resources(
        &mut self,
        framebuffer_size: (i32, i32),
    ) -> Result<()> {
        // Stall the gpu so we can be sure no operations still depend on these
        // resources.
        self.render_device.wait_idle()?;

        self.swapchain_image_views.clear();
        self.framebuffers.clear();

        // Try to get exclusive ownership of the old swapchain if it exists.
        // If ownership cannot be taken it means some resource stil has an Arc
        // and we forgot to drop something. (this means a bug in the app logic)
        let old_swap = if let Some(swap_arc) = self.swapchain.take() {
            let swap_result = Arc::try_unwrap(swap_arc);
            if swap_result.is_err() {
                return Err(Error::msg("Unable to unwrap swapchain!"));
            }
            swap_result.ok()
        } else {
            None
        };

        // create a new swapchain
        let (w, h) = framebuffer_size;
        self.swapchain = Some(Arc::new(Swapchain::new(
            self.render_device.clone(),
            (w as u32, h as u32),
            old_swap,
        )?));

        // create a render pass which can target swapchain images
        self.render_pass = Some(RenderPass::single_sampled(
            self.render_device.clone(),
            self.swapchain.as_ref().unwrap().format(),
        )?);

        // build per-frame resources for each swapchain image
        let image_count =
            self.swapchain.as_ref().unwrap().swapchain_image_count();
        for index in 0..image_count {
            let swapchain_image_view =
                Arc::new(ImageView::for_swapchain_image(
                    self.render_device.clone(),
                    self.swapchain.as_ref().unwrap().clone(),
                    index as usize,
                )?);
            let framebuffer = Framebuffer::new(
                self.render_device.clone(),
                self.render_pass.as_ref().unwrap(),
                &[swapchain_image_view.clone()],
                self.swapchain.as_ref().unwrap().extent(),
            )?;
            self.framebuffers.push(framebuffer);
            self.swapchain_image_views.push(swapchain_image_view);
        }

        Ok(())
    }
}

impl State for Example1ClearScreen {
    fn new(window: &mut GlfwWindow) -> Result<Self> {
        window.window_handle.set_key_polling(true);

        let render_device = Arc::new(window.create_render_device()?);
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

        let acquire_semaphore = Semaphore::new(render_device.clone())?;
        let release_semaphore = Semaphore::new(render_device.clone())?;
        let queue_submit_fence = Fence::new(render_device.clone())?;

        Ok(Self {
            swapchain_needs_rebuild: true,
            framebuffers: vec![],
            swapchain_image_views: vec![],
            render_pass: None,
            swapchain: None,

            acquire_semaphore,
            release_semaphore,
            queue_submit_fence,
            command_pool,
            command_buffer,
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
                self.swapchain_needs_rebuild = true;
                //self.rebuild_swapchain((w, h))?;
            }
            _ => (),
        }
        Ok(())
    }

    fn update(&mut self, glfw_window: &mut GlfwWindow) -> Result<()> {
        if self.swapchain_needs_rebuild {
            self.swapchain_needs_rebuild = false;
            return self.rebuild_swapchain_resources(
                glfw_window.window_handle.get_framebuffer_size(),
            );
        }

        // Get Swapchain index
        // -------------------
        let result = self
            .swapchain
            .as_ref()
            .unwrap()
            .acquire_next_swapchain_image(
                Some(&self.acquire_semaphore),
                None,
            )?;

        let index = match result {
            SwapchainStatus::NeedsRebuild => {
                return self.rebuild_swapchain_resources(
                    glfw_window.window_handle.get_framebuffer_size(),
                );
            }
            SwapchainStatus::ImageAcquired(index) => index,
        };

        // Wait for the previous frame's commands to finish executing before
        // submitting anyithng for this frame.
        self.queue_submit_fence.wait_and_reset()?;

        // Safe because the queue submit fence ensures that the command buffer
        // is done executing before being reset.
        unsafe {
            self.command_pool.reset()?;
        }

        // draw frame
        // ----------
        self.command_buffer.begin_one_time_submit()?;

        // safe because the render pass and framebuffer will always outlive the
        // command buffer
        unsafe {
            self.command_buffer.begin_render_pass_inline(
                self.render_pass.as_ref().unwrap(),
                &self.framebuffers[index],
                self.swapchain.as_ref().unwrap().extent(),
                [0.0, 0.0, 1.0, 1.0],
            );
        }
        self.command_buffer.end_render_pass();

        self.command_buffer.end_command_buffer()?;

        unsafe {
            self.command_buffer.submit_graphics_commands(
                &[&self.acquire_semaphore],
                &[vk::PipelineStageFlags::COLOR_ATTACHMENT_OUTPUT],
                &[&self.release_semaphore],
                Some(&self.queue_submit_fence),
            )?;
        }

        // present swapchain image
        // -----------------------

        self.swapchain
            .as_ref()
            .unwrap()
            .present_swapchain_image(index, &self.release_semaphore)?;

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
    let _logger = logging::setup()?;
    Application::<Example1ClearScreen>::new("Example 1 - Clear Screen")?.run()
}
