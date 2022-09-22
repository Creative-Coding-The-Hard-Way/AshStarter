use anyhow::Result;
use ccthw::{
    application::{Application, GlfwWindow, State},
    logging,
};

struct VulkanState {
    //
}

impl State for VulkanState {
    fn new(_window: &mut GlfwWindow) -> Result<Self> {
        Ok(Self {})
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
