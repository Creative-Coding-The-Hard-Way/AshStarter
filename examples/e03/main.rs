use {
    anyhow::Result,
    ccthw::{
        application::{Application, GlfwWindow, State},
        graphics::{
            vulkan_api::{Framebuffer, RenderDevice, RenderPass},
            AcquiredFrame, SwapchainFrames,
        },
    },
    std::sync::Arc,
};

/// This example uses SwapchainFrames type to manage the swapchain and
/// per-frame synchronization.
struct Example3SwapchainFrames {
    swapchain_frames: SwapchainFrames,
    framebuffers: Vec<Framebuffer>,
    render_pass: Option<RenderPass>,
    render_device: Arc<RenderDevice>,
}

impl Example3SwapchainFrames {
    fn build_swapchain_resources(
        &mut self,
        framebuffer_size: (i32, i32),
    ) -> Result<()> {
        self.swapchain_frames.wait_for_all_frames_to_complete()?;
        self.framebuffers.clear();
        self.swapchain_frames.rebuild_swapchain(framebuffer_size)?;

        self.render_pass = Some(RenderPass::single_sampled(
            self.render_device.clone(),
            self.swapchain_frames.swapchain().format(),
        )?);

        let extent = self.swapchain_frames.swapchain().extent();
        for i in 0..self.swapchain_frames.swapchain_image_count() {
            let image_view = self.swapchain_frames.swapchain_image_view(i)?;
            self.framebuffers.push(Framebuffer::new(
                self.render_device.clone(),
                self.render_pass.as_ref().unwrap(),
                &[image_view.clone()],
                extent,
            )?);
        }

        Ok(())
    }
}

impl State for Example3SwapchainFrames {
    fn new(window: &mut GlfwWindow) -> Result<Self> {
        window.window_handle.set_key_polling(true);

        let render_device = Arc::new(window.create_render_device()?);
        let swapchain_frames = SwapchainFrames::new(render_device.clone())?;

        Ok(Self {
            framebuffers: vec![],
            render_pass: None,
            swapchain_frames,
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
                self.swapchain_frames.invalidate_swapchain();
            }
            _ => (),
        }
        Ok(())
    }

    fn update(&mut self, glfw_window: &mut GlfwWindow) -> Result<()> {
        let mut frame = match self.swapchain_frames.acquire_swapchain_frame()? {
            AcquiredFrame::SwapchainNeedsRebuild => {
                return self.build_swapchain_resources(
                    glfw_window.window_handle.get_framebuffer_size(),
                );
            }
            AcquiredFrame::Available(frame) => frame,
        };

        // safe because the render pass and framebuffer will always outlive the
        // command buffer
        unsafe {
            let framebuffer = &self.framebuffers[frame.swapchain_image_index()];
            frame.command_buffer().begin_render_pass_inline(
                self.render_pass.as_ref().unwrap(),
                framebuffer,
                self.swapchain_frames.swapchain().extent(),
                [0.0, 0.0, 1.0, 1.0],
            );
        }
        frame.command_buffer().end_render_pass();

        self.swapchain_frames.present_frame(frame)?;

        Ok(())
    }
}

impl Drop for Example3SwapchainFrames {
    fn drop(&mut self) {
        self.render_device
            .wait_idle()
            .expect("Unable to wait for the device to idle");
    }
}

fn main() -> Result<()> {
    Application::<Example3SwapchainFrames>::new("Example 3- SwapchainFrames")?
        .run()
}
