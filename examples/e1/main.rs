use std::sync::Arc;

use anyhow::Result;
use ccthw::{
    application::{Application, GlfwWindow, State},
    graphics::vulkan_api::{
        RenderDevice, Semaphore, SemaphorePool, Swapchain, SwapchainStatus,
    },
    logging,
};

/// Example 1 is to use the Vulkan swapchain with a render pass to clear the
/// screen to a flat color.
struct Example1ClearScreen {
    semaphore_pool: SemaphorePool,
    per_frame_acquire_semaphore: Vec<Option<Semaphore>>,
    swapchain: Option<Swapchain>,
    render_device: Arc<RenderDevice>,
}

impl Example1ClearScreen {
    fn rebuild_swapchain(
        &mut self,
        framebuffer_size: (i32, i32),
    ) -> Result<()> {
        let (w, h) = framebuffer_size;
        self.render_device.wait_idle()?;
        self.swapchain = Some(Swapchain::new(
            self.render_device.clone(),
            (w as u32, h as u32),
            self.swapchain.take(),
        )?);
        Ok(())
    }
}

impl State for Example1ClearScreen {
    fn new(window: &mut GlfwWindow) -> Result<Self> {
        window.window_handle.set_key_polling(true);

        let (w, h) = window.window_handle.get_framebuffer_size();
        let render_device = Arc::new(window.create_render_device()?);
        let swapchain =
            Swapchain::new(render_device.clone(), (w as u32, h as u32), None)?;

        let mut semaphore_pool = SemaphorePool::new(render_device.clone());

        let mut per_frame_acquire_semaphore = vec![];
        for _ in 0..swapchain.swapchain_image_count() {
            per_frame_acquire_semaphore
                .push(Some(semaphore_pool.get_semaphore()?));
        }

        Ok(Self {
            semaphore_pool,
            per_frame_acquire_semaphore,
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
                let old_semaphore = self.per_frame_acquire_semaphore[index]
                    .replace(acquire_semaphore);
                if let Some(semaphore) = old_semaphore {
                    self.semaphore_pool.return_semaphore(semaphore);
                }
                index
            }
        };

        self.swapchain.as_ref().unwrap().present_swapchain_image(
            index,
            self.per_frame_acquire_semaphore[index].as_ref().unwrap(),
        )?;

        Ok(())
    }
}

fn main() -> Result<()> {
    logging::setup()?;
    Application::<Example1ClearScreen>::new("Example 1 - Clear Screen")?.run()
}
