use anyhow::Result;
use ccthw::{
    application::{Application, GlfwWindow, State},
    graphics::vulkan_api::Instance,
    logging,
};

struct VulkanState {
    instance: Instance,
}

impl State for VulkanState {
    fn new(window: &mut GlfwWindow) -> Result<Self> {
        let instance = window.create_vulkan_instance()?;
        Ok(Self { instance })
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
