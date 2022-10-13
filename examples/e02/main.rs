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

/// This example uses the Vulkan swapchain + render pass to clear the screen.
///
/// Unlike the previous example, each frame has a separate queue submission
/// fence and semaphores. This means that Frame 0's commands and rasterization
/// will not block Frame 1's submission. The only time the per-frame fences
/// will stall is if the frame's previous submission is still executing by the
/// time the swapchain picks it for presentation.
struct Example2MultipleFrames {
    frames: Vec<Frame>,

    render_pass: Option<RenderPass>,
    semaphore_pool: SemaphorePool,

    swapchain_needs_rebuild: bool,
    swapchain: Option<Arc<Swapchain>>,
    render_device: Arc<RenderDevice>,
}

impl Example2MultipleFrames {
    fn rebuild_swapchain_resources(
        &mut self,
        framebuffer_size: (i32, i32),
    ) -> Result<()> {
        // Stall the gpu so we can be sure no operations still depend on these
        // resources.
        self.render_device.wait_idle()?;

        // Drop all per-frame resources. The number of swapchain images and
        // format could change which will require these to be rebuilt anyways.
        self.frames.clear();

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
            self.frames.push(Frame::new(
                &self.render_device,
                self.render_pass.as_ref().unwrap(),
                &mut self.semaphore_pool,
                self.swapchain.as_ref().unwrap(),
                index as usize,
            )?);
        }

        Ok(())
    }
}

impl State for Example2MultipleFrames {
    fn new(window: &mut GlfwWindow) -> Result<Self> {
        window.window_handle.set_key_polling(true);

        let render_device = Arc::new(window.create_render_device()?);
        let semaphore_pool = SemaphorePool::new(render_device.clone());

        Ok(Self {
            frames: vec![],
            render_pass: None,
            semaphore_pool,
            swapchain_needs_rebuild: true,
            swapchain: None,
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
        let acquire_semaphore = self.semaphore_pool.get_semaphore()?;
        let result = self
            .swapchain
            .as_ref()
            .unwrap()
            .acquire_next_swapchain_image(Some(&acquire_semaphore), None)?;

        let index = match result {
            SwapchainStatus::NeedsRebuild => {
                return self.rebuild_swapchain_resources(
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

        let current_frame = &self.frames[index];

        // Ideally this doesn't make the program stall for very long because
        // other frames have been rendered between now and when this image
        // was last used.
        current_frame.queue_submit_fence.wait_and_reset()?;

        // safe because the queue submit fence ensures that the command buffer
        // is done executing before being reset
        unsafe {
            current_frame.command_pool.reset()?;
        }

        // draw frame
        // ----------
        current_frame.command_buffer.begin_one_time_submit()?;

        // safe because the render pass and framebuffer will always outlive the
        // command buffer
        unsafe {
            current_frame.command_buffer.begin_render_pass_inline(
                self.render_pass.as_ref().unwrap(),
                &current_frame.framebuffer,
                self.swapchain.as_ref().unwrap().extent(),
                [0.0, 0.0, 1.0, 1.0],
            );
        }
        current_frame.command_buffer.end_render_pass();

        current_frame.command_buffer.end_command_buffer()?;

        unsafe {
            current_frame.command_buffer.submit_graphics_commands(
                &[current_frame.acquire_semaphore.as_ref().unwrap()],
                &[vk::PipelineStageFlags::COLOR_ATTACHMENT_OUTPUT],
                &[&current_frame.release_semaphore],
                Some(&current_frame.queue_submit_fence),
            )?;
        }

        // present swapchain image
        // -----------------------

        self.swapchain
            .as_ref()
            .unwrap()
            .present_swapchain_image(index, &current_frame.release_semaphore)?;

        Ok(())
    }
}

impl Drop for Example2MultipleFrames {
    fn drop(&mut self) {
        self.render_device
            .wait_idle()
            .expect("Unable to wait for the device to idle");
    }
}

fn main() -> Result<()> {
    let _logger = logging::setup()?;
    Application::<Example2MultipleFrames>::new("Example 2 - Multiple Frames")?
        .run()
}
