use anyhow::Result;
use ccthw::{
    application::{Application, GlfwWindow, State},
    logging,
};

/// The pattern in this project is for an application to be a type which
/// impements the State interface.
struct Example0AppLifecycle {}

impl State for Example0AppLifecycle {
    /// Create a new instance of my application. The GLFW window is provided
    /// for the creation of graphic resources and manipulating window settings.
    fn new(window: &mut GlfwWindow) -> Result<Self> {
        window.window_handle.set_key_polling(true);
        Ok(Self {})
    }

    /// Handle a GLFW window event.
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
            _ => (),
        }
        Ok(())
    }

    /// Update internal state and render a frame.
    fn update(&mut self, _glfw_window: &mut GlfwWindow) -> Result<()> {
        // currently a no-op
        Ok(())
    }
}

fn main() -> Result<()> {
    let _logger = logging::setup()?;
    Application::<Example0AppLifecycle>::new("Example 0 - App Lifecycle")?.run()
}
