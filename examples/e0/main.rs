use anyhow::Result;
use ccthw::{
    application::{Application, GlfwWindow, State},
    graphics::vulkan_api::{Instance, RenderDevice},
    logging,
};

struct VulkanState {
    render_device: RenderDevice,
}

impl State for VulkanState {
    fn new(window: &mut GlfwWindow) -> Result<Self> {
        let render_device = window.create_render_device()?;
        Ok(Self { render_device })
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
