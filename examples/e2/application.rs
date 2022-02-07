//! This module defines the main application initialization, event loop, and
//! rendering.
use ::{anyhow::Result, ccthw::glfw_window::GlfwWindow};

// The main application state.
pub struct Application {
    glfw_window: GlfwWindow,
}

impl Application {
    /// Build a new instance of the application.
    ///
    /// Returns `Err()` if anything goes wrong while building the app.
    pub fn new() -> Result<Self> {
        let mut glfw_window = GlfwWindow::new("GLFW Lib")?;
        glfw_window.window.set_key_polling(true);
        Ok(Self { glfw_window })
    }

    /// Run the application, blocks until the main event loop exits.
    pub fn run(mut self) -> Result<()> {
        let event_receiver = self.glfw_window.take_event_receiver()?;

        // block the application on the window staying open
        while !self.glfw_window.window.should_close() {
            for (_, event) in
                self.glfw_window.flush_window_events(&event_receiver)
            {
                self.handle_event(event)?;
            }
        }
        Ok(())
    }

    fn handle_event(&mut self, event: glfw::WindowEvent) -> Result<()> {
        use glfw::{Action, Key, Modifiers, WindowEvent};
        match event {
            WindowEvent::Close => {
                self.glfw_window.window.set_should_close(true);
            }
            WindowEvent::Key(Key::Escape, _, Action::Press, _) => {
                self.glfw_window.window.set_should_close(true);
            }
            WindowEvent::Key(
                Key::Space,
                _,
                Action::Press,
                Modifiers::Control,
            ) => {
                self.glfw_window.toggle_fullscreen()?;
            }
            _ => {}
        }
        Ok(())
    }
}
