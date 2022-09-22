use std::sync::Arc;

use anyhow::Result;
use ccthw::{
    application::{Application, GlfwWindow, State},
    graphics::vulkan_api::Swapchain,
    logging,
};

struct VulkanState {
    _swapchain: Swapchain,
}

impl State for VulkanState {
    fn new(window: &mut GlfwWindow) -> Result<Self> {
        let (w, h) = window.window_handle.get_framebuffer_size();
        let render_device = Arc::new(window.create_render_device()?);
        let swapchain =
            Swapchain::new(render_device, (w as u32, h as u32), None)?;

        Ok(Self {
            _swapchain: swapchain,
        })
    }

    fn handle_event(
        &mut self,
        _glfw_window: &mut GlfwWindow,
        _window_event: glfw::WindowEvent,
    ) -> Result<()> {
        Ok(())
    }

    fn update(&mut self, _glfw_window: &mut GlfwWindow) -> Result<()> {
        Ok(())
    }
}

fn main() -> Result<()> {
    logging::setup()?;
    Application::<VulkanState>::new("Hello GLFW")?.run()
}
